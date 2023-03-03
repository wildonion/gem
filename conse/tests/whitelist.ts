import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ConseGemWhitelist } from "../target/types/conse_gem_whitelist";
import { PublicKey } from '@solana/web3.js';

describe("whitelist", () => {
  

    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
  
    const server = anchor.web3.Keypair.generate(); // TODO - a secure account, preferred to be a none NFT owner
    const nft_owner = anchor.web3.Keypair.generate(); // TODO 
    const nft_mint = anchor.web3.Keypair.generate(); // TODO 
    const metadata = anchor.web3.Keypair.generate(); // TODO 
    const token = anchor.web3.Keypair.generate(); // TODO 
    const edition = anchor.web3.Keypair.generate(); // TODO 
    const spl_token = anchor.web3.Keypair.generate(); // TODO 
    const collection_metadata = anchor.web3.Keypair.generate(); // TODO 

    // only the program itself can mutate passed in instruction data to a instruction handler on chain
    const program = anchor.workspace.ConseGemReservation as Program<ConseGemWhitelist>;
    const provider = anchor.AnchorProvider.env();

    

    it("Pda created!", async () => {
        

        // -------------
        // Creating PDA
        // -------------

        // build PDA from NFT owner and the NFT mint address
        // since it might be multiple burn for a user thus
        // these params will be unique inside the whitelist. 
        const [NftStatsPDA, bump] = PublicKey
        .findProgramAddressSync(
            [nft_owner.publicKey.toBuffer(), nft_mint.publicKey.toBuffer()],
            program.programId
          )





        // --------------
        // Init Whitelist
        // --------------

        // a secure account must init a whitelist like the server account 
        // and also the one who init the server is the authority of 
        // the the whitelist state and data and only she/he can add 
        // PDA to whitelist or call that method
        await program.methods.initializeWhitelist().accounts({
            user: server.publicKey, 
            whitelistState: server.publicKey, 
            whitelistData: server.publicKey,
        }).rpc();


        // deserializing the `whitelist_state` account
        // which is owned by the server.
        let deserialized_whitelist_state_account = await program.account.whitelistState.fetch(server.publicKey);
        console.log("deserialized_whitelist_state_account: >>>>>> ", deserialized_whitelist_state_account);




        // ------------
        // Burn Request
        // ------------
          await program.methods.burnRequest(bump).accounts({
                user: nft_owner.publicKey, 
                nftStats: NftStatsPDA, 
                nftMint: nft_mint.publicKey,
                metadata: metadata.publicKey,
                token: token.publicKey,
                edition: edition.publicKey,
                splToken: spl_token.publicKey,
                theProgramId: program.programId,
                collectionMetadata: collection_metadata.publicKey
            }).rpc();

          // deserializing the PDA account
          // since the `Nft` struct inside the contract
          // is bounded to `#[account]` proc macro attribute
          // which is owned by the PDA and only the PDA can mutate
          // its data on the chain.
          let deserialized_nft_stats_account = await program.account.nft.fetch(NftStatsPDA);
        console.log("deserialized_nft_stats_account: >>>>>> ", deserialized_nft_stats_account);
          





        // ----------------
        // Add to Whitelist
        // ----------------
        // call this after successful burn request
        await program.methods.addToWhitelist().accounts({
            authority: server.publicKey, // the signer must the one who initialized the whitelist to add a PDA into the chain
            nftStats: NftStatsPDA, // this will be added to the whitelist by the server authority
            whitelistState: server.publicKey, 
            whitelistData: server.publicKey,
        }).rpc(); 

        let deserialized_whitelist_data_account_after_adding = await program.account.whitelistData.fetch(server.publicKey);
        console.log("deserialized_whitelist_data_account: >>>>>> ", deserialized_whitelist_data_account_after_adding);





        // ----------------
        // Remove Whitelist
        // ---------------- 
        await program.methods.removeFromWhitelist().accounts({
            authority: server.publicKey,  // the signer must the one who initialized the whitelist to remove a PDA from the chain
            nftStats: NftStatsPDA, // this will be removed from the whitelist by the server authority
            whitelistState: server.publicKey, 
            whitelistData: server.publicKey,
        }).rpc(); 

        let deserialized_whitelist_data_account_after_removing = await program.account.whitelistData.fetch(server.publicKey);
        console.log("deserialized_whitelist_data_account_after_removing: >>>>>> ", deserialized_whitelist_data_account_after_removing);


    
    });
});
