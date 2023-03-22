import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Whitelist } from "../target/types/whitelist";
import { PublicKey } from '@solana/web3.js';

describe("conse whitelist", () => {
  

    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    //// `provider.wallet.publickey` is signer by default 
    //// since we're using it to pay for transaction fees
    //// signer must pay for the transaction fees 
    //// and we can make an account as the signer by putting 
    //// it inside the signers([]) array
    //
    //// use a real provider or connection like testnet or devnet
    //// Configure the client to use the local cluster. 
    const provider = anchor.AnchorProvider.env(); //// the authority who has deployed this program is: 8SzHrPVkDf5xhmjyUJ7W8vDaxhTiGF9XBT9XX2PtiwYF

    
    const server = anchor.web3.Keypair.generate(); // TODO - a secure account, preferred to be a none NFT owner
    const nft_owner = anchor.web3.Keypair.generate(); // TODO 
    const nft_mint = anchor.web3.Keypair.generate(); // TODO 
    const metadata = anchor.web3.Keypair.generate(); // TODO 
    const token = anchor.web3.Keypair.generate(); // TODO 
    const edition = anchor.web3.Keypair.generate(); // TODO 
    const spl_token = anchor.web3.Keypair.generate(); // TODO 
    const collection_metadata = anchor.web3.Keypair.generate(); // TODO 
    
    
    // https://solana.stackexchange.com/questions/2057/what-is-the-relation-between-signers-wallets-in-testing?rq=1
    // only the program itself can mutate passed in 
    // instruction data to a instruction handler on chain
    const program = anchor.workspace.Whitelist as Program<Whitelist>;
    
    
    
    it("Tested!", async () => {
        
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
        let nft1 = {
          owner: nft_mint.publicKey,
          mint: nft_mint.publicKey,
          metadata: metadata.publicKey,
          token: token.publicKey,
          edition: edition.publicKey,
          spl_token: spl_token.publicKey,
          program_id: program.programId,
          collection_metadata: collection_metadata.publicKey
        }
        let nft2 = {
          owner: nft_mint.publicKey,
          mint: nft_mint.publicKey,
          metadata: metadata.publicKey,
          token: token.publicKey,
          edition: edition.publicKey,
          spl_token: spl_token.publicKey,
          program_id: program.programId,
          collection_metadata: collection_metadata.publicKey
        }
          await program.methods.burnRequest([nft1, nft2]).accounts({
                user: nft_owner.publicKey, 
            }).signers([nft_owner]).rpc(); //// signer of this call who must pay for the transaction fee is the NFT owner
          





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




    
    });
});
