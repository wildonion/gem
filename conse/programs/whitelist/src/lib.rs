



use anchor_lang::prelude::*;
use mpl_token_metadata::instruction::burn_nft;


//// wallet address in solana are based on 
//// base58 encoding thus we can't simply 
//// build the public key from them 
declare_id!("2YQmwuktcWmmhXXAzjizxzie3QWEkZC8HQ4ZnRtrKF7p"); //// this is the program public key of the the program wallet info which can be found in `target/deploy/whitelist-keypair.json` 



#[program] //// the program entrypoint which must be defined only once
pub mod whitelist {

    
    // https://github.com/samuelvanderwaal/solana-whitelist/tree/main/programs/whitelist


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
        //// a shared reference can be in used by other scopes and threads thus moving out of 
        //// a shared referenced type requires one of the dereferencing methods which is either 
        //// copying (the Copy trait must be implemented), borrowing it using & or cloning 
        //// (Clone trait must be implemented) it or dereference it by using `*` otherwise 
        //// we can' move out of it in our case `whitelist_state` which is of type Account, can't be moved 
        //// since `Account` type doesn't implement Copy trait we have to either borrow it or clone it. 
        //
        //// based on above notes we should borrow the ctx.accounts mutably if the type was either Account,
        //// or AccountInfo since we might want to mutate its fields later and these accounts don't implement
        //// the Copy trait thus we can't dereference them by * cause they might be being in used by other scopes 
        //// and threads (they are shared references) we have to borrow them or clone them.
        //
        //// we can't move the type between scopes and threads if there is a pointer or shared reference of 
        //// that type exists which is being in used by other scopes and threads and due to the fact that 
        //// rust doesn't support gc thus by moving it its lifetime will be dropped hence its pointer will 
        //// be point to no where or a location inside the stack or heap that doesn't exist!

        let whitelist_data = &mut ctx.accounts.whitelist_data; //// a mutable reference to the whitelist data account, since Account is a mutable shared reference which doesn't implement Copy trait we can't move out of it thus we have to either clone it or borrow it 
        whitelist_data.list = vec![]; //// creating empty vector on chain
        //// the authority is the server account which must 
        //// be passed from the frontend call, also this account
        //// has signed its own creation thus we can use it 
        //// to sign the add to whitelist tx call later
        //// since we're setting the authority of the whitelist_data 
        //// on chain to the passed in authority which is the
        //// server which is the owner of the whitelist_data 
        whitelist_data.authority = authority; 
        whitelist_data.counter = 0;


