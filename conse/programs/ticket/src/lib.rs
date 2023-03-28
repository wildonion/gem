



use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash;
use percentage::Percentage;



declare_id!("bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk"); //// this is the program public (a 32 bytes slice array or [u8; 32] in Base-58 or Uint8 format) key which can be found in `target/deploy/conse-keypair.json`




pub fn generate_decks(player: Pubkey, bump: u8, iteration: u8) -> Option<Vec<Deck>>{
    
    let mut decks: Vec<Deck> = Vec::new();
    for deck in 0..iteration{ 
        
        //// ---------------- HASH NOTES 
        //// ----------------------------
        //// sha512 bits hash contains a slice of 64 bytes (each byte is a utf8 element) which can be shown in hex as of 128 chars or 64 packs of 2 chars in hex
        //// sha256 bits hash contains a slice of 32 bytes (each byte is a utf8 element) which can be shown in hex as of 64 chars or 32 packs of 2 chars in hex
        //// ----------------------------
        /// 
        //// since built in solana hash function
        //// doesn't support sha512 thus we must 
        //// generate two 32 bytes hash then
        //// concatenate them.
        let mut nonce = deck+1;
        let player_key_string = player.to_string(); //// we've convrted the public key into string since the type Pubkey will be moved in first input
        let first_32bytes_input = format!("{}${}${}${}", player_key_string, bump, deck, nonce);
        let first_hash = hash::hash(first_32bytes_input.as_bytes());
        let first_part_deck = first_hash.try_to_vec().unwrap();  
        nonce+=1;
        let second_32bytes_input = format!("{}${}${}${}", player_key_string, bump, deck, nonce);
        let second_hash = hash::hash(second_32bytes_input.as_bytes());
        let second_part_deck = &mut second_hash.try_to_vec().unwrap();  
        let mut new_deck = first_part_deck;
        
        new_deck.append(second_part_deck);
        new_deck = new_deck.into_iter().map(|byte|{
            if byte % 52 == 0{
                1
            } else{
                byte % 52
            }
        }).collect::<Vec<u8>>();   

        let generated_deck = Deck{
            data: {
                ///// ------------------- SHUFFLING PORCESS
                ///// -------------------------------------
                new_deck.reverse();
                let new_deck_len = new_deck.len();
                let mut card_index = 0;
                while card_index < new_deck_len{
                    let position = (card_index * 100) % new_deck_len; //// kinda random position, but we're happy :)
                    let prev_card = new_deck[card_index];
                    let new_card = new_deck[position];
                    new_deck[position] = prev_card;
                    new_deck[card_index] = new_card;
                    card_index+=1;
                } 

                //// we have to borrow the new_deck since it has no fixed size also it must be 
                //// mutable because we want to fill it with the first 12 bytes of the first
                //// deck data cards also final_deck must be a mutable slice since clone_from_slice() 
                //// method will borrow the self as mutable otherwise we'll get a compiler ERROR:  
                ////    cannot borrow `*last_deck_data` as 
                ////    mutable, as it is behind a `&` 
                ////    reference, `last_deck_data` is a 
                ////    `&` reference, so the data it refers 
                ////    to cannot be borrowed as mutable

                let first_deck = &decks.get(0); //// since indexing in rust returns a slice of the vector thus we have to put it behind a pointer
                let final_deck = if first_deck.is_some(){
                    let first_deck_data = first_deck.unwrap().data.as_slice(); 
                    new_deck[0..13].clone_from_slice(&first_deck_data[0..13]);
                    new_deck.to_vec()
                } else{
                    new_deck
                };

                let mut new_deck_iter = final_deck.into_iter().take(52); 
                let mut sliced_deck = vec![];
                while let Some(card) = new_deck_iter.next(){
                    sliced_deck.push(card);
                } 
                sliced_deck
                ///// -------------------------------------
                ///// -------------------------------------
            }
        };


        decks.push(generated_deck);

    }
    

    Some(decks)

}



#[program]
pub mod ticket {

    use super::*;

    pub fn start_game(ctx: Context<StartGame>, amount: u64, bump: u8, match_id: u8) -> Result<()> {
        
        let game_state = &mut ctx.accounts.game_state;
        let pda_lamports = game_state.to_account_info().lamports();
        
        //// amount is the total amount of the bet 
        //// including the server and player deposits
        //// thus if the total balance of the PDA 
        //// was not equal to the passed in amount 
        //// means that the PDA is not fully charged 
        //// because one of the server or client 
        //// didn't charge that. 
        if pda_lamports != amount { //// amount is the total amounts of PDA (server bet + player bet)
            return err!(ErrorCode::InsufficientFund);
        }

        game_state.server = *ctx.accounts.user.key;
        game_state.player = *ctx.accounts.player.key;
        game_state.amount = amount;
        game_state.bump = bump; //// NOTE - we must set the game state bump to the passed in bump coming from the frontend
        // game_state.bump = *ctx.bumps.get("game_state").unwrap(); // proper err handling    

        
        //// generating 10 random decks based
        //// on the player public key, bump and
        //// index iteration. 
        let decks = generate_decks(game_state.player, bump, 10);


        let match_info = MatchInfo{
            decks: if decks.is_some(){
                decks.unwrap()
            } else{
                vec![]
            },
            match_id,
            final_deck: vec![]
        };
        game_state.match_infos.push(match_info.clone()); 

        emit!(StartGameEvent{ 
            server: ctx.accounts.user.key(), 
            player: ctx.accounts.player.key(),
            match_info, 
            amount,
        });
        
        Ok(())
    
    }

