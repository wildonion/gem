


use anchor_lang::prelude::*;
use mpl_token_metadata::instruction::burn_nft;

declare_id!("4ZdXCpgo5wZTVbh1QV2yjcsiX1jCSzsqkfWeYwXwcAU2"); //// this is the program public key of the the program wallet info which can be found in `target/deploy/whitelist-keypair.json` 



pub fn gen_program_pub_key(program_id: &str) -> Option<Pubkey>{

    let program_id_bytes = program_id.as_bytes(); //// convert the &str into slice of utf8 bytes
    let program_id_bytes_vec = program_id_bytes.to_vec(); //// we must convert the bytes into vector to build the [u8; 44]
    
    //////////////////////
    /////// first approach
    //////////////////////
    let mut pubkey: [u8; 32] = Default::default();
    pubkey[..program_id_bytes.len()].clone_from_slice(program_id_bytes); //// clone_from_slice will clone a new one from the passed in utf8 slice 

    ///////////////////////
    /////// second approach
    ///////////////////////
    let program_id_bytes_fixed_size: [u8; 32] = match program_id_bytes_vec.try_into(){ //// converting the vector into the slice form or array with a fixed size of 32 bytes which is the size of each public key
        Ok(data) => data,
        Err(e) => [0u8; 32], //// returning an empty array of zero with a fixed size of 32 bytes 
    };
    if !program_id_bytes_fixed_size
            .into_iter()
            .all(|byte| byte == 0){
                let program_public_key = Pubkey::from(program_id_bytes_fixed_size);
                Some(program_public_key)
            } else{
                None //// returning None if the public key contains zero bytes means that we couldn't convert the vector into [u8; 44] correctly
            }
}

#[program] //// the program entrypoint which must be defined only once
pub mod whitelist {


    //// `AccountLoader` type and zero_copy attribute
    //// will be used to deserialize the zero copy types
    //// or the borrowed form of dynamic types.
    
    //// every function in here 
    //// is a separate instruction
    //// and every instruction
    //// requires separate structure
    //// to handle data on chain.
    
    //// we will need to pass the accounts through 
    //// the context in order to be able to access 
    //// its data, this design allows Solana to parallelize 
    //// transactions better by knowing which accounts 
    //// and data is required before runtime.
    


    use super::*;

