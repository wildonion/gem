
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ognils {


    use super::*;

    pub fn start_game(ctx: Context<StartGame>, match_id: String, players: Vec<Pubkey>, player_commits: Vec<String>, amount: u64, rounds: u16, size: u16) -> Result<()> {
    
        let user_pda_account = &mut ctx.accounts.user_pda;
        let match_pda = &mut ctx.accounts.match_pda;
        let match_pda_account = match_pda.to_account_info();
        let user_pda_lamports = user_pda_account.to_account_info().lamports();
        let signer = ctx.accounts.signer.key();
        let server = ctx.accounts.server.key();

        if user_pda_lamports < amount {
            
            // revert logic, payback amount to players
            // ...
            
            return err!(ErrorCode::InsufficientFund);
        }

        if signer != server{
            return err!(ErrorCode::RestrictionError);
        } 


        **user_pda_account.try_borrow_mut_lamports()? -= amount;
        **match_pda_account.try_borrow_mut_lamports()? += amount;


        Ok(())

    } 

    pub fn finish_game(ctx: Context<FinishGame>, winners: Vec<Pubkey>) -> Result<()>{ 
        
        // withdraw from matchPDA and spread between winners equally
        // ...

        Ok(())
    }

    pub fn withdraw(ctx: Context<WithdrawFromUserPda>, bump: u8) -> Result<()>{

        let user_pda = &mut ctx.accounts.user_pda;
        let signer = &ctx.accounts.signer; //// only player can withdraw
        let player = &ctx.accounts.player;
        let pda = user_pda.to_account_info();
        let current_pda_amount = pda.lamports(); 

        if signer.key != player.key{
            return err!(ErrorCode::RestrictionError);
        }

        **pda.try_borrow_mut_lamports()? -= current_pda_amount;
        **player.try_borrow_mut_lamports()? += current_pda_amount;


        Ok(())
    }


}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Cell{
   pub x: u16,
   pub y: u16,
   pub value: u16,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Player{
   pub pub_key: Pubkey,
   pub table: Vec<Cell>,
}

impl Player{

    fn create_table(&mut self, size: u16) -> Vec<Cell>{
        self.table = Vec::with_capacity(size as usize);
        for _ in 0..size{
            self.table.push(Cell { x: 0, y: 0, value: 0 });
        }
        //// since self.table is behind a mutable reference we can't 
        //// move it around since rust doesn't allow use to move the 
        //// heap type if it's behind a muable pointer the soultion 
        //// can be either cloning which will return the type itself 
        //// or Self, borrowing (use their slice form) or dereferencing.
        let table = self.table.clone();
        table 
    }

    fn get_column_range(&self, x: u16) -> (u16, u16){
        for cell in self.table.clone(){
            if cell.x == x{
                return (cell.x, cell.y);
            }
        }
        return (0,0)   
    }

    fn create_announced_values(&mut self, size: u16, max_rounds: u16) -> Vec<Vec<Round>>{

        todo!()
    } 

}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Round{
    pub values: Vec<u16>,
}


#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct CurrentMatch{
   pub match_id: String,
   pub announced_values: Vec<Round>,
   pub players: Vec<Player>,
}


#[account]
pub struct MatchPda{
    current_match: CurrentMatch,
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct StartGame<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
   #[account(mut)]
   pub player: AccountInfo<'info>,
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   /// CHECK:
   #[account(init, payer = signer, space = 300, seeds = [b"slingo", player.key().as_ref()], bump)]
   pub user_pda: AccountInfo<'info>,
   #[account(init, payer = signer, space = 300, seeds = [match_id.as_bytes(), player.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct WithdrawFromUserPda<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>,
    #[account(mut, seeds = [b"slingo", player.key().as_ref()], bump = bump)]
    pub user_pda: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub player: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinishGame<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// the caller must be the server
    #[account(init, space = 300, payer = signer, seeds = [b"slingo", server.key().as_ref()], bump)]
    pub match_pda: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub server: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund!")]
    InsufficientFund,
    #[msg("Restriction error!")]
    RestrictionError,
}
