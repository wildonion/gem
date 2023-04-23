use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ognils {

    use super::*;

    pub fn start_game(ctx: Context<StartGame>, bump: u8, amount: u64, match_id: u8) -> Result<()> {
    
        Ok(())
    
    }

    pub fn finish_game(ctx: Context<GameResult>) -> Result<()>{
        
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositIntoPda>) -> Result<()>{

        Ok(())
    }

    pub fn withdraw(ctx: Context<WithdrawFromPda>) -> Result<()>{

        Ok(())
    }


}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Cell{
   pub x: u16,
   pub y: u16,
   pub value: u16,
   pub is_marked: bool
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Player{
   pub pub_key: PubKey,
   pub table: Vec<Cell>,
}

impl Player{

    fn create_table(&mut self, size: u16) -> Vec<Cell>{
        self.table = (0..size)
            .map(|_|{
                0
            })
            .collect::<Vec<Cell>>();
    }

    fn get_column_range(&self, x: u16) -> (u16, u16){

    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Round{
    pub values: Vec<u16>,
}


#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct CurrentMatch{
   pub match_id: u16,
   pub announced_values: Vec<Round>,
}

#[account]
pub struct GameStatePda{
   pub current_match: CurrentMatch,
   pub players: Vec<Player>,
   pub player_deposit: u64,
   pub player_locked_deposit: u64,
   pub server_deposit: u64,
}

#[derive(Accounts)]
pub struct StartGame<'info>{
   pub signer: Signer<'info>,
   #[account(mut)]
   pub player: AccountInfo<'info>,
   #[account(init, payer = signer, sapce = 300, seeds = [signer.key().as_ref(), player.key().as_ref()])]
   pub game_state_pda: Account<'info, GameStatePda>,
   pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund!")]
    InsufficientFund,
    #[msg("Restriction error!")]
    RestrictionError,
}
