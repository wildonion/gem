import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Whitelist } from "../target/types/whitelist";
import { PublicKey } from '@solana/web3.js';

describe("conse whitelist", () => {
  

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
    // only the program itself can mutate passed in 
    // instruction data to a instruction handler on chain
    const program = anchor.workspace.Whitelist as Program<Whitelist>;
    // https://solana.stackexchange.com/questions/2057/what-is-the-relation-between-signers-wallets-in-testing?rq=1
    //// server.publicKey is the one who
    //// has deployed the program thus is the authority 
    //// of the program.
    //
    //// `provider.wallet.publickey` is signer by default 
    //// since we're using it to pay for transaction fees
    //// signer must pay for the transaction fees 
    //// and we can make an account as the signer by putting 
    //// it inside the signers([]) array
    const provider = anchor.AnchorProvider.env(); //// the authority who has deployed this program is: F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ

    

    
    
    it("Pda created!", async () => {
        
        const latestBlockHashforUserOne = await provider.connection.getLatestBlockhash();
        const lamport_amount = 10_000_000_000;
    



        //---------------------------
        // charging NFT owner account
        //---------------------------
        // since NFT owner must be the signer
        // thus there must be enough lamports in 
        // his/her account to pay the gass fee
        await provider.connection.confirmTransaction ({
            blockhash: latestBlockHashforUserOne.blockhash,
            lastValidBlockHeight: latestBlockHashforUserOne.lastValidBlockHeight,
            signature: await provider.connection.requestAirdrop(nft_owner.publicKey, lamport_amount)
        });
        console.log("NFT owner balance >>>> ", await provider.connection.getBalance(nft_owner.publicKey));



        // -------------
        // Creating PDA
        // -------------

        // build PDA from NFT owner and the NFT mint address
        // since it might be multiple burn for a user thus
        // these params will be unique inside the whitelist. 
        //
        //// contract doesn't return NFT burn tx hash
        //// thus we have to create the PDA based on the 
        //// NFT owner and NFT mint address
        let burn_tx_hash = "0000000000" ///////////////// TODO - this is the NFT burn tx hash
        const [NftStatsPDA, bump] = PublicKey
        .findProgramAddressSync(
            // [nft_owner.publicKey.toBuffer(), Buffer.from(burn_tx_hash, "utf-8")],
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
        await program.methods.initializeWhitelist(server.publicKey).accounts({ //// initializing the whitelist state and whitelist data accounts    
            user: server.publicKey, 
            whitelistState: server.publicKey, 
            whitelistData: server.publicKey,
        }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee is the server


        // deserializing the `whitelist_state` account
        // which contains the instruction data on chain 
        // and is owned by the server which are accessible
        // in here by using `.` notation on `deserialized_whitelist_state_account`
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
            }).signers([nft_owner]).rpc(); //// signer of this call who must pay for the transaction fee is the NFT owner

          // deserializing the PDA account
          // since the `Nft` struct inside the contract
          // is bounded to `#[account]` proc macro attribute
          // which is owned by the PDA and only the PDA can mutate
          // its data on the chain which are accessible
            // in here by using `.` notation on `deserialized_nft_stats_account`
          let deserialized_nft_stats_account = await program.account.nft.fetch(NftStatsPDA);
            console.log("deserialized_nft_stats_account: >>>>>> ", deserialized_nft_stats_account);
          





        // ----------------
        // Add to Whitelist
        // ----------------
        // call this after successful burn request
        // await program.methods.addToWhitelist().accounts({
        //     authority: server.publicKey, // the signer must the one who initialized the whitelist to add a PDA into the chain
        //     nftStats: NftStatsPDA, // this will be added to the whitelist by the server authority
        //     whitelistState: server.publicKey, 
        //     whitelistData: server.publicKey,
        // }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee is the server

        // let deserialized_whitelist_data_account_after_adding = await program.account.whitelistData.fetch(server.publicKey);
        // console.log("deserialized_whitelist_data_account: >>>>>> ", deserialized_whitelist_data_account_after_adding);





        // // ----------------
        // // Remove Whitelist
        // // ---------------- 
        // await program.methods.removeFromWhitelist().accounts({
        //     authority: server.publicKey,  // the signer must the one who initialized the whitelist to remove a PDA from the chain
        //     nftStats: NftStatsPDA, // this will be removed from the whitelist by the server authority
        //     whitelistState: server.publicKey, 
        //     whitelistData: server.publicKey,
        // }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee is the server

        // let deserialized_whitelist_data_account_after_removing = await program.account.whitelistData.fetch(server.publicKey);
        // console.log("deserialized_whitelist_data_account_after_removing: >>>>>> ", deserialized_whitelist_data_account_after_removing);


    
    });
});
