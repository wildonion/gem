

use anchor_lang::prelude::*;
use percentage::Percentage;
declare_id!("bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk"); //// this is the program public key which can be found in `target/deploy/conse-keypair.json`

#[program]
pub mod ticket {


    // https://github.com/coral-xyz/anchor/tree/master/tests/cpi-returns
    // https://github.com/switchboard-xyz/vrf-demo-walkthrough


    use super::*;

    //// amount is the total amount of the bet 
    //// including the server and player deposits
    //// thus if the total balance of the PDA 
    //// was not equal to the passed in amount 
    //// means that the PDA is not fully charged 
    //// because one of the server or client 
    //// didn't charge that. 
    pub fn start_game(ctx: Context<StartGame>, amount: u64, bump: u8, match_id: u8) -> Result<()> {
        
        let game_state = &mut ctx.accounts.game_state;
        let pda_lamports = game_state.to_account_info().lamports();
        if pda_lamports != amount { //// amount is the total amounts of PDA (server bet + player bet)
            return err!(ErrorCode::InsufficientFund);
        }

        game_state.server = *ctx.accounts.user.key;
        game_state.player = *ctx.accounts.player.key;
        game_state.match_id = match_id;
        game_state.amount = amount;
        game_state.bump = bump; //// NOTE - we must set the game state bump to the passed in bump coming from the frontend
        // game_state.bump = *ctx.bumps.get("game_state").unwrap(); // proper err handling    
        

        // TODO
        // generate deck in here 
        // ...
        game_state.deck = vec![0, 1, 43, 56, 34];


        emit!(StartGameEvent{ 
            server: ctx.accounts.user.key(), 
            player: ctx.accounts.player.key(), 
            amount,
        });
        
        Ok(())
    
    }

    pub fn game_result(ctx: Context<GameResult>, winner: u8, instruct: u8) -> Result<()> {
        
        let game_state = &mut ctx.accounts.game_state;
        let signer_account = ctx.accounts.user.key();
        let server = game_state.server.key();
        let mut is_equal_condition = false;
        let mut amount_receive: u64 = 0;
        let mut event_tax_amount: u64 = 0;


        if server != signer_account { //// the signer of the tx call or the one who paid the gas fee is the server account itself
            return err!(ErrorCode::RestrictionError);
        }


        let pda = game_state.to_account_info();
        let amount = game_state.amount;
        let revenue_share_wallet = ctx.accounts.revenue_share_wallet.to_account_info();
        //// calculating the general tax amount
        //// and transferring from PDA to revenue share wallet 
        let total_amount_after_general_tax = receive_amount(amount, 5); // general tax must be calculated from the deposited amount since it's a general tax
        let general_tax_amount = amount - total_amount_after_general_tax;
        //// withdraw %5 fom PDA to fill the revenue share account 
        **pda.try_borrow_mut_lamports()? -= general_tax_amount;
        **revenue_share_wallet.try_borrow_mut_lamports()? += general_tax_amount;

    
        let mut to_winner = match winner{
            0 => Some(ctx.accounts.player.to_account_info()),
            1 => Some(ctx.accounts.server.to_account_info()),
            3 => {
                
                // equal condition

                let pda_amount = **pda.try_borrow_mut_lamports()?;
                let server_account = ctx.accounts.server.to_account_info();
                let player_account = ctx.accounts.player.to_account_info();
                
                    let half = (pda_amount / 2) as u64;
                    **pda.try_borrow_mut_lamports()? -= half; //// double dereferencing pda account since try_borrow_mut_lamports() returns RefMut<&'a mut u64>
                    **server_account.try_borrow_mut_lamports()? += half; //// double dereferencing server account since try_borrow_mut_lamports() returns RefMut<&'a mut u64>

                    **player_account.try_borrow_mut_lamports()? += half; //// double dereferencing player account since try_borrow_mut_lamports() returns RefMut<&'a mut u64>
                    **pda.try_borrow_mut_lamports()? -= half; //// double dereferencing pda account since try_borrow_mut_lamports() returns RefMut<&'a mut u64>

                    is_equal_condition = true;
                    
                    None
            
            },
            _ => return err!(ErrorCode::InvalidWinnerIndex),
        };
        
  
   
        //// we're sure that we have a winner
        if !is_equal_condition && to_winner.is_some(){
            //// every types and variable that 
            //// are defined here are only accessible
            //// to this scope since their lifetimes 
            //// out of this if block will be dropped,
            //// thanks to the rust :) which doesn't 
            //// collect garbages.
            let to_winner = to_winner.unwrap();
            //// calculating the amount that must be sent
            //// the winner from the PDA account based on
            //// instruction percentages.
            //
            //// we've assumed that the third instruction 
            //// is the event with 25 percent special tax.
            amount_receive = if instruct == 0 { //// we've defined the amount_receive earlier up
                receive_amount(total_amount_after_general_tax, 95)
            } else if instruct == 1 {
                receive_amount(total_amount_after_general_tax, 70)
            } else if instruct == 2 {
                receive_amount(total_amount_after_general_tax, 35)
            } else if instruct == 3 {
                receive_amount(total_amount_after_general_tax, 25)
            } else if instruct == 4 {
                receive_amount(total_amount_after_general_tax, 0)
            } else {
                return err!(ErrorCode::InvalidInstruction);
            };

            ///////////////////////////////////////////////////
            ////////// CALCULATING TAX BASED ON THE INSTRUCTION
            ///////////////////////////////////////////////////
            //--------------------------------------------
            // we must withdraw all required lamports 
            // from the PDA since the PDA 
            // has all of it :)
            //--------------------------------------------
            // bet amount      : 1    SOL - %5  = 0.95 -> 1    - 0.95 = 0.05 must withdraw for general tax to revenue share wallet 
            // amount after tax: 0.95 SOL - %25 = 0.24 -> 0.95 - 0.24 = 0.71 must withdraw for %25 tax to revenue share wallet 
            event_tax_amount = total_amount_after_general_tax - amount_receive; //// we've defined the event_tax_amount earlier up
            //// withdraw event tax fom PDA to fill the revenue share account 
            **pda.try_borrow_mut_lamports()? -= event_tax_amount;
            **revenue_share_wallet.try_borrow_mut_lamports()? += event_tax_amount;
            //// withdraw amount receive fom PDA to fill the winner 
            **pda.try_borrow_mut_lamports()? -= amount_receive;
            **to_winner.try_borrow_mut_lamports()? += amount_receive;
        }
        

        emit!(GameResultEvent{
            amount_receive: { ////--- we can also omit this
                if is_equal_condition{
                    0 as u64
                } else{
                    amount_receive
                }
            }, ////--- we can also omit this
            event_tax_amount: { ////--- we can also omit this
                if is_equal_condition{
                    0 as u64
                } else{
                    event_tax_amount
                }
            }, ////--- we can also omit this
            winner: { ////--- we can also omit this
                if winner == 0{
                    Some(ctx.accounts.player.key())
                } else if winner == 1{
                    Some(ctx.accounts.server.key())
                } else{
                    None
                }
            }, ////--- we can also omit this
            is_equal: is_equal_condition,
        });

        Ok(())
    
    }

