


/*


https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses
https://docs.rs/anchor-lang/latest/anchor_lang/index.html
https://solana.stackexchange.com/a/1480


======================================
============= ABOUT SOLANA WALLET INFO 
======================================

`solana-keygen new` will generate a new wallet info which contains 
public and private keys produced from the elliptic curve algorithm
based on Ed25519 which is a public key digital signature which means
that a message came from the holder of a certain private key and that 
the information has not been tampered with in flight; the hash of the 
pulic key will be used as the wallet address or it can be used as it is
in its raw format and the private key to sign transaction method calls 
to make sure that the public key of the transaction call or the signer 
info is the one who signed the call with his or her private key, also 
the private key is a pen that can be used to sign 
every program transaction call.  

=============================================
============= ABOUT NULL POINTER OPTIMISATION
=============================================

borsh uses a null-pointer optimization in serializing Option means it takes 
extra 1 byte instead of allocating extra 8 bytes tag which is used to 
point to the current variant; by this it serializes an Option as 1 byte for the 
variant identifier and then additional x bytes for the content if it's Some
otherwise there will be just 1 byte to avoid null pointer or zero bytes,
a null-pointer optimization means a reference can never be null since 
Option<&T> is the exact size of the T because in enum the size of the 
whole enum is equals to the size of the biggest variant, in Option enum 
and all enums with two variants instead of requiring an extra word or 8 bytes 
tag which can points to the current variant of the enum we can use the size of T
with 1 extra byte to represent the tag to make sure that there is 
no invalid pointer or reference.

=========================================
=============  SOLANA RUNTIME EXPLANATION
=========================================

solana runtime has its own bpf loader which supports no std libs
since contracts can't interact with the ouside world thus there 
is no socket to do this due to the securtiy reasons although
the reason that solana contract gets compiled to .so is because 
they can be loaded from the linux kernel which is blazingly 
fast also from the browsers, a json RPC call must be invoked 
with a contract method name and id (wallet address or public key) 
to the RPC server on the solana runtime node to load the .so contract 
which has bee deployed and contains the BPF bytecode in it to call 
the method name inside the incoming RPC request 
to change the state of the blockchain.

=========================================
============= SOLANA ACCOUNTS EXPLANATION
=========================================

singer is the one who sign the transaction with his or her private key, 
owner is the contract owner which the program is must be equals to the 
owner public key or address, PDA is an off curve address with no private key 
that can be used as a staking pool account for transferring and withdrawing 
lamports since it has no private key thus no one can sign a transaction 
call to that address to mutate the state of the account; the PDA can be generated 
from a seed which can be a unique indentifer like public key plus a bump 
which is a one byte number.

pda can be used to generate signature to be used for calling between programs
since they have no private keys thus no third party can sign the transaction
only the pda owner can do this (without private key) which can be used for 
signing a transaction method call of another contract and also used for 
depositing lamports as a escrow contract.

*/

use anchor_lang::prelude::*;
use percentage::Percentage;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod conse_gem_reservation {
    
    use super::*;

    pub fn start_game(ctx: Context<StartGame>, amount: u64, bump: u8) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        let pda_lamports = game_state.to_account_info().lamports();
        if pda_lamports != amount {
            return err!(ErrorCode::InsufficientFund);
        }

        game_state.server = *ctx.accounts.user.key;
        game_state.player_one = *ctx.accounts.player_one.key;
        game_state.amount = amount;
        game_state.bump = bump; //// NOTE - we must set the game state bump to the passed in bump coming from the frontend
        // game_state.bump = *ctx.bumps.get("game_state").unwrap(); // proper err handling    
    
        
        Ok(())
    }


    pub fn reserve_ticket(ctx: Context<ReserveTicket>, amount: u64) -> Result<()>{

        Ok(())
    }

    pub fn second_player(ctx: Context<SecondPlayer>, amount: u64) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        let lamports_before_second_player = game_state.amount;
        let pda_lamports = game_state.to_account_info().lamports();
        if (pda_lamports - lamports_before_second_player) != amount {
            return err!(ErrorCode::InsufficientFund);
        }
        game_state.player_two = *ctx.accounts.player_two.key;
        game_state.amount += amount;
        Ok(())
    }

    pub fn game_result(ctx: Context<GameResult>, winner: u8, instruct: u8) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;

        let signer_account = ctx.accounts.user.key();
        let server = game_state.server.key();

        if server != signer_account {
            return err!(ErrorCode::RestrictionError);
        }

        let amount = game_state.amount;
        let pda = game_state.to_account_info();
        let to = if winner == 0 {
            ctx.accounts.player_one.to_account_info()
        } else if winner == 1 {
            ctx.accounts.player_two.to_account_info()
        } else {
            panic!("err!")
        };
        let tax = ctx.accounts.tax_account.to_account_info();

        let amount_recieve = if instruct == 0 {
            get_amount(amount, 98)
        } else if instruct == 1 {
            get_amount(amount, 88)
        } else if instruct == 2 {
            get_amount(amount, 48)
        } else if instruct == 3 {
            get_amount(amount, 38)
        } else if instruct == 4 {
            get_amount(amount, 0)
        } else {
            panic!("err!")
        };

        let tax_amount = amount - amount_recieve;

        fn get_amount(amount: u64, perc: u8) -> u64{
            let percent = Percentage::from(perc);
            let amount_recieve = percent.apply_to(amount);
            amount_recieve
        }

        // amount sent to winner
        **pda.try_borrow_mut_lamports()? -= amount_recieve;
        **to.try_borrow_mut_lamports()? += amount_recieve;
        // tax amount
        **pda.try_borrow_mut_lamports()? -= tax_amount;
        **tax.try_borrow_mut_lamports()? += tax_amount;

        Ok(())
    }
}   

#[account]
pub struct GameState {
    server: Pubkey,
    player_one: Pubkey,
    player_two: Pubkey,
    amount: u64,
    bump: u8, //// this must be filled from the frontend
}

//// `#[account]` proc macro attribute sets 
//// the owner of that data to the 
//// `declare_id` of the crate
#[account]
pub struct TicketStats{
    pub id: String, //// the mongodb objectid
    pub server: Pubkey, //// this is the server solana public key
    pub bump: u8, 

}

#[derive(Accounts)]
pub struct StartGame<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer= user,
        space= 300, // https://www.anchor-lang.com/docs/space
        //// following will create the pda using
        //// user which is the signer and player 
        //// one public keys as the seed and the 
        //// passed in bump to start_game() function.
        //// NOTE that the generated pda in here 
        //// must be equals to the one in frontend
        seeds = [user.key().as_ref(), player_one.key().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub player_one: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SecondPlayer<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        //// following will create the pda using
        //// server and player one public keys as 
        //// the seed and the passed in bump to 
        //// start_game() function.
        //// NOTE that the generated pda in here 
        //// must be equals to the one in frontend
        seeds = [game_state.server.key().as_ref(), game_state.player_one.key().as_ref()], 
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub player_two: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GameResult<'info> {
    //// if we want to take money from someone, we should make them sign as well as 
    //// mark their account as mutable also if the singer constraint is available 
    //// means that the account field must be the signer of this transaction or 
    //// this deposit method
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        //// following will create the pda using
        //// server and player one public keys as 
        //// the seed and the passed in bump to 
        //// start_game() function.
        //// NOTE that the generated pda in here 
        //// must be equals to the one in frontend
        seeds = [game_state.server.key().as_ref(), game_state.player_one.key().as_ref()], 
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub player_one: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub player_two: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we just pay to this account
    #[account(mut)]
    pub tax_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct ReserveTicket<'info>{
    //// signer is the one who must pay 
    //// for the ticket and signed this 
    //// transaction method call with his
    //// or her private key also since we 
    //// want to take money from him or her
    //// the account must be mutable
    #[account(mut)]
    pub user: Signer<'info>,
    /*

        following will create the pda from the 
        ticket_stats using
        server and the signer of this transaction
        call public key as the seed and the passed 
        in bump to start_game() function.
        NOTE that the generated pda in here 
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
        mut,
        seeds = [ticket_stats.server.key().as_ref(), user.key().as_ref()],
        bump = ticket_stats.bump
    )]
    pub ticket_stats: Account<'info, TicketStats>,
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
    pub satking_pool: AccountInfo<'info>, 
    pub system_program: Program<'info, System>
}

#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund")]
    InsufficientFund,
    #[msg("Restriction error")]
    RestrictionError,
}