        Ok(())

    }

    //// this instruction handler must be called from the server or 
    //// where the whitelist state authority wallet info exists.
    //// in centralized servers the security check of the caller 
    //// can be done using a JWT or a dev token.
    pub fn add_to_whitelist(ctx: Context<AddToWhitelistRequest>, addresses: Vec<Pda>) -> Result<()>{

        let signer = ctx.accounts.authority.key();
        //// a mutable reference to the whitelist data account, we can't move the whitelist_data while we have 
        //// pointer of that since by moving it all of its pointer might be converted into a dangling ones which rust
        //// doesn't allow this from the first, since Account is a mutable shared reference which doesn't implement 
        //// Copy trait we can't move out of it thus we have to either clone it or borrow it, this is for dynamic 
        //// data sized like vector and string in which we can't move them without cloning them or borrowing, for 
        //// stack data types like u8 we can have a pointer of them and simply move them between threads
        //// and scopes without losing their ownerships. 
        //
        //// whitelist_data is of type vector which doesn't implement Copy trait (since heap data sized are can't 
        //// be simply copied and in order to have them in later scopes we have to either clone them which is expensive 
        //// or borrow them) means that by moving it into new scopes we'll lose it's ownership and will be dropped 
        //// (rust doesn't have gc) thus if a shared pointer of that exists we can't move it since by moving it its 
        //// lifetime will be dropped and the pointer will point to no where which remains a dangling pointer which 
        //// rust doesn't allow use to do this from the first step, the solution to this, for dynamic data sized like 
        //// vector is by either passing the already pointer or cloned version of the vector into the new scopes, but 
        //// for stack data sized we can have them behind a pointer and moving them between scopes and threads because 
        //// by moving them rust will actually copying their bits into a new type inside the new scope.
        let whitelist_data = &mut ctx.accounts.whitelist_data; //// since Copy is not implement for Account type we've borrowed the whitelist_data mutably
        let who_initialized_whitelist = whitelist_data.authority.key(); //// the whitelist owner 
        let mut counter = whitelist_data.counter as usize;
        //// we can't move heap data size into new scopes when there 
        //// is a shared reference or pointer of them is exists (we've
        //// borrowed it mutably earlier) but it's ok for the stack 
        //// data types, the solution is either clone it or borrowing it.  
        let mut whitelist_data_list = whitelist_data.list.clone(); 
        let length = addresses.len();

        //// the signer of this tx call must be the one
        //// who initialized the whitelist instruction handler
        //// which must pay for gas fee because the NFT burner 
        //// shouldn't pay for the whitelist gas fee.
        if signer != who_initialized_whitelist{
            return err!(ErrorCode::WhitelistOwnerRestriction);
        }

        if counter > WhitelistData::MAX_SIZE{ //// make sure that we have enough space
            return err!(ErrorCode::NotEnoughSpace);
        }
        
        whitelist_data_list.extend_from_slice(&addresses[0..length]); //// extending the on chain data with the passed in ones
        counter += length;

        whitelist_data.list = whitelist_data_list; //// since whitelist_data is behind a reference we have to dereference it
        whitelist_data.counter = counter as u64;
        msg!("----------- ----------- ----------- ");
        msg!("[+] whitelist data on chain {:?}", whitelist_data.list);
        msg!("----------- ----------- ----------- ");
        
        Ok(()) 
        
    }


}


//// `#[account]` proc macro attribute will implement the
//// AccountSerialize and AccountDeserialize traits for the generic
//// thus the implementations of AccountSerialize and AccountDeserialize 
//// do the discriminator check on the first 8 bytes of the account name 
//// and Borsh ser/deser of the rest of the account data means
//// that the account serialization is: 
//// (sha256(account_name).get_first_byte() + account data).serialize()
//// account name is a unique identifier that can be used to detect 
//// which account is being executed on runtime in parallel.
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
        //
        //// every instruction on solana is 
        //// a transaction which contains
        //// serialized data, accounts and 
        //// the program id that executed
        //// this instruction.
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
    pub system_program: Program<'info, System>, //// this refers to this program itself and it can be another program also it's carrying 0-bytes of account data and can be used for CPI calls (simply put the contract name instead of System)
}


impl<'info> BurnRequest<'info>{}