    pub fn burn_request(ctx: Context<BurnRequest>, bump: u8) -> Result<()>{
        
        let nft_stats = &mut ctx.accounts.nft_stats; //// nft_stats field is a mutabe field thus we have to get it mutably
        let signer_account = ctx.accounts.user.key();

        //// `ctx.accounts.collection_metadata` can't be 
        //// dereferenced because we can't move out of it 
        //// since it's behind a mutable reference of type 
        //// `AccountInfo` which doesn't implement Copy trait 
        //// thus if it's inside an Option we can't move out 
        //// of it too since methods of Option<T> is the same as 
        //// `T` methods means that Copy is not implemented for 
        //// `Option<AccountInfo>`, we can either move it between 
        //// threads and scopes by borrowing it or cloning it.  
        //
        //// a shared reference can be in use by other scopes and threads
        //// thus moving out of a shared referenced type requires one 
        //// of the dereferencing methods which is either copying
        //// (the Copy trait must be implemented), borrowing it using & 
        //// or cloning (Clone trait must be implemented) it or dereference
        //// it by using `*` otherwise we can' move out of it.
        let collection_metadata = ctx.accounts.collection_metadata.clone(); 

        //// we could also use the found bump by the anchor 
        //// when initializing the nft_stats account over
        //// generic `Nft` struct inside the `BurnRequest` struct
        // user_stats.bump = *ctx.bumps.get("user_stats")?;
        nft_stats.bump = bump; //// setting the bump to the passed in bump from the frontend
        nft_stats.program_id = *ctx.accounts.the_program_id.key;
        nft_stats.metadata = *ctx.accounts.metadata.key;
        nft_stats.token = *ctx.accounts.token.key;
        nft_stats.mint = *ctx.accounts.nft_mint.key; 
        nft_stats.owner = signer_account; //// set the owner field of the `nft_stats` account to the signer of this instruction handler since only the NFT owner can call this method and pay for the transaction fees
        nft_stats.edition = *ctx.accounts.edition.key;
        nft_stats.spl_token = *ctx.accounts.spl_token.key;
        nft_stats.collection_metadata = if let Some(collection_metadata_account) = collection_metadata{
            Some(collection_metadata_account.key())
        } else{
            None
        };

        // TODO - this will broke the program at runtime
        // with error: Error processing Instruction 0: Program failed to complete
        //// checking the passed in program id from then frontend
        //// into the accounts section of this instruction handler 
        //// against the current program id.
        // let this_program_id_public_key = gen_program_pub_key("4ZdXCpgo5wZTVbh1QV2yjcsiX1jCSzsqkfWeYwXwcAU2");
        // if let Some(pub_key) = this_program_id_public_key{
        //     if nft_stats.program_id != pub_key{
        //         return err!(ErrorCode::AccessDeniedDueToInvalidProgramId);
        //     }
        // } else{
        //     return err!(ErrorCode::RuntimeError);
        // }


        // ------------------ Burning Process -----------------------

        if nft_stats.burn_that(){
            emit!(NftBurnEvent{ //// log the burn event 
                owner: signer_account,
                mint_address: nft_stats.mint,
            });
        } else{
            return err!(ErrorCode::InvalidBurnInstruction);
        }

        // ------------------------------------------------------------
            
        Ok(())
        
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
        
        let signer = ctx.accounts.user.key(); //// this can be a server or a none NFT owner which has signed this instruction handler and can be used as the whitelist state authority 
        let whitelist_state = &mut ctx.accounts.whitelist_state;
        let whitelist_data = ctx.accounts.whitelist_data.load_init()?; //// a mutable reference to the whitelist data account loader
        let mut wl_data = whitelist_data.to_owned(); //// to_owned() will convert the borrowed data into the owned data
        wl_data.list = [Pubkey::default(); 5000]; // TODO - need to change the 5000 since it's the total number of PDAs that must be inside the list
        whitelist_state.authority = authority; //// the signer must be the whitelist_state authority
        whitelist_state.counter = 0;

        Ok(())

    }

    //// this instruction handler must be called from the server or 
    //// where the whitelist state authority wallet info exists.
    //// in centralized servers the security check of the caller 
    //// can be done using a JWT or a dev token.
    pub fn add_to_whitelist(ctx: Context<AddToWhitelistRequest>) -> Result<()>{

        let signer = ctx.accounts.authority.key();
        let whitelsit_state = &mut ctx.accounts.whitelist_state;
        let who_initialized_whitelist = whitelsit_state.authority.key(); //// the whitelist owner 
        let mut whitelist_data = ctx.accounts.whitelist_data.load_mut()?.to_owned();
        let mut counter = whitelsit_state.counter as usize;

        //// the signer of this tx call must be the one
        //// who initialized the whitelist instruction handler
        //// or the server account must pay for gas fee
        //// because the burner shouldn't pay for the whitelist
        //// gas fee.
        // if signer != who_initialized_whitelist{
        //     return err!(ErrorCode::AddToWhitelistSignerIsNotTheInitializedAuthority);
        // }

        //// this is the `nft_stats` field which is the PDA
        //// account that can mutate instruction data on the chain
        let pda_account = *ctx.accounts.nft_stats.to_account_info().key;


        if counter > WhitelistData::MAX_SIZE{ //// make sure that we have enough space
            return err!(ErrorCode::NotEnoughSpace);
        }

        msg!("[+] current counter: {}", counter);
        let mut current_data = Vec::<Pubkey>::new();
        current_data.extend_from_slice(&whitelist_data.list[0..counter]); //// counter has the current size of the total PDAs, so we're filling the vector with the old PDAs on chain
        if current_data.contains(&pda_account){
            return err!(ErrorCode::PdaIsAlreadyAdded);
        }
        current_data.push(pda_account); //// at this stage the size of the vector might not be the MAX_SIZE since the `list` field of the `whitelist_data` might not be reached that size yet means that we have still enough storage to store PDAs  
        counter += 1; //// a new PDA added

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
            current_data.push(Pubkey::default()); //// filling the rest of the vector with default public key
        }
        msg!("[+] vector length on chain {:?}", current_data.len());

        //// converting the vector into the slice with the size of MAX_SIZE
        //// try_into() will convert the vector into a slice with a fixed size
        let updated_data_on_chain: [Pubkey; 5000] = match current_data.try_into(){ // TODO - need to change the 5000 since it's the total number of PDAs that must be inside the list
            Ok(data) => data,
            Err(e) => {
                msg!("[+] error in filling whitelist data on the chain {:?}", e);
                return err!(ErrorCode::FillingDataOnChainError);
            }
        };

        whitelist_data.list = updated_data_on_chain;
        whitelsit_state.counter = counter as u64;
        msg!("[+] current counter after adding one PDA: {}", counter);

        Ok(())

    }
    
    pub fn remove_from_whitelist(ctx: Context<RemoveFromWhitelistRequest>) -> Result<()>{

        let signer = ctx.accounts.authority.key();
        let whitelsit_state = &mut ctx.accounts.whitelist_state;
        let mut whitelist_data = ctx.accounts.whitelist_data.load_mut()?.to_owned();
        let mut counter = whitelsit_state.counter as usize;
        let who_initialized_whitelist = whitelsit_state.authority.key();

        //// the signer of this tx call must be the one
        //// who initialized the whitelist instruction handler
        if signer != who_initialized_whitelist{
            return err!(ErrorCode::AddToWhitelistSignerIsNotTheInitializedAuthority);
        }

        //// this is the `nft_stats` field which is the PDA
        //// account that can mutate instruction data of type `Nft`
        //// on the chain and with this we can make sure that 
        //// mutating on chain data with the correct input.
        let pda_account = *ctx.accounts.nft_stats.to_account_info().key;

        msg!("[+] current counter: {}", counter);
        let mut current_data = Vec::<Pubkey>::new();
        let pda_to_remove_index = if let Some(index) 
                                        = current_data
                                            .iter()
                                            .position(|p| *p == pda_account){
                            index
        } else{
            return err!(ErrorCode::PdaNotFoundToRemove);
        };

        current_data.remove(pda_to_remove_index);
        counter -= 1; //// a PDA is removed

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
            current_data.push(Pubkey::default()); //// filling the rest of the vector with default public key
        }
        msg!("[+] vector length on chain {:?}", current_data.len());

        //// converting the vector into the slice with the size of MAX_SIZE
        //// try_into() will convert the vector into a slice with a fixed size
        let updated_data_on_chain: [Pubkey; 5000] = match current_data.try_into(){ // TODO - need to change the 5000 since it's the total number of PDAs that must be inside the list
            Ok(data) => data,
            Err(e) => {
                msg!("[+] error in filling whitelist data on the chain {:?}", e);
                return err!(ErrorCode::FillingDataOnChainError);
            }
        };

        whitelist_data.list = updated_data_on_chain;
        whitelsit_state.counter = counter as u64;
        msg!("[+] current counter after removing PDA: {}", counter);
        

        Ok(())
        
    }

}



