

use anchor_lang::prelude::*;
use mpl_token_metadata::instruction::burn_nft;

declare_id!("6oRp5W29ohs29iGqyn5EmYw2PQ8fcYZnCPr5HCdKwkp9"); //// this is the program public key of the the program wallet info which can be found in `target/deploy/whitelist-keypair.json` 

#[program]
pub mod conse_gem_whitelist {


    use super::*;

    pub fn burn_request(ctx: Context<BurnRequest>) -> Result<()>{
        
        let nft_stat = &mut ctx.accounts.nft_stat; //// nft_stat field is a mutabe field thus we have to get it mutably
        let signer_account = ctx.accounts.user.key();
        let nft_owner = nft_stat.owner; 

        //// checking that the transaction call signer
        //// which is the one who has called this method
        //// and sign it with his/her private key is the 
        //// NFT owner 
        if nft_owner != signer_account {
            return err!(ErrorCode::RestrictionError);
        }

        if nft_stat.burn_it(){

            // https://solana.stackexchange.com/questions/3746/how-can-i-create-hash-table-using-pdas
            // TODO - then call self.add_to_whitelist()
            // TODO - PDA hashmap for whitelist
            // there must be a mutable account that can mutate 
            // the whitelist on chain and its owner id must 
            // equals to the program id and must be the PDA account
            // ...    

            emit!(NftBurnEvent{
                owner: nft_owner,
                mint_address: nft_stat.mint,
            })

        } else{
            return err!(ErrorCode::InvalidBurnInstruction);
        }
            
        Ok(())
        
    }

}



#[account]
#[derive(Default)]
pub struct Nft{
    pub program_id: Pubkey,
    pub metadata: Pubkey,
    pub owner: Pubkey, //// this must be the signer of the burn_request method or the nft owner and also must be a writable account 
    pub mint: Pubkey,
    pub token: Pubkey,
    pub edition: Pubkey,
    pub spl_token: Pubkey,
    pub collection_metadata: Option<Pubkey>
}


impl Nft{

    pub const MAX_SIZE: usize = 1 + 32 + (7 * 32); //// 1 + 32 bytes is for Option null pointer optimisation and the Some part of the collection_metadata field
    pub fn burn_it(&self) -> bool{
        //// in order to burn the NFT 
        //// its owner must be the signer 
        //// of the transaction call or 
        //// the burn_request method.
        let transaction = burn_nft(
            self.program_id, 
            self.metadata, 
            self.owner, 
            self.mint, 
            self.token, 
            self.edition, 
            self.spl_token, 
            self.collection_metadata
        ); 
        //// if the instruction is executed by
        //// the current program thus we can 
        //// return true since everything went well. 
        if transaction.program_id == self.program_id{
            true
        } else{
            false
        }
    }

}


#[derive(Accounts)]
pub struct BurnRequest<'info> { //// 'info lifetime in function signature is required to store utf8 bytes or &[u8] instruction data in the accounts
    #[account(mut)]
    pub user: Signer<'info>, //// a mutable and a signer account which means this transaction call will be signed by a specific holder or NFT owner and is writable to make changes on it
    #[account(init, //// for the initial call nft_stat can't be mutable but we're telling anchor that this account is the owner of the program whic can mutate instruction data on the chain   
              space = 8 + Nft::MAX_SIZE, //// first 8 byte is the anchor discriminator and the rest is the size of the Nft struct
              payer = user //// the payer is the signer which must be the NFT owner, this constraint will be checked inside the burn_request method
            )]
    pub nft_stat: Account<'info, Nft>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Whitelist{}



#[error_code]
pub enum ErrorCode {
    #[msg("Signer Is Not The Nft Owner")]
    RestrictionError,
    #[msg("Invalid Nft Burn Instruction In Metaplex Candy Machine")]
    InvalidBurnInstruction,
}


#[event]
pub struct NftBurnEvent{
    pub owner: Pubkey,
    pub mint_address: Pubkey,
}