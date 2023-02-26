


use anchor_lang::prelude::*;
use mpl_token_metadata::instruction::burn_nft;

declare_id!("6oRp5W29ohs29iGqyn5EmYw2PQ8fcYZnCPr5HCdKwkp9"); //// this is the program public key of the the program wallet info which can be found in `target/deploy/whitelist-keypair.json` 

#[program]
pub mod conse_gem_whitelist {


    //// every function in here 
    //// is a separate instruction

    use super::*;

    pub fn burn_request(ctx: Context<BurnRequest>, bump: u8) -> Result<()>{
        
        let nft_stats = &mut ctx.accounts.nft_stats; //// nft_stats field is a mutabe field thus we have to get it mutably
        let signer_account = ctx.accounts.user.key();
        let nft_owner = nft_stats.owner; 

        //// we could also use the found bump by the anchor 
        //// when initializing the nft_stats account over
        //// generic `Nft` struct inside the `BurnRequest` struct
        // user_stats.bump = *ctx.bumps.get("user_stats").unwrap();
        nft_stats.bump = bump; //// set the bump to the passed in bump from the frontend

        //// checking that the transaction call signer
        //// which is the one who has called this method
        //// and sign it with his/her private key is the 
        //// NFT owner 
        if nft_owner != signer_account {
            return err!(ErrorCode::RestrictionError);
        }

        if nft_stats.burn_that(){

            emit!(NftBurnEvent{
                owner: nft_owner,
                mint_address: nft_stats.mint,
            });

        } else{
            return err!(ErrorCode::InvalidBurnInstruction);
        }
            
        Ok(())
        
    }

    pub fn add_to_whitelist(ctx: Context<AddToWhitelistRequest>) -> Result<()>{

        let signer = ctx.accounts.user.key();
        let nft_owner = ctx.accounts.nft_owner.key();
        let server = ctx.accounts.nft_stats.server;

        //// the signer of this transaction or instruction call
        //// must be the server account means that a regular 
        //// user can't call this transaction.
        if signer != server{ 
            return err!(ErrorCode::AddToWhitelistRequestRestriction);
        }

        //// nft owner account field inside the `AddToWhitelistRequest` struct
        //// related to the `add_to_whitelist` instruction must be equals 
        //// with the owner field inside the generic `Nft` which can be 
        //// accessible by calling it on the PDA account or the `nft_stats`
        //// account which is the owner of the generic `Nft` that can mutate 
        //// the instruction data of type `Nft` on the chain since it's owner id
        //// is equals to the program id.
        if nft_owner != ctx.accounts.nft_stats.owner{
            return err!(ErrorCode::NftOwnerDifferentWithPDANftOwner);
        } 

        //// this is the `nft_stats` field which is the PDA
        //// account that can mutate instruction data on the chain
        let pda_account = *ctx.accounts.nft_stats.to_account_info().key;
        let whitelist = &mut ctx.accounts.whitelist.list;
        whitelist.push(pda_account);

        Ok(())

    }

}



#[account]
#[derive(Default)]
pub struct Nft{
    pub program_id: Pubkey,
    pub metadata: Pubkey,
    pub owner: Pubkey, //// this must be the signer of the `burn_request` method or the nft owner and also must be a writable account 
    pub mint: Pubkey,
    pub token: Pubkey,
    pub edition: Pubkey,
    pub spl_token: Pubkey,
    pub collection_metadata: Option<Pubkey>,
    pub bump: u8,
    pub server: Pubkey, //// this is required for calling the `add_to_whitelist` instruction by the server
}


impl Nft{

    pub const MAX_SIZE: usize = 1 + 32 + (7 * 32); //// 1 + 32 bytes is for Option null pointer optimisation and the Some part of the collection_metadata field
    pub fn burn_that(&self) -> bool{
        //// in order to burn the NFT 
        //// its owner must be the signer 
        //// of the transaction call or 
        //// the `burn_request` method.
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

        //// an instruction has program_id, data and accounts
        let accounts = transaction.accounts;
        let data = transaction.data;
        let the_program_id = transaction.program_id;
        
        //// if the instruction is executed by
        //// the current program thus we can 
        //// return true since everything went well. 
        if the_program_id == self.program_id{
            true
        } else{
            false
        }
    }

}


#[account]
#[derive(Default)]
pub struct Whitelist{
    // https://solana.stackexchange.com/questions/2339/account-size-calculation-when-using-vectors
    //// heap data size like string and vector are always 24 bytes
    //// which will be stored on the stack: 8 bytes for capaciy, 8 bytes for length
    //// and the last one is the size of their pointer which points to a heap-allocated buffer
    //// but borsh actually serializing a slice of the vector thus the size of the following 
    //// list will be 4 + (32 * N) which N referes to the number of elements and 32 is the
    //// size of public key and 4 is the size of one public key since the public key is of
    //// type u32 which 4 bytes long.
    //
    //// the maximum size of any account is 10 megabytes
    //// we can store an array of nft owners inside a whitelist 
    //// variable so if 5000 owners burnt their nfts our contract 
    //// size on chain will be about ~0.160004 MB which has calculated
    //// by 4 + (32 * 5000).
    pub list: Vec<Pubkey>, //// list of all PDAs that shows an owner has burnt his/her NFT
}