    pub fn return_deck_info(ctx: Context<DeckInfo>) -> Result<Vec<u8>>{

        //// we can't move game_state which is of type Account
        //// since Vec<u8> doesn't implement Copy thus we have
        //// to either borrow it or clone it.
        let deck = ctx.accounts.game_state.deck.clone(); 
        Ok(deck)

    }

    pub fn reserve_ticket(ctx: Context<ReserveTicket>, deposit: u64, user_id: String, bump: u8) -> Result<()>{

        let ticket_stats = &mut ctx.accounts.ticket_stats;
        let pda_lamports = ticket_stats.to_account_info().lamports();
        let pda_account = ticket_stats.to_account_info(); //// ticket_stats is the PDA account itself
        let staking_pool_account = ctx.accounts.satking_pool.to_account_info(); //// this is only an account info which has no instruction data to mutate on the chain 

        //// the lamports inside the PDA account 
        //// must equals to the deposited amount
        //// also we've created a PDA account to 
        //// deposit all the tickets in there. 
        if pda_lamports != deposit{ 
            return err!(ErrorCode::InsufficientFund);
        }

        ticket_stats.amount = deposit;
        ticket_stats.bump = bump;
        ticket_stats.id = user_id.clone();

        //// since try_borrow_mut_lamports returns 
        //// Result<RefMut<&'a mut u64>> which is a
        //// RefMut type behind a mutable pointer
        //// we must dereference it in order to 
        //// mutate its value
        //
        //// *pda_account.try_borrow_mut_lamports()?
        //// returns &mut u64 which requires another
        //// dereference to mutate its value, after 
        //// tranferring the balance of the PDA
        //// must be zero
        **pda_account.try_borrow_mut_lamports()? -= deposit; //// withdraw from PDA account that has been charged inside the frontend
        **staking_pool_account.try_borrow_mut_lamports()? += deposit; //// deposit inside the conse staking pool account

        if **pda_account.try_borrow_mut_lamports()? != 0 as u64{
            return err!(ErrorCode::UnsuccessfulReservation);
        }


        emit!(ReserveTicketEvent{
            deposit,
            user_id
        });

        Ok(())
    }

}   

