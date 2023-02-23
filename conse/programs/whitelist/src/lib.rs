

use anchor_lang::prelude::*;

declare_id!("6oRp5W29ohs29iGqyn5EmYw2PQ8fcYZnCPr5HCdKwkp9"); //// this is the program public key of the the program wallet info which can be found in `target/deploy/whitelist-keypair.json` 

#[program]
pub mod whitelist {


    use super::*;

    pub fn burn(ctx: Context<NftInfo>) -> Result<()>{

        // step1) get nft info from Metaplex Candy Machine using cpi calls
        // step2) check the candy machine nft info with the passed in nft 
        // step3) if the nft.owner == caller of this method
        //          we can burn to zero address and put the user with his/her nft into the whitelist
        //        else
        //          return error!(NftOwnerError);
    

        Ok(())
        
    }
    
    pub fn add_to_whitelist(ctx: Context<NftBurnInfo>) -> Result<()>{
    
        Ok(())
    
    }

}




#[derive(Accounts)]
pub struct NftInfo {}

#[derive(Accounts)]
pub struct NftBurnInfo{}