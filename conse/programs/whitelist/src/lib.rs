


use anchor_lang::prelude::*;
use mpl_token_metadata::instruction::burn_nft;

declare_id!("HTx9H8pfWeCGXfe7Hfap6LkVXgQNLNh5CnCSNRscFyCk"); //// this is the program public key of the the program wallet info which can be found in `target/deploy/whitelist-keypair.json` 



#[program] //// the program entrypoint which must be defined only once
pub mod whitelist {


    use super::*;


    pub fn burn_request(ctx: Context<BurnRequest>, nfts: Vec<Nft>) -> Result<()>{

        let owner = ctx.accounts.user.key;
        let is_owner = nfts.clone().into_iter().all(|nft| &nft.owner == owner);
        if is_owner{
            let flags = nfts.clone().into_iter().map(|nft| {
                if nft.burn_that(){
                    true
                } else{
                    false
                }
            }).collect::<Vec<bool>>();

            let all_of_them_burned = flags.into_iter().all(|flag| flag == true);
            if all_of_them_burned{

                emit!(NftBurnEvent{
                    owner: *owner, //// since owner is behind a reference we have to dereference it
                    mint_addresses: nfts,
                });

                Ok(())

            } else{
                return err!(ErrorCode::InvalidBurnInstruction);
            }
        } else{
            return err!(ErrorCode::RestrictionError);
        }
        
    }

    //// this instruction handler MUST be signed by a third party, secure 
    //// and a none NFT owner account like a server public key so we can use
    //// it as the whitelist state authority who can add PDAs to the list on 
    //// the chain, we'll check the signer of the `add_to_whitelist` instruction
    //// handler against the authority of the whitelist state.
    //
    //// also an initialization method is required to allocate whitelist structures
    //// on the whitelist account on the chain thus every init method needs 
    //// an account to store data on it.
    //
    //// since every data will be stored on an account thus every instruction 
    //// data must be deserialize on a sepcific account hence every method needs 
    //// a separate generic or data structure on the chain to be 
    //// loaded inside the account.
    pub fn initialize_whitelist(ctx: Context<IntializeWhitelist>, authority: Pubkey) -> Result<()>{
        
        //// `ctx.accounts.whitelist_state` can't be dereferenced because we can't move out of it 
        //// since it's behind a mutable shared reference of type `Account` which doesn't implement 
        //// Copy trait also rust doesn't allow us to move a type into other scopes or types 
        //// when there is a borrowed type of that type is being used by other scopes since 
        //// if we move that its lifetime will be dropped and its pointer will be a dangling 
        //// pointer, pointing to a location which doesn't exist which is dropped, thus if 
        //// it's inside an Option we can't move out of it too since methods of Option<T> 
        //// is the same as `T` methods means that Copy is not implemented for `Option<Account>` 
        //// we can either move it between threads and scopes by borrowing it or cloning it.
        //
        //// a shared reference can be in use by other scopes and threads thus moving out of 
        //// a shared referenced type requires one of the dereferencing methods which is either 
        //// copying (the Copy trait must be implemented), borrowing it using & or cloning 
        //// (Clone trait must be implemented) it or dereference it by using `*` otherwise 
        //// we can' move out of it in our case `whitelist_state` which is of type Account, can't be moved 
        //// since `Account` type doesn't implement Copy trait we have to either borrow it or clone it. 

        let signer = ctx.accounts.user.key(); //// this can be a server or a none NFT owner which has signed this instruction handler and can be used as the whitelist state authority 
        let whitelist_state = &mut ctx.accounts.whitelist_state;
        let whitelist_data = &mut ctx.accounts.whitelist_data; //// a mutable reference to the whitelist data account, since Account is a mutable shared reference which doesn't implement Copy trait we can't move out of it thus we have to either clone it or borrow it 
        whitelist_data.list = vec![]; //// creating empty vector on chain
        whitelist_state.authority = authority; //// the signer must be the whitelist_state authority
        whitelist_state.counter = 0;

        Ok(())

    }

    //// this instruction handler must be called from the server or 
    //// where the whitelist state authority wallet info exists.
    //// in centralized servers the security check of the caller 
    //// can be done using a JWT or a dev token.
    pub fn add_to_whitelist(ctx: Context<AddToWhitelistRequest>, addresses: Vec<Pda>) -> Result<()>{

        let signer = ctx.accounts.authority.key();
        let whitelsit_state = &mut ctx.accounts.whitelist_state;
        let who_initialized_whitelist = whitelsit_state.authority.key(); //// the whitelist owner 
        let whitelist_data = &mut ctx.accounts.whitelist_data.list; //// a mutable reference to the whitelist data account, since Account is a mutable shared reference which doesn't implement Copy trait we can't move out of it thus we have to either clone it or borrow it 
        let mut counter = whitelsit_state.counter as usize;

        //// the signer of this tx call must be the one
        //// who initialized the whitelist instruction handler
        //// or the server account must pay for gas fee
        //// because the burner shouldn't pay for the whitelist
        //// gas fee.
        if signer != who_initialized_whitelist{
            return err!(ErrorCode::WhitelistOwnerRestriction);
        }

        let length = addresses.len();
        if counter > WhitelistData::MAX_SIZE{ //// make sure that we have enough space
            return err!(ErrorCode::NotEnoughSpace);
        }

        msg!("[+] current counter: {}", counter);
        let mut current_data = Vec::<Pda>::new();
        current_data.extend_from_slice(&whitelist_data[0..counter]); //// counter has the current size of the total PDAs, so we're filling the vector with the old PDAs on chain
        counter += length;


        //// vector types are dynamic sized means
        //// their size is not known at compile time
        //// since our PDAs is of type array thus we 
        //// need to know the exact size of the elements
        //// inside of it and also we need to convert the
        //// vector into the array or the slice form after 
        //// adding a new PDA inside of it hence after 
        //// adding one PDA into the vector we must fill
        //// the rest of the vector with a default keys.
        for _ in counter..WhitelistData::MAX_SIZE{ //// if the counter is full then the starting point will be the MAX_SIZE itself thus the loop won't be run
            current_data.push(Pda::default()); //// filling the rest of the vector with default public key
        }
        msg!("[+] vector length on chain {:?}", current_data.len());

        *whitelist_data = current_data; //// since whitelist_data is behind a reference we have to dereference it
        whitelsit_state.counter = counter as u64;
        msg!("[+] current counter after adding one PDA: {}", counter);

        Ok(())

    }


}



#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, Default)] //// no need to bound the Pda struct to `#[account]` proc macro attribute since this is not a generic instruction data
pub struct Pda{
    pub owner: Pubkey,
    pub nfts: Vec<Pubkey>, //// the NFT mint addresses that are burned
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)] //// no need to bound the Nft struct to `#[account]` proc macro attribute since this is not a generic instruction data
pub struct Nft{
    pub program_id: Pubkey,
    pub metadata: Pubkey,
    pub owner: Pubkey, //// this must be the signer of the `burn_request` method or the nft owner and also must be a writable account 
    pub mint: Pubkey,
    pub token: Pubkey,
    pub edition: Pubkey,
    pub spl_token: Pubkey,
    pub collection_metadata: Option<Pubkey>,
}


impl Nft{

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
        msg!("after burn : account >>> {:#?}", accounts);
        msg!("after burn : data >>> {:#?}", data);
        msg!("after burn : program id >>> {:#?}", the_program_id);

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



//// since everything on solana will be stored 
//// inside accounts thus the following structure 
//// contains all the required accounts to use them
//// inside the contract, they can be of type either
//// `Account` which can contains the serialized 
//// instruction data to be mutated on the chain by 
//// its owner or `AccountInfo` which is just an account 
//// info contains public key without the 
//// serialized instruction data.
//
//// Accounts trait bounding will add the AnchorSerialize 
//// and AnchorDeserialize traits to the generic or 
//// the BurnRequest struct.
#[derive(Accounts)] 
pub struct BurnRequest<'info> { //// 'info lifetime in function signature is required to store utf8 bytes or &[u8] instruction data in the accounts
    #[account(mut)]
    pub user: Signer<'info>, //// this is the NFT owner which must pay for the gas fee
    pub system_program: Program<'info, System>, //// this refers to this program itself and it can be another program
}


impl<'info> BurnRequest<'info>{}