//// structs that are bounded to this proc macro attribute, are the instruction data
//// which are the generic of the account (`Account` type) that wants to mutate them on the chain and 
//// their fields can be accessible by calling the `program.account.<STRUCT_NAME>`
//// in frontend side which allows us to get current value of each field on chain,
//// here the owner of this struct is the PDA account which its owner must be
//// the program id so it can mutate the deserialized data on the chain although
//// the `#[account]` proc macro on top of the generic `T` or Nft in here will set 
//// the owner of the `Account` type that contains the generic `T` to the program id since 
//// the account must be the owner of the program in order to mutate data on the chain
//
//// `#[account]` proc macro attribute sets 
//// the owner of that data to the 
//// `declare_id` of the crate 
#[account] 
#[derive(Default, PartialEq)]
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
//// `AccountInfo` which can contains the serialized 
//// instruction data to be mutated on the chain by 
//// its owner or `Account` which is just an account 
//// info contains public key without the 
//// serialized instruction data.
#[derive(Accounts)] //// Accounts trait bounding will add the AnchorSerialize and AnchorDeserialize traits to the generic or the BurnRequest struct
// #[instruction(burn_tx_hash: String)] //// we can access the instruction's arguments here which are passed inside the `burn_request` instruction handler with the #[instruction(..)] attribute
pub struct BurnRequest<'info> { //// 'info lifetime in function signature is required to store utf8 bytes or &[u8] instruction data in the accounts
    #[account(mut)]
    pub user: Signer<'info>, //// a mutable and a signer account which means this transaction call will be signed by a specific holder or NFT owner and is writable to make changes on it
    //// bump is a utf8 bytes format so we can create it from a unique string also
    //// when init is used with seeds and bump, it will always search for the canonical bump
    //// this means that it can only be called once (because the 2nd time it's called 
    //// the PDA will already be initialized). 
    #[account( // https://www.anchor-lang.com/docs/pdas
        //// to use an account that owns a generic data 
        //// on chain like `Nft` data which is 
        //// owned by the `nft_stats` account, the 
        //// account which is of type `Account` must be 
        //// initialized first and limited to a space,
        //// the initialization process is a CPI call 
        //// to the solana runtime to set the owner of 
        //// the `nft_stats` to the program id
        //// since only the program must be able to 
        //// mutate its generic data on chain.
        //
        //// `init` will initialize a call using a cpi to the solana 
        //// to create the init account that its owner is the program
        //// itself which allows us to mutate its instruction data on chain 
        //// by deserializing it using borsh.
        //
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
        //
        //// we can't use `nft_stats.mint.key().as_ref()` since
        //// in here we're initializing the `nft_stats` in the
        //// first step and we don't have access to that field. 
        //
        //// the PDA can be built from the NFT owner and the NFT
        //// mint address since an owner might have burned multiple NFTs
        //// thus the tracking must be unique to add them to whitelist
        //// also since PDA is a public key only wallet address, we can 
        //// send to and withdraw from it and sign cpi with it inside the 
        //// contract since it has no private key and is off curve.
        init, //// --- init also requires space and payer constraints --- 
        space = 300, //// first 8 byte is the anchor discriminator and the rest is the size of the Nft struct which is Nft::MAX_SIZE or 256 bytes
        payer = user, //// the payer is the signer which must be the NFT owner, this constraint will be checked inside the `burn_request` method
        // seeds = [user.key().as_ref(), burn_tx_hash.as_bytes()], //// the following is the PDA account that can be created using the signer public key which is the nft owner and the nft burn tx hash to create the whitelist id; as_ref() converts the public key of each account into utf8 bytes  
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
    //// only the program will have control over it also PDA 
    //// doesn't need to be a signer since it has no private key.
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
    //
    //// `#[account()]` proc macro attribute is on top of 
    //// the `nft_stats` field thus the generic of this account, 
    //// the `Nft` structure must be bounded to the `#[account()]`
    //// proc macro attribute in order to be accessible inside 
    //// the frontend also `#[account()]` proc macro attribute 
    //// sets the owner of the generic to the program id.
    pub nft_stats: Account<'info, Nft>,
    //// more than one account of type `AccountInfo` 
    //// needs to be checked also with `AccountInfo` type we 
    //// can mutate nothing on chain since it has no passed 
    //// in generic instruction data and only it can be used 
    //// to make some changes on the account itself like 
    //// transferring lamports from it to another accounts 
    //// which must be mutable or use it to create the PDA like 
    //// the following account that will be used to create the 
    //// whitelist PDA.
    /// CHECK: This is not dangerous because we're using this account to create the PDA and check against the passed in NFT mint address from the frontend
    pub nft_mint: AccountInfo<'info>, //// this needs to be checked since it's unsafe because we're initializing the PDA account and any other account will be unsafe thus we have to check them as safe 
    /// CHECK: This is not dangerous because we're using this account to set the `nft_stats` field inside the `burn_request` method
    pub metadata: AccountInfo<'info>, 
    /// CHECK: This is not dangerous because we're using this account to set the `nft_stats` field inside the `burn_request` method
    pub token: AccountInfo<'info>, 
    /// CHECK: This is not dangerous because we're using this account to set the `nft_stats` field inside the `burn_request` method
    pub edition: AccountInfo<'info>, 
    /// CHECK: This is not dangerous because we're using this account to set the `nft_stats` field inside the `burn_request` method
    pub spl_token: AccountInfo<'info>, 
    /// CHECK: This is not dangerous because we're using this account to set the `nft_stats` field inside the `burn_request` method
    pub collection_metadata: Option<AccountInfo<'info>>, 
    /// CHECK: This is not dangerous because we're using this account to set the `nft_stats` field inside the `burn_request` method
    pub the_program_id: AccountInfo<'info>, 
    pub system_program: Program<'info, System>,
}