#[derive(Accounts)] //// Accounts trait bounding will add the AnchorSerialize and AnchorDeserialize traits to the generic or the BurnRequest struct
pub struct BurnRequest<'info> { //// 'info lifetime in function signature is required to store utf8 bytes or &[u8] instruction data in the accounts
    #[account(mut)]
    pub user: Signer<'info>, //// a mutable and a signer account which means this transaction call will be signed by a specific holder or NFT owner and is writable to make changes on it
    //// bump is a utf8 bytes format so we can create it from a unique string also
    //// when init is used with seeds and bump, it will always search for the canonical bump
    //// this means that it can only be called once (because the 2nd time it's called 
    //// the PDA will already be initialized). 
    #[account( // https://www.anchor-lang.com/docs/pdas
              //// `init` immediately shouts at us and tells 
              //// us to add a payer because init creates 
              //// rent-exempt accounts and someone has 
              //// to pay for that.
              //
              //// `init` wants the system program to be inside 
              //// the struct because init creates the `nft_stats` 
              //// account by making a call to that program.
              //
              //// in initial call or creating the PDA account, 
              //// `nft_stats` can't be mutable but we're telling 
              //// anchor that this account is the owner of the 
              //// program which can mutate instruction 
              //// data on the chain.   
              init, //// --- init also requires space and payer constraints --- 
              space = 8 + Nft::MAX_SIZE, //// first 8 byte is the anchor discriminator and the rest is the size of the Nft struct
              payer = user, //// the payer is the signer which must be the NFT owner, this constraint will be checked inside the `burn_request` method
              seeds = [user.key().as_ref(), nft_mint.key().as_ref()], //// the following is the PDA account that can be created using the signer public key which is the nft owner and the nft mint address to create the whitelist id; as_ref() converts the public key of each account into utf8 bytes  
              bump //// we're adding an empty bump constraint to signal to anchor that it should find the canonical bump itself, then in the `burn_request` handler, we call ctx.bumps.get("nft_statss") to get the bump anchor found and save it to the nft stats account as an extra property
            )]
    //// this is the account that can mutate the generic Nft 
    //// on the contract and its owner is the program id, 
    //// in a sence the PDA account must only be owned by the 
    //// program or account.owner == program_id
    //
    //// the PDA account that will be used to create whitelist
    //// based on the nft owner and the nft mint address and 
    //// only the program will have control over it.
    //
    //// an account that stores an instruction data on the 
    //// chain must be of type `Account` which owns the generic 
    //// inside of itself and its owner must be the program itself
    //// to be able to mutate data on the chain since writing to 
    //// an account that is not owned by the program will 
    //// cause the transaction failed.
    //
    //// `Account` types are wrapper around `AccountInfo` 
    //// that verifies program ownership and deserializes 
    //// underlying data into a Rust type.
    pub nft_stats: Account<'info, Nft>,
    //// more than one mutable account of type `AccountInfo` 
    //// needs to be checked also with `AccountInfo` type we 
    //// can mutate nothing on chain since it has no passed 
    //// in generic instruction data and only it can be used 
    //// to make some changes on the account itself like 
    //// transferring lamports from it to another accounts 
    //// which must be mutable or use it to create the PDA like 
    //// the following account that will be used to create the 
    //// whitelist PDA.
    pub nft_mint: AccountInfo<'info>, 
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddToWhitelistRequest<'info>{
    //// if we want to take money from someone, 
    //// we should make them sign as well as mark 
    //// their account as mutable.
    #[account(mut)]
    pub user: Signer<'info>, //// the transaction signer account
    //// if we then want to use the created PDA in 
    //// a different instruction like the `add_to_whitelist` 
    //// method, we can add a new validation struct which 
    //// in our case is `AddToWhitelistRequest`, this will be checked
    //// by the anchor at runtime that the nft_stats account 
    //// field is the PDA created by running 
    //// hash(seeds, nft_stats.bump, program_id)):
    #[account( // https://www.anchor-lang.com/docs/pdas 
        //// instruction data will be stored in accounts 
        //// in solana thus we need to make the account 
        //// writable or mutable by putting the mut constraint
        //// also the owner of the account will be checked by 
        //// `#[account]` proc macro attribute which will grant
        //// access to the account to mutate and deserialize 
        //// the passed in instruction data if its owner 
        //// was equals to program id.
        mut, 
        seeds = [nft_owner.key().as_ref(), nft_stats.mint.key().as_ref()], //// the following is the PDA account that can be created using the nft owner and the nft mint address to create the whitelist id
        bump = nft_stats.bump //// use the nft_stats bump itself which has been founded inside the frontend
    )]
    //// this is the account that can mutate the generic `Nft` 
    //// on the contract and its owner is the program id, 
    //// in a sence the PDA account must only be owned by the 
    //// program or account.owner == program_id
    //
    //// the PDA account that will be used to create whitelist
    //// based on the nft owner and the nft mint address also 
    //// they can be used to create hashmap structures and in 
    //// cpi calls and they must be mutable and owned by the program
    //// in order to be able to mutate their generic (`Nft` in our case)
    //// on chain.
    pub nft_stats: Account<'info, Nft>, 
    #[account(
        //// we need to define this account mutable or 
        //// writable since we want to add PDAs to it in runtime
        //
        //// since this is not initialize account we don't 
        //// need to add space and payer constraint.
        mut, 
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub nft_owner: AccountInfo<'info>, //// the NFT owner account that will be used to create the PDA
}



#[error_code]
pub enum ErrorCode {
    #[msg("Signer Is Not The Nft Owner")]
    RestrictionError,
    #[msg("Invalid Nft Burn Instruction In Metaplex Candy Machine")]
    InvalidBurnInstruction,
    #[msg("Signer Is Not The Server")]
    AddToWhitelistRequestRestriction,
    #[msg("Nft Owner Is Different With The PDA Nft Owner")]
    NftOwnerDifferentWithPDANftOwner
}


#[event]
pub struct NftBurnEvent{
    pub owner: Pubkey,
    pub mint_address: Pubkey,
}