


use anchor_lang::prelude::*;
use percentage::Percentage;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod conse_gem_transaction {


    //// https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses
    //// https://docs.rs/anchor-lang/latest/anchor_lang/index.html
    //// https://solana.stackexchange.com/a/1480
    //// singer is the one who sign the transaction with his or her private key, 
    //// owner is the contract owner which the program is must be equals to the 
    //// owner public key or address, PDA is an off curve address with no private key 
    //// that can be used as a staking pool account for transferring and withdrawing 
    //// lamports since it has no private key thus no one can sign a transaction 
    //// call to that address to mutate the state of the account; the PDA can be generated 
    //// from a seed which can be a unique indentifer like public key plus a bump 
    //// which is a one byte number.
    //
    //// pda can be used to generate signature to be used for calling between programs
    //// since they have no private keys thus no third party can sign the transaction
    //// only the pda owner can do this (without private key) which can be used for 
    //// signing a transaction method call of another contract and also used for 
    //// depositing lamports as a escrow contract.
    
    
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
        space= 300, 
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
pub struct ReserveTicket{
    //// signer is the one who must pay 
    //// for the ticket and signed this 
    //// transaction method call also since we 
    //// want to take money from him/her
    //// the account must be mutable
    #[account(mut)]
    pub user: Signer<'info>,
    //// following will create the pda using
    //// server and the signer of this transaction
    //// call public key as the seed and the passed 
    //// in bump to start_game() function.
    //// NOTE that the generated pda in here 
    //// must be equals to the one in frontend
    #[account(
        mut,
        seeds = [ticket_stats.server.key().as_ref(), user.key().as_ref()],
        bump = ticket_stats.bump
    )]
    pub ticket_stats: Account<'info, GameState>, //// ticket_stats account is the PDA
    pub system_program: Program<'info, System>
}

#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund")]
    InsufficientFund,
    #[msg("Restriction error")]
    RestrictionError,
}