fn receive_amount(amount: u64, perc: u8) -> u64{
    let percent = Percentage::from(perc);
    let amount_receive = percent.apply_to(amount);
    amount_receive
}

#[account] //// means the following structure will be used to mutate data on the chain which this generic must be owned by the program or Account<'info, GameState>.owner == program_id
pub struct GameState { //// this struct will be stored inside the PDA
    server: Pubkey, // 32 bytes
    player: Pubkey, // 32 bytes
    amount: u64, // 8 bytes
    match_id: u8,
    deck: Vec<u8>, // the deck of this player for the passed in match id
    bump: u8, //// this must be filled from the frontend; 1 byte
}

//// the `#[account]` proc macro on top 
//// of the generic `T` or TicketStats 
//// in here will set the owner of the 
//// `Account` type that contains the 
//// generic `T` to the program id since 
//// the account must be the owner of the 
//// program in order to mutate data on the chain
//
//// `#[account]` proc macro attribute sets 
//// the owner of that data to the 
//// `declare_id` of the crate
#[account] //// means the following structure will be used to mutate data on the chain which this generic must be owned by the program or Account<'info, TicketStats>.owner == program_id
pub struct TicketStats{
    id: String, //// the mongodb objectid
    server: Pubkey, //// this is the server solana public key
    amount: u64,
    bump: u8, 

}


#[derive(Accounts)] //// means the following structure contains Account and AccountInfo fields which can be used for mutating data on the chain if it was Account type 
pub struct StartGame<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        //// by using init we're telling anchor 
        //// that we want to initialize an account
        //// with the following details and constraints
        //// in which the payer of this account is the signer
        //// also the game_state is the owner that can 
        //// amend data on the the chain.
        init,
        //// payer of this transaction call is 
        //// the signer which is the user field
        payer = user, 
        // https://www.anchor-lang.com/docs/space
        // https://docs.metaplex.com/programs/understanding-programs#discriminators
        //// the space that is required to store
        //// GameState data which in total is:
        //// 8 + (32 * 3) + 8 + 1 in which any 
        //// public key or amount higher than 32
        //// will throw an error also the first 
        //// 8 bytes will be used as discriminator 
        //// by the anchor to point to a type like 
        //// the one in enum tag to point to a variant.
        space = 300, 
        //// following will create the PDA using
        //// user which is the signer and player 
        //// one public keys as the seed and the 
        //// passed in bump to start_game() function.
        //// NOTE that the generated PDA in here 
        //// must be equals to the one in frontend
        seeds = [user.key().as_ref(), player.key().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub player: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)] //// means the following structure contains Account and AccountInfo fields which can be used for mutating data on the chain if it was Account type
pub struct GameResult<'info> {
    //// if we want to take money from someone, we should make them sign 
    //// to call pay transaction method call as well as mark their account 
    //// as mutable also if the singer constraint is available means that 
    //// the account field must be the signer of this transaction or 
    //// this deposit method
    #[account(
        //// make the signer account mutable or writable 
        //// which enable us to make changes to this account
        //// like deposit/withdraw lamports into/from their accounts,
        //// means signer and writable at the same time, this 
        //// combination is pretty common since programs will 
        //// usually require the owner of an account to prove 
        //// who they are with their private key before mutating 
        //// that account otherwise, anyone could mutate any 
        //// account they don't own without needing the private
        //// key of that account.
        mut
    )] 
    //// since we have one signer account which must be mutable
    //// we have to put a CHECK doc for other accounts that are of 
    //// type Account which are also mutable or writable since they 
    //// are safe and allow us to make changes to those accounts like 
    //// transferring lamports from another account which 
    //// makes some write to the account. 
    //
    //// a writable account will be mutated by the instruction 
    //// this information is important for the blockchain to 
    //// know which transactions can be run in parallel 
    //// and which ones can't.
    pub user: Signer<'info>,
    #[account(
        mut,
        //// following will create the PDA using
        //// server and player one public keys as 
        //// the seed and the passed in bump to 
        //// start_game() function.
        //// NOTE that the generated PDA in here 
        //// must be equals to the one in frontend
        seeds = [game_state.server.key().as_ref(), game_state.player.key().as_ref()], 
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub server: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub player: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we just pay to this account (general tax account)
    #[account(mut)]
    pub revenue_share_wallet: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct DeckInfo<'info>{
    #[account(mut, seeds = [game_state.server.key().as_ref(), game_state.player.key().as_ref()], bump = game_state.bump)]
    pub game_state: Account<'info, GameState>,
}