#[derive(Accounts)]
pub struct IntializeWhitelist<'info>{
    #[account( //// can't use mut constraint here since we're initializing this account
        //// data store on solana accounts and if it's not exists on the 
        //// runtime `init` will initialize the account via CPI call to the 
        //// runtime and sets its owner to the program id by default since
        //// only the program must be able to mutate its generic instruction data 
        //// on chain by deserializing it using borsh hence to use an account 
        //// that owns a generic data on chain like `WhitelistData` data which 
        //// is owned by the `whitelist_state` account, the account which is of 
        //// type `Account` must be initialized first by a payer and limited to 
        //// a space then we can use it in the next instruction handler.
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
        //
        //// since init constraint needs a payer who must pay
        //// for the account creation by signing the tx thus 
        //// the payer is the signer of this call and must be 
        //// the one who pays for the whitelist server account
        //// creation also in frontend we have to pass the payer
        //// account along with the server account itself inside 
        //// the signers[] array if we don't want to use the 
        //// provider.wallet.publickey which signs every tx call 
        //// by default, although the reason that we must pass 
        //// the server as the signer too is because every account 
        //// needs to sign its own creation.
        //
        //// to init an account other than PDAs (cause PDAs has no
        //// private key thus they can't sign), there must be two 
        //// signers the first is the one who must pay for the account 
        //// creation and the second is the account itself that we're 
        //// creating it which must sign with its private key for its 
        //// own creation. 
        //
        //// zero constraint or `#[account(zero)]` is necessary for 
        //// accounts that are larger than 10 Kibibyte also will deserialize 
        //// their data using a zero copy technique because those accounts 
        //// cannot be created via a CPI (which is what init would do).
        //
        //// we can use #[account(mut)] only if the account has already 
        //// been initialized otherwise we should first init it then 
        //// try to mutate it. 
        //
        //// the `WhitelistData` structure must be bounded to the `#[account]` 
        //// proc macro attribute in order to be accessible inside the frontend 
        //// also the `#[account()]` proc macro attribute sets the owner 
        //// of the generic to the program id.
        //
        //// account must starts with certain 8 bytes tag or 
        //// (hash of the account name) also we must include 
        //// this extra space inside the space constraint 
        //// to calculate the total space required for the account.
        // ------------------------------------------------------
        // --- init also requires space and payer constraints ---
        // ------------------------------------------------------
        init, //// initializing the whitelist_state account via CPI call to the solana runtime to grant the access of instruction data on chain for later mutation, init by default sets the owner of this account to the program id so the program can mutate data on chain later
        payer = user, //// init requires payer (the signer of this tx call) who must pay for the gas fee and account creation
        space = 300 //// total space required for this account on chain with extra 8 bytes discriminator tag, 300 bytes is good for all data i think :/
    )]
    pub whitelist_data: Account<'info, WhitelistData>, //// this account is also the singer of the transaction call that means it must pay for the gas fee
    #[account(mut)]
    pub user: Signer<'info>, //// signer or payer of this tx call which must be mutable since it's the payer for initializing the `whitelist_state` account that must pay for the call which leads to decreasing lamports from his/her account
    pub system_program: Program<'info, System>, //// when we use `init` system program account info must be exists also system program is carrying 0-bytes of account data and can be used for CPI calls (simply put the contract name instead of System)

}

#[derive(Accounts)]
// #[instruction(burn_tx_hash: String)] //// we can access the instruction's arguments here which are passed inside the `add_to_whitelist` instruction handler with the #[instruction(..)] attribute
pub struct AddToWhitelistRequest<'info>{
    //// if we want to take money from someone, 
    //// we should make them sign as well as mark 
    //// their account as mutable.
    #[account(mut)]
    pub authority: Signer<'info>, //// the transaction signer account which must be mutable and is the one who pays for the gas fee and sing this call
    #[account(
        //// we need to define this account mutable or 
        //// writable since we want to add PDAs to it in runtime
        //
        //// to mutate the `whitelist_data` account must be initialized first which 
        //// this can be done inside the `initialize_whitelist` handler
        //// in its generic or `IntializeWhitelist` struct by putting 
        //// init, payer and space constraint on top of it.
        mut, 
        //// `has_one` constraint will check 
        //// that whitelist_data.owner == authority.key()
        //// so the whitelist_data must be initialized first
        has_one = authority
    )]
    pub whitelist_data: Account<'info, WhitelistData>, //// `AccountLoader` will be used to deserialize zero copy types 
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
pub struct WhitelistData{
    // https://solana.stackexchange.com/questions/2339/account-size-calculation-when-using-vectors
    //// heap data size like string and vector are always 24 bytes
    //// which will be stored on the stack: 8 bytes for capaciy, 8 bytes for length
    //// and the last one is the size of their pointer which points to a heap-allocated buffer
    //// but borsh actually serializing a slice of the vector thus the size of the following 
    //// list will be 4 + (32 * N) which N referes to the number of elements and 32 is the
    //// size of public key and 4 is the size of one public key since the public key is of
    //// type u32 which 4 bytes long.
    //
    //// slice form of dynamic sized types like strings, traits and vectors 
    //// must be behind a pointer since their size are not know at compile 
    //// time and will be specified at runtime either they are in binary, 
    //// stack or heap itself thus we have to take a pointer to them to store 
    //// the pointer in the stack to access their heap location using that 
    //// pointer to reach the data itself because pointers store the address
    //// of the data either in heap or stack. 
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
    pub authority: Pubkey, //// the owner of the whitelist state
    pub counter: u64, //// total number of PDAs inside the whitelist
}


impl WhitelistData{
    pub const MAX_SIZE: usize = 2000;
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
