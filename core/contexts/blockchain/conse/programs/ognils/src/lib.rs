


/*

    for every player `init_user_pda` must be called before starting the game 
    to initialize all the player PDAs inside the queue for the current matc

    `init_match_pda` needs to be called by the server or a higher authority
    to initialize the match PDA account and initialize its first data on chain.

    `deposit` and `withdraw` both can be called by the user to deposit into 
    the match PDA and withdraw from the user PDA account.

    `start_game` will be called by the server after initializing the PDAs 
    to generate the game logic on chain thus all player public keys inside 
    the queue must be passed into this call. 

    `finish_game` must be called by the server after the game has finished 
    to pay the winners, thus it requires all the player PDAs to be passed 
    in to the call, also there must be 6 PDAs inside the call since maximum
    players inside the queue are 6 thus not all of them can be Some, it must 
    be checked for its Some part before paying the winner.  

*/




use anchor_lang::{prelude::*, solana_program::hash};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");




pub fn generate_cell_values_for_player(player_commit: String) -> Vec<u8>{

    let input = format!("{}${}", player_commit, 0); //// sha256 bits has 32 bytes length, each of which is in a range between 0 up to 255 
    let hash = hash::hash(input.as_bytes());
    let cells = hash.try_to_vec().unwrap();
    cells
}


pub fn create_table(size: u16, player_commit: String) -> Vec<Cell>{
    let mut cells: Vec<Cell> = vec![];
    pub fn is_duplicate(val: u16, col_vals: Vec<u16>) -> bool{
        for i in 0..col_vals.len(){
            if col_vals[i] == val{
                return true;
            }
        }
        return false;
    }
    let cell_values = crate::generate_cell_values_for_player(player_commit);
    
    for val in 0..size{
        let col_vals: Vec<u16> = vec![];
        let (min, max) = get_column_range(val);
    }


    cells
}

pub fn get_column_range(x: u16) -> (u16, u16){
    let min = x * 20;
    let max = (x + 1) * 20; 
    return (min, max)
}

pub fn create_announced_values(size: u16, max_rounds: u16) -> Vec<Vec<Round>>{
    todo!()
} 





#[program]
pub mod ognils {


    use super::*;
    
    pub fn init_match_pda(ctx: Context<InitMatchPda>, match_id: String, bump: u8) -> Result<()>{

        let server = &ctx.accounts.server;
        let match_pda = &mut ctx.accounts.match_pda;
        match_pda.current_match = CurrentMatch{
            match_id: match_id.clone(),
            bump,
            is_locked: false,
            server: server.key(),
            announced_values: vec![],
            players: vec![],
        };

        msg!("{:#?}", CurrentMatchEvent{ 
            match_id: match_id.clone(), 
            server: server.key(), 
            is_locked: false, 
            announced_values: vec![], 
            players: vec![] 
        });

        emit!(CurrentMatchEvent{ 
            match_id: match_id.clone(), 
            server: server.key(), 
            is_locked: false, 
            announced_values: vec![], 
            players: vec![] 
        });

        Ok(())

    }
    
    pub fn init_user_pda(ctx: Context<InitUserPda>, amount: u64) -> Result<()> {
        
        let user_pda = &ctx.accounts.user_pda;
        
        //// since there is no data inside user PDA account
        //// there is no need to mutate anything in here,
        //// chill and call next method :)
        
        // chill zone

        //...
        
        Ok(())

    } 

    pub fn withdraw(ctx: Context<WithdrawFromUserPda>, player_bump: u8, amount: u64) -> Result<()>{

        let user_pda = &mut ctx.accounts.user_pda;
        let signer = &ctx.accounts.signer; //// only player can withdraw
        let player = &ctx.accounts.player;
        //// accounts fields doesn't implement Copy trait 
        //// like Account fields are not Copy thus we must 
        //// borrow the ctx in order not to move 
        let match_pda = &mut ctx.accounts.match_pda; 
        let match_pda_account = match_pda.to_account_info();
        let current_match = match_pda.current_match.clone();
        let player_pda_balance = player.try_lamports()?; 

        if signer.key != player.key{
            return err!(ErrorCode::RestrictionError);
        }

        let index = current_match.players.iter().position(|p| p.pub_key == player.key());
        if index.is_some(){ //// we found a player
            let player_index = index.unwrap();
            let current_match_players = current_match.players.clone();
            let mut find_player = current_match_players[player_index].clone();
            if player_pda_balance > 0{
                **user_pda.try_borrow_mut_lamports()? -= amount;
                **player.try_borrow_mut_lamports()? += amount;
            } else{
                return err!(ErrorCode::PlayerBalanceIsZero);
            }
        } else{
            return err!(ErrorCode::PlayerDoesntExist);
        }

        Ok(())
        
    }