impl<'info> for BurnRequest<'info>{}

#[derive(Accounts)]
pub struct IntializeWhitelist<'info>{
    // anchor `#[account]` proc macro attribute constraint guide: https://docs.rs/anchor-lang/latest/anchor_lang/derive.Accounts.html
    #[account( //// can't use mut constraint since we're initializing this account
        //// to use an account that owns a generic data 
        //// on chain like `WhitelistState` data which is 
        //// owned by the `whitelist_state` account, the 
        //// account which is of type `Account` must be 
        //// initialized first and limited to a space,
        //// the initialization process is a CPI call 
        //// to the solana runtime to set the owner of 
        //// the `whitelist_state` to the program id
        //// since only the program must be able to 
        //// mutate its generic data on chain.
        init, //// initializing the whitelist_state account 
        payer = user, //// init requires payer (the signer of this tx call) and space constraint
        space = 50 //// total space required for this account on chain which is the size of its generic or `WhitelistState` struct or WhitelistState::MAX_SIZE
    )]
    //// `#[account()]` proc macro attribute is on top of the `whitelist_state` field thus the generic of this account, the `WhitelistState` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_state: Account<'info, WhitelistState>, //// this account is also the singer of the transaction call that means it must pay for the gas fee
    #[account(zero)] //// zero constraint is necessary for accounts that are larger than 10 Kibibyte because those accounts cannot be created via a CPI (which is what init would do)
    //// `#[account()]` proc macro attribute is on top of the `whitelist_data` field thus the generic of this account, the `WhitelistData` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_data: AccountLoader<'info, WhitelistData>, //// since `WhitelistData` is a zero copy data structure we must use `AccountLoader` for deserializing it
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
        //
        //// instead of creating signature from wallet addresses 
        //// or creating merkle root from them we can use PDA
        //// to add or remove them into or from the list.
        mut, 
        // seeds = [nft_stats.owner.key().as_ref(), burn_tx_hash.as_bytes()], //// the following is the PDA account that can be created using the nft owner and the nft burn tx hash to create the whitelist id
        seeds = [nft_stats.owner.key().as_ref(), nft_stats.mint.key().as_ref()], //// the following is the PDA account that can be created using the nft owner and the nft mint address to create the whitelist id
        bump = nft_stats.bump //// use the nft_stats bump itself which has been founded inside the frontend
    )]
    //// this is the account that can mutate the generic `Nft` 
    //// on the contract and its owner is the program id, 
    //// in a sence the PDA account must only be owned by the 
    //// program or account.owner == program_id, since this 
    //// is not initialize account we don't need to add 
    //// space and payer constraint.
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
        mut, 
    )]
    //// `#[account()]` proc macro attribute is on top of the `whitelist_data` field thus the generic of this account, the `WhitelistData` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_data: AccountLoader<'info, WhitelistData>, //// `AccountLoader` will be used to deserialize zero copy types 
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
//// `declare_id` of the crate
//
//// zero copy technique: https://rkyv.org/zero-copy-deserialization.html#zero-copy-deserialization 
//// since we're using array to store PDAs we must 
//// use the zero copy technique which is directly
//// referencing bytes of the serialized type instead of
//// copying the serialized type into the buffer, for ex
//// we can borrow the String of a JSON into &str instead of
//// loading those chars into a String also the lifetime of 
//// the &str depends on the String buffer itself and because
//// there is no copying operation this is called zero copy 
//// deserialization technique, also zero copy needs that 
//// the Copy trait implemented for the type thus dynamic
//// size types like Vec can't be bounded to zero copy.
#[account(zero_copy)] 
pub struct WhitelistData{ //// slice types or borrowed form of dynamic sized types require `zero_copy` feature on `#[account()]` proc macro
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
    //// by (32 * 5000).
    //
    //// note that we must initialize the list before calling `add_to_whitelist`
    //// instruction handler since it's an array that must be filled with some 
    //// default public key then in `add_to_whitelist` handler we can replace it 
    //// with the passed PDAs.
    //
    //// dynamic size can be in their borrowed form like &str or &[u8],
    //// since str and [u8] don't have fixed size at compile time 
    //// we must use them behind a reference or the borrowed form of their dynamic size
    //// about the &[u8] we can also use its fixed size like [0u8; 32] also
    //// we have to pass types into other scopes and threads by reference or 
    //// in their borrowed form and we have to fill the remaining bytes if 
    //// we're using the array slices with a fixed size since their size must 
    //// be specified at compile time also all the allocated size for 
    //// them must be filled at runtime.
    pub list: [Pubkey; 5000], //// list of all PDAs that shows an owner has burnt his/her NFT; TODO - need to change the 5000 since it's the total number of PDAs that must be inside the list
}