// https://docs.rs/anchor-lang/latest/anchor_lang/derive.Accounts.html
// https://docs.metaplex.com/programs/understanding-programs#signer-andor-writable-accounts
#[derive(Accounts)] //// means the following structure contains Account and AccountInfo fields which can be used for mutating data on the chain if it was Account type
pub struct ReserveTicket<'info>{
    //// signer is the one who must pay 
    //// for the ticket and signed this 
    //// transaction method call with his
    //// or her private key also since we 
    //// want to take money from him or her
    //// the account must be mutable since 
    //// he or she must pay for the gas fee also.
    #[account(mut)]
    pub user: Signer<'info>, //// the signer who must sign the call and pay for the transaction fees
    /*
        // https://solana.stackexchange.com/questions/26/what-is-a-program-derived-address-pda-exactly/1480#1480
        // https://solana.stackexchange.com/a/1480

        following will create the PDA from the 
        ticket_stats using server and the signer 
        of this transaction call public key as 
        the seed and the passed in bump to 
        start_game() function.
        NOTE that the generated PDA in here 
        must be equals to the one in frontend
    
        the `Account` type is used when an instruction
        is interested in the deserialized data of the
        account means if we have a data coming from the 
        transaction call we can store it inside the 
        `Account` type with the `'info` lifetime
        since `Account` type is generic over `T` which `T`
        is the struct that contains the instruction data
        that can be deserialized using borsh which will be 
        bounded to the `T` once we added the `#[account]`
        proc macro attribute on top of it.
    
        the `Account` type will verify that the owner of 
        generic `T` or the `TicketStats` struct equals the
        address we declared with `declare_id`.

    */
    #[account(
        //// since this is a new PDA account we must initialize it 
        //// using init contraint which do a CPI call to the runtime 
        //// to set its owner to the program id in order to be able 
        //// to mutate the `TicketStats` instruction generic data 
        init, 
        space = 300,
        payer = user,
        //// we can't use ticket_stats.server.key().as_ref() 
        //// since the ticket_stats is not initialized yet thus 
        //// we can't use its fields.
        seeds = [server.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    //// declaration of account owned by the 
    //// program for storing data on chain means
    //// that the owner is the program owner
    //// and can store and mutate data on chain.
    //// since init constraint will initialize 
    //// this account on solana runtime via a CPI call
    //// in which its owner by default is the program id
    //// also since this is a PDA account it must be initialized
    //// in order to use it later inside another 
    //// instruction handler method. 
    pub ticket_stats: Account<'info, TicketStats>, 
    /// CHECK: this is safe since it's a server account and we want to use it to build the PDA
    #[account(mut)]
    pub server: AccountInfo<'info>,
    //// `AccountInfo` type don't implement any checks 
    //// on the account being passed and we can fix the
    //// compile time error by writing a CHECK doc.
    /// CHECK: This is not dangerous because we don't read and write from this account and we'll transfer lamports of the bought ticket to this account 
    #[account(mut)]
    //// this is the staking pool account that will be used 
    //// to transfer the paid amount to this one also the type
    //// `AccountInfo` will be used to only indicate that the 
    //// following is just an account without any instruction 
    //// data and if we want to deserialize a data we must 
    //// use `Account` type which is a wrapper around the 
    //// `AccountInfo` type.
    //
    //// more than one `AccountInfo` fields inside the struct
    //// needs to be checked and tell solana that 
    //// these are safe. 
    pub satking_pool: AccountInfo<'info>, 
    pub system_program: Program<'info, System> //// this can also be another program instead of System
}

#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund!")]
    InsufficientFund,
    #[msg("Error InsufficientFund In Equal Condition!")]
    InsufficientFundEqualCondition,
    #[msg("Restriction error!")]
    RestrictionError,
    #[msg("Invalid Winner Index")]
    InvalidWinnerIndex,
    #[msg("Invalid Instruction")]
    InvalidInstruction,
    #[msg("Unsuccessful Reservation")]
    UnsuccessfulReservation
}


#[event]
pub struct StartGameEvent{
    pub server: Pubkey,
    pub player: Pubkey,
    pub amount: u64,
}

#[event]
pub struct GameResultEvent{
    pub amount_receive: u64,
    pub event_tax_amount: u64,
    pub winner: Option<Pubkey>, //// since it might be happened the equal condition which there is no winner  
    pub is_equal: bool,
}

#[event]
pub struct ReserveTicketEvent{
    pub deposit: u64,
    pub user_id: String,
}