    pub fn deposit(ctx: Context<DepositToMatchPda>, amount: u64) -> Result<()>{

        let user_pda_account = &mut ctx.accounts.user_pda;
        let match_pda = &mut ctx.accounts.match_pda;
        let match_pda_account = match_pda.to_account_info();
        let user_pda_lamports = user_pda_account.to_account_info().lamports();
        let signer = ctx.accounts.signer.key();
        let server = ctx.accounts.server.key();
        
        // ----------------- finding a PDA logic ----------------- 
        // let program_id = ctx.accounts.system_program.to_account_info();
        // let player_pubkey = user_pda_account.key();
        // let player_seeds = &[b"slingo", player_pubkey.as_ref()]; //// this is of type &[&[u8]; 2]
        // let player_pda = Pubkey::find_program_address(player_seeds, &program_id.key()); //// output is an off curve public key and a bump that specify the iteration that this public key has generated 
        // let player_pda_account = player_pda.0;

        if user_pda_lamports < amount {
            return err!(ErrorCode::InsufficientFund);
        }

        if signer != server{
            return err!(ErrorCode::RestrictionError);
        } 

        **user_pda_account.try_borrow_mut_lamports()? -= amount;
        **match_pda_account.try_borrow_mut_lamports()? += amount;

        Ok(())

    }

    pub fn start_game(ctx: Context<StartGame>, players: Vec<PlayerInfo>, bump: u8,
                      rounds: u16, size: u16, match_id: String) -> Result<()>
    {

        let announced_values = create_announced_values(size, rounds);
        let server = &ctx.accounts.server;
        let server_pda = &mut ctx.accounts.match_pda; // a mutable pointer to the match pda since ctx.accounts fields doesn't implement Copy trait 
        
        let mut players_data = vec![]; 
        for player in players{
            let player_table = vec![];
            let player_instance = Player{
                pub_key: player.pub_key,
                table: player_table
            };

            create_table(size, player.commit); // creating the table with the passed in size 
            players_data.push(player_instance);
        }

        let current_match = CurrentMatch{
            match_id: match_id.clone(),
            bump,
            server: server.key(),
            is_locked: false,
            announced_values: announced_values.clone(), 
            players: players_data.clone()
        };

        server_pda.current_match = current_match; //// updating the current_match field inside the PDA 


        msg!("{:#?}", CurrentMatchEvent{ 
            match_id: match_id.clone(),  
            server: server.key(), 
            is_locked: false, 
            announced_values: announced_values.clone(), 
            players: players_data.clone() 
        });

        emit!(CurrentMatchEvent{ 
            match_id: match_id.clone(),  
            server: server.key(), 
            is_locked: false, 
            announced_values: announced_values.clone(), 
            players: players_data.clone(),
        });

        Ok(())
        
    }