impl WhitelistData{
    // https://solana.stackexchange.com/questions/3816/thread-main-panicked-at-called-resultunwrap-on-an-err-value-parsein
    //// note that we can't use MAX_SIZE variablt 
    //// inside the array we must use a number means 
    //// we can't have [Pubkey; MAX_SIZE] since it'll
    //// face us an error like: thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }'
    pub const MAX_SIZE: usize = 5000; // TODO - need to change the 5000 since it's the total number of PDAs that must be inside the list
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
    //// `AccountLoader` will bound the `Account` into ZeroCopy trait and return a RefMut or the borrowed form of the deserialized instruction data  
    //// `#[account()]` proc macro attribute is on top of the `whitelist_data` field thus the generic of this account, the `WhitelistData` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id
    pub whitelist_data: AccountLoader<'info, WhitelistData>, 
    #[account(mut, has_one = authority)]
    pub whitelist_state: Account<'info, WhitelistState>, //// `#[account()]` proc macro attribute is on top of the `whitelist_state` field thus the generic of this account, the `WhitelistState` structure must be bounded to the `#[account()]` proc macro attribute in order to be accessible inside the frontend also the `#[account()]` proc macro attribute sets the owner of the generic to the program id  
    #[account( 
        mut, 
        // seeds = [nft_stats.owner.key().as_ref(), burn_tx_hash.as_bytes()], //// the following is the PDA account that can be created using the nft owner and the nft burn tx hash to create the whitelist id
        seeds = [nft_stats.owner.key().as_ref(), nft_stats.mint.key().as_ref()], //// the following is the PDA account that can be created using the nft owner and the nft mint address to create the whitelist id
        bump = nft_stats.bump //// use the nft_stats bump itself which has been founded inside the frontend
    )]
    pub nft_stats: Account<'info, Nft>, 
}

#[error_code]
pub enum ErrorCode {
    #[msg("Signer Is Not The Nft Owner")]
    RestrictionError,
    #[msg("Invalid Nft Burn Instruction In Metaplex Candy Machine")]
    InvalidBurnInstruction,
    #[msg("Signer Is Not The Whitelist Authority")]
    AddToWhitelistSignerIsNotTheInitializedAuthority,
    #[msg("Not Enough Space")]
    NotEnoughSpace,
    #[msg("Fillin Data On Chain Error")]
    FillingDataOnChainError,
    #[msg("PDA Not Found To Remove")]
    PdaNotFoundToRemove,
    #[msg("Different Nft Mint Address")]
    DifferentNftMintAddress,
    #[msg("Access Denied, Passed In Program Id Is Invalid")]
    AccessDeniedDueToInvalidProgramId,
    #[msg("Runtime Error")]
    RuntimeError,
    #[msg("PDA Is Already Added")]
    PdaIsAlreadyAdded
}


#[event]
pub struct NftBurnEvent{
    pub owner: Pubkey,
    pub mint_address: Pubkey,
}