#[derive(Accounts)]
pub struct IntializeWhitelist<'info>{
    // anchor `#[account]` proc macro attribute constraint guide: https://docs.rs/anchor-lang/latest/anchor_lang/derive.Accounts.html
    #[account( //// can't use mut constraint here since we're initializing this account
        //// data store on solana accounts and if it's not exists on the 
        //// runtime `init` will initialize the account via CPI call to the 
        //// runtime and sets its owner to the program id by default since
        //// only the program must be able to mutate its generic instruction data 
        //// on chain by deserializing it using borsh hence to use an account 
        //// that owns a generic data on chain like `WhitelistState` data which 
        //// is owned by the `whitelist_state` account, the account which is of 
        //// type `Account` must be initialized first and limited to a space.
        //
        //// in initial call or creating the whitelist_state, 
        //// account can't be mutable but we're telling 
        //// anchor that this account is the owner of the 
        //// program which can mutate instruction 
        //// data on the chain.   
        //
        //// we can't use `whitelist_state` fields since
        //// in here we're initializing the `whitelist_state` in the
        //// first step and we don't have access to that field. 
        //
        //// when we initialize an account it must be limited to 
        //// some allocated space on chain since solana stores 
        //// everything inside accounts and thus they must be 
        //// sized and limited to space allocation on chain.
        //// ------------------------------------------------------
        //// --- init also requires space and payer constraints ---
        //// ------------------------------------------------------
        init, //// initializing the whitelist_state account 
        payer = user, //// init requires payer (the signer of this tx call) who must pay for the gas fee and account creation
        space = 50 //// total space required for this account on chain which is the size of its generic or `WhitelistState` struct or WhitelistState::MAX_SIZE
    )]
    //// `#[account()]` proc macro attribute is on top of the `whitelist_state` field thus the generic of this account, the `WhitelistState` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_state: Account<'info, WhitelistState>, //// this account is also the singer of the transaction call that means it must pay for the gas fee
    #[account(zero)] //// zero constraint is necessary for accounts that are larger than 10 Kibibyte also will deserialize using a zero copy technique because those accounts cannot be created via a CPI (which is what init would do)
    //// `#[account()]` proc macro attribute is on top of the `whitelist_data` field thus the generic of this account, the `WhitelistData` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_data: Account<'info, WhitelistData>, //// since `WhitelistData` is a zero copy data structure we must use `AccountLoader` for deserializing it
    #[account(mut)]
    pub user: Signer<'info>, //// signer or payer of this tx call which must be mutable since it's the payer for initializing the `whitelist_state` account that must pay for the call which leads to decreasing lamports from his/her account
    pub system_program: Program<'info, System>, //// when we use `init` system program account info must be exists

}

#[derive(Accounts)]
// #[instruction(burn_tx_hash: String)] //// we can access the instruction's arguments here which are passed inside the `add_to_whitelist` instruction handler with the #[instruction(..)] attribute
pub struct AddToWhitelistRequest<'info>{
    //// if we want to take money from someone, 
    //// we should make them sign as well as mark 
    //// their account as mutable.
    #[account(mut)]
    pub authority: Signer<'info>, //// the transaction signer account which must be mutable
    #[account(
        //// we need to define this account mutable or 
        //// writable since we want to add PDAs to it in runtime
        mut, 
    )]
    //// `#[account()]` proc macro attribute is on top of the `whitelist_data` field thus the generic of this account, the `WhitelistData` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_data: Account<'info, WhitelistData>, //// `AccountLoader` will be used to deserialize zero copy types 
    #[account(
        //// we need to define this account mutable or 
        //// writable since we want to mutate the 
        //// white list state at runtime
        mut,
        //// `has_one` constraint will check 
        //// that whitelist_state.owner == authority.key()
        has_one = authority
    )]
    //// `#[account()]` proc macro attribute is on top of the `whitelist_state` field thus the generic of this account, the `WhitelistState` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    //
    //// `whitelist_state` account must be initialized first which 
    //// this can be done inside the `initialize_whitelist` handler
    //// in its generic or `IntializeWhitelist` struct by putting 
    //// init, payer and space constraint on top of it.
    pub whitelist_state: Account<'info, WhitelistState>,
}

//// structs that are bounded to this proc macro attribute, are the instruction data
//// which are the generic of the account (`Account` type) that wants to mutate them on the chain and 
//// their fields can be accessible by calling the `program.account.<STRUCT_NAME>`
//// in frontend side which allows us to get current value of each field on chain,
//// here the owner of this struct is the `whitelist_data` account field which its 
//// owner must be the program id so it can mutate the deserialized data on the chain although
//// the `#[account]` proc macro on top of the generic `T` or Nft in here will set 
//// the owner of the `Account` type that contains the generic `T` to the program id since 
//// the account must be the owner of the program in order to mutate data on the chain
//
//// `#[account]` proc macro attribute sets 
//// the owner of that data to the 
//// `declare_id` of the crate also 
//// we can deserialize this account
//// on frontend to get the list field
#[account]
pub struct WhitelistData{ // https://solana.stackexchange.com/questions/2339/account-size-calculation-when-using-vectors
    // https://solana.stackexchange.com/questions/2339/account-size-calculation-when-using-vectors
    //// heap data size like string and vector are always 24 bytes
    //// which will be stored on the stack: 8 bytes for capaciy, 8 bytes for length
    //// and the last one is the size of their pointer which points to a heap-allocated buffer
    //// but borsh actually serializing a slice of the vector thus the size of the following 
    //// list will be 4 + (32 * N) which N referes to the number of elements and 32 is the
    //// size of public key and 4 is the size of one public key since the public key is of
    //// type u32 which 4 bytes long.
    //
    //// dynamic size can be in their borrowed form like &str or &[u8],
    //// since str and [u8] don't have fixed size at compile time 
    //// we must use them behind a reference or the borrowed form of their dynamic size
    //// about the &[u8] we can also use its fixed size like [0u8; 32] also
    //// we have to pass types into other scopes and threads by reference or 
    //// in their borrowed form and know this that if a pointer of a type is being 
    //// used by other threads and scopes we can't move the type itself into another
    //// type or other scopes since rust doesn't allow use to do this because there 
    //// is a shared pointer that is being in used by other scopes and we must either
    //// borrow the type or clone it to move it into ther scopes.
    pub list: Vec<Pda>, //// list of all PDAs that shows an owner has burnt his/her NFT
}

impl WhitelistData{
    pub const MAX_SIZE: usize = 2000;
}

//// structs that are bounded to this proc macro attribute, are the instruction data
//// which are the generic of the account (`Account` type) that wants to mutate them on the chain and 
//// their fields can be accessible by calling the `program.account.<STRUCT_NAME>`
//// in frontend side which allows us to get current value of each field on chain,
//// here the owner of this struct is the `whitelist_state` account field which its 
//// owner must be the program id so it can mutate the deserialized data on the chain although
//// the `#[account]` proc macro on top of the generic `T` or Nft in here will set 
//// the owner of the `Account` type that contains the generic `T` to the program id since 
//// the account must be the owner of the program in order to mutate data on the chain
//
//// `#[account]` proc macro attribute sets 
//// the owner of that data to the 
//// `declare_id` of the crate
#[account]
pub struct WhitelistState{
    pub authority: Pubkey, //// the owner of the whitelist state
    pub counter: u64, //// total number of PDAs inside the whitelist
}

impl WhitelistState{
    pub const MAX_SIZE: usize = 32 + 8;
}


#[derive(Accounts)]
// #[instruction(burn_tx_hash: String)] //// we can access the instruction's arguments here which are passed inside the `remove_from_whitelist` instruction handler with the #[instruction(..)] attribute
pub struct RemoveFromWhitelistRequest<'info>{ //// this is exactly like the `AddToWhitelistRequest` struct but will be used for removing a PDA 
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)] //// this account must be mutable since it wants to mutate the instruction data of type `WhitelistData` on the chain by removing a PDA from the list
    //// `#[account()]` proc macro attribute is on top of the `whitelist_data` field thus the generic of this account, the `WhitelistData` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_data: Account<'info, WhitelistData>, 
    #[account(mut, has_one = authority)]
    pub whitelist_state: Account<'info, WhitelistState>, //// `#[account()]` proc macro attribute is on top of the `whitelist_state` field thus the generic of this account, the `WhitelistState` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id  
}



#[error_code]
pub enum ErrorCode {
    #[msg("Signer Is Not The Nft Owner")]
    RestrictionError,
    #[msg("Invalid Nft Burn Instruction")]
    InvalidBurnInstruction,
    #[msg("Not Enough Space")]
    NotEnoughSpace,
    #[msg("Whitelist Owner Restriction")]
    WhitelistOwnerRestriction,
    #[msg("PDA Is Already Added")]
    PdaIsAlreadyAdded
}


#[event]
pub struct NftBurnEvent{
    pub owner: Pubkey,
    pub mint_addresses: Vec<Nft>,
}