    pub fn finish_game(ctx: Context<FinishGame>, player_bumps: Vec<u8>) -> Result<()>{ 
        
        //// since Copy traint is not implemented for ctx.accounts fields
        //// like AccountInfo and Account we must borrow the ctx and because 
        //// AccountInfo and Account fields don't imeplement Copy trait 
        //// we must borrow their instance if we want to move them or 
        //// call a method that takes the ownership of their instance 
        //// like unwrap() in order not to be moved. 


        let match_pda = &ctx.accounts.match_pda;
        // ----------------- players accounts ----------------------
        //// can't move out of a type if it's behind a shread reference
        //// if there was Some means we have winners
        let first_player_account = &ctx.accounts.first_user_pda;
        let second_player_account = &ctx.accounts.second_user_pda;
        let third_player_account = &ctx.accounts.third_user_pda;
        let fourth_player_account = &ctx.accounts.fourth_user_pda;
        let fifth_player_account = &ctx.accounts.fifth_user_pda;
        let sixth_player_account = &ctx.accounts.sixth_user_pda;
        let winners = vec![first_player_account, second_player_account, third_player_account,
                                                     fourth_player_account, fifth_player_account, sixth_player_account];
        
        let mut winner_count = 0;
        let current_match_pda_amout = **ctx.accounts.match_pda.try_borrow_lamports()?;
        if current_match_pda_amout > 0{

            let winner_flags = winners
                .into_iter()
                .map(|w|{
                    if w.is_some(){
                        winner_count += 1;
                        true
                    } else{
                        false
                    }
                })
                .collect::<Vec<bool>>();
            
            let winner_reward = current_match_pda_amout / winner_count; //// spread between winners equally

            for is_winner in winner_flags{
                //// every element inside winner_flags is a boolean map to the winner index inside the winners 
                //// vector also player accounts are behind a shared reference thus we can't move out of them
                //// since unwrap(self) method takes the ownership of the type and return the Self because 
                //// in its first param doesn't borrow the self or have &self, the solution is to use a borrow 
                //// of the player account then unwrap() the borrow type like first_player_account.as_ref().unwrap()
                //// with this way we don't lose the ownership of the first_player_account and we can call 
                //// the to_account_info() method on it.
                if is_winner{
                    let winner_account = first_player_account.as_ref().unwrap().to_account_info();
                    **winner_account.try_borrow_mut_lamports()? += winner_reward;
                    **match_pda.try_borrow_mut_lamports()? -= winner_reward;
                }
            }

        } else{
            return err!(ErrorCode::MatchPdaIsEmpty);
        }

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

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct PlayerInfo{
   pub pub_key: Pubkey,
   pub commit: String
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct Round{
    pub values: Vec<u16>,
}


#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct CurrentMatch{
   pub match_id: String,
   pub bump: u8,
   pub server: Pubkey,
   pub is_locked: bool,
   pub announced_values: Vec<Vec<Round>>,
   pub players: Vec<Player>,
}


#[account]
pub struct MatchPda{
    current_match: CurrentMatch,
}


#[derive(Accounts)]
pub struct InitUserPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// player
   #[account(mut)]
   pub player: AccountInfo<'info>,
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   /// CHECK:
   #[account(init, payer = signer, space = 300, seeds = [b"slingo", player.key().as_ref()], bump)]
   pub user_pda: AccountInfo<'info>,
   #[account(mut, seeds = [match_pda.current_match.match_id.as_bytes(), player.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct DepositToMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
   #[account(mut)]
   pub player: AccountInfo<'info>,
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   /// CHECK:
   #[account(mut, seeds = [b"slingo", player.key().as_ref()], bump)]
   pub user_pda: AccountInfo<'info>,
   #[account(mut, seeds = [match_pda.current_match.match_id.as_bytes(), player.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct InitMatchPda<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// server
   #[account(mut)]
   pub player: AccountInfo<'info>,
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   #[account(init, payer = signer, space = 300, seeds = [match_id.as_bytes(), player.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(match_id: String)]
pub struct StartGame<'info>{
   #[account(mut)]
   pub signer: Signer<'info>, //// only server
   /// CHECK:
   #[account(mut)]
   pub server: AccountInfo<'info>,
   #[account(init, payer = signer, space = 300, seeds = [match_id.as_bytes(), server.key().as_ref()], bump)]
   pub match_pda: Account<'info, MatchPda>,
   pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(player_bump: u8)]
pub struct WithdrawFromUserPda<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only player
    #[account(mut, seeds = [b"slingo", player.key().as_ref()], bump = player_bump)]
    pub user_pda: AccountInfo<'info>,
    #[account(mut, seeds = [match_pda.current_match.match_id.as_bytes(), 
                            match_pda.current_match.server.key().as_ref()], 
                            bump = match_pda.current_match.bump)]
    pub match_pda: Account<'info, MatchPda>,
    /// CHECK:
    #[account(mut)]
    pub player: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(match_id: String, player_bumps: Vec<u8>)]
pub struct FinishGame<'info>{
    #[account(mut)]  
    pub signer: Signer<'info>, //// only server
    #[account(init, space = 300, payer = signer, seeds = [b"slingo", server.key().as_ref()], bump)]
    pub match_pda: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub server: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, seeds = [b"slingo", first_user_pda.key().as_ref()], bump = player_bumps[0])]
    pub first_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"slingo", second_user_pda.key().as_ref()], bump = player_bumps[1])]
    pub second_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"slingo", third_user_pda.key().as_ref()], bump = player_bumps[2])]
    pub third_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"slingo", fourth_user_pda.key().as_ref()], bump = player_bumps[3])]
    pub fourth_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"slingo", fifth_user_pda.key().as_ref()], bump = player_bumps[4])]
    pub fifth_user_pda: Option<AccountInfo<'info>>,
    /// CHECK:
    #[account(mut, seeds = [b"slingo", sixth_user_pda.key().as_ref()], bump = player_bumps[5])]
    pub sixth_user_pda: Option<AccountInfo<'info>>,
    pub system_program: Program<'info, System>,
}


#[error_code]
pub enum ErrorCode {
    #[msg("Error InsufficientFund!")]
    InsufficientFund,
    #[msg("Restriction error!")]
    RestrictionError,
    #[msg("Player Doesn't Exist!")]
    PlayerDoesntExist,
    #[msg("Player Balance Is Zero!")]
    PlayerBalanceIsZero,
    #[msg("Match Is Locked!")]
    MatchIsLocked,
    #[msg("Match PDA Is Empty!")]
    MatchPdaIsEmpty,
}


#[event]
#[derive(Debug)]
pub struct CurrentMatchEvent{
    pub match_id: String,
    pub server: Pubkey,
    pub is_locked: bool,
    pub announced_values: Vec<Vec<Round>>,
    pub players: Vec<Player>,
}