    pub fn game_result(ctx: Context<GameResult>, winner: u8, instruct: u8, match_id: u8, deck: Vec<u16>) -> Result<()> { //// AnchorSerialize is not implement for [u8; 52] (u8 bytes with 52 elements)
        
        let game_state = &mut ctx.accounts.game_state;
        let match_infos = &game_state.match_infos;
        let signer_account = ctx.accounts.user.key();
        let server = game_state.server.key();
        let mut is_equal_condition = false;
        let mut event_tax_amount: u64 = 0;
        let mut winner_reward: u64 = 0;


        if server != signer_account { //// the signer of the tx call or the one who paid the gas fee is the server account itself
            return err!(ErrorCode::RestrictionError);
        }


        let pda = game_state.to_account_info();
        let amount = game_state.amount; //// amount is the total number of player and server deposit
        let revenue_share_wallet = ctx.accounts.revenue_share_wallet.to_account_info();
        //// calculating the general tax amount
        //// and transferring from PDA to revenue share wallet 
        let general_tax_amount = receive_amount(amount, 5); // general tax must be calculated from the total deposited amount since it's a general tax
        //// withdraw %5 fom PDA to fill the revenue share account 
        **pda.try_borrow_mut_lamports()? -= general_tax_amount;
        **revenue_share_wallet.try_borrow_mut_lamports()? += general_tax_amount;
        let total_amount_after_general_tax = amount - general_tax_amount;

    
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
            event_tax_amount = if instruct == 0 { //// we've defined the event_tax_amount earlier up
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
            // bet amount is: 2 (1 for server and 1 for player)
            // %5 of 2 SOL is 0.1 ---> 0.1 must go to revenue
            // 2 - 0.1 = 1.9 is the amounts inside the PDA
            // %25 of 1.9 SOL is 0.475 ---> 0.475 must to go revenue
            // 1.9 - 0.475 = 1.425 ---> must go to the winner
            winner_reward = total_amount_after_general_tax - event_tax_amount; //// we've defined the event_tax_amount earlier up
            //// withdraw event tax fom PDA to fill the revenue share account 
            **pda.try_borrow_mut_lamports()? -= event_tax_amount;
            **revenue_share_wallet.try_borrow_mut_lamports()? += event_tax_amount;
            //// withdraw amount receive fom PDA to fill the winner 
            **pda.try_borrow_mut_lamports()? -= winner_reward;
            **to_winner.try_borrow_mut_lamports()? += winner_reward;
        }

        //// -------------------- UPDATING FINAL DECK --------------------
        //// ------------------------------------------------------------- 
        let reveal_deck = deck.clone().into_iter().map(|card| card as u8).collect::<Vec<u8>>();
        let mut iter = match_infos.clone().into_iter(); //// since iterating through the iterator is a mutable process thus we have to define mutable
        while let Some(mut match_info) = iter.next(){
            if match_info.match_id == match_id {
                let mut decks_iter = match_info.decks.iter();
                while let Some(deck) = decks_iter.next(){
                    if reveal_deck.len() == 52 && reveal_deck.clone().into_iter().all(|card| deck.data.contains(&card)){
                        match_info.final_deck = reveal_deck.clone();
                    }
                }
                if match_info.final_deck.is_empty(){
                    return err!(ErrorCode::InvalidDeck);
                }
            } 
        }
        //// since we did a clone of match_infos to update the final_deck 
        //// thus the one inside the game_state won't be updated
        //// hence we have to update the game_state.match_infos by 
        //// setting it to a new one which is the updated match_infos.
        game_state.match_infos = match_infos.to_vec();
        //// -------------------------------------------------------------

        emit!(GameResultEvent{
            amount_receive: { ////--- we can also omit this
                if is_equal_condition{
                    0 as u64
                } else{
                    winner_reward
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

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)] //// no need to bound the Pda struct to `#[account]` proc macro attribute since this is not a generic instruction data
pub struct MatchInfo{
    pub decks: Vec<Deck>,
    pub match_id: u8,
    pub final_deck: Vec<u8>,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)] //// no need to bound the Pda struct to `#[account]` proc macro attribute since this is not a generic instruction data
pub struct Deck{
    pub data: Vec<u8>
}

#[account] //// means the following structure will be used to mutate data on the chain which this generic must be owned by the program or Account<'info, GameState>.owner == program_id
pub struct GameState { //// this struct will be stored inside the PDA
    server: Pubkey, // 32 bytes
    player: Pubkey, // 32 bytes
    amount: u64, // 8 bytes
    match_id: u8,
    match_infos: Vec<MatchInfo>, // all player matches and decks
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
//// with the #[instruction(..)] attribute we can access the instruction’s arguments 
//// we have to list them in the same order as in the instruction but 
//// we can omit all arguments after the last one you need.
// #[instruction(seed: [u8; 32])] 
pub struct StartGame<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    //// more than multiple AccountInfo 
    //// inside needs to be checked
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub player: AccountInfo<'info>,
    #[account(
        //// by using init we're telling anchor 
        //// that we want to initialize an account
        //// with the following details and constraints
        //// in which the payer of this account is the signer
        //// also the game_state is the owner that can 
        //// amend data on the the chain.
        //
        //// each field of type Account must be initialized first
        //// the it can be mutated in the next instruction call 
        init,
        //// payer of this transaction call is 
        //// the signer which is the user field
        payer = user, 
        // https://www.anchor-lang.com/docs/space
        // https://docs.metaplex.com/programs/understanding-programs#discriminators
        space = 4096, //// since we're storing decks on chain :) 
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
    //// these are safe or not safe. 
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
    #[msg("Invalid Deck")]
    InvalidDeck,
    #[msg("Unsuccessful Reservation")]
    UnsuccessfulReservation,
}


#[event]
pub struct StartGameEvent{
    pub server: Pubkey,
    pub player: Pubkey,
    pub match_info: MatchInfo,
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
