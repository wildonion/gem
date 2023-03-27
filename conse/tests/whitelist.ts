import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Whitelist } from "../target/types/whitelist";
import { PublicKey } from '@solana/web3.js';

describe("conse whitelist", () => {
  

  //// `provider.wallet.publickey` is signer by default 
  //// since we're using it to pay for transaction fees
  //// signer must pay for the transaction fees 
  //// and we can make an account as the signer by putting 
  //// it inside the signers([]) array
  //
  //// use a real provider or connection like testnet or devnet
  //// Configure the client to use the local cluster. 
  const provider = anchor.AnchorProvider.env(); //// the authority who has deployed this program is: 8SzHrPVkDf5xhmjyUJ7W8vDaxhTiGF9XBT9XX2PtiwYF
  anchor.setProvider(anchor.AnchorProvider.env()); // Configure the client to use the local cluster.
  
    
    const wl_signer = anchor.web3.Keypair.generate(); // TODO - the signer of the whitelist account initialization
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


        //---------------------------
        // charging wl signer account
        //---------------------------
        // since wl must be the signer
        // thus there must be enough lamports in 
        // his/her account to pay the gass fee
        await provider.connection.confirmTransaction ({
          blockhash: latestBlockHashforUserOne.blockhash,
          lastValidBlockHeight: latestBlockHashforUserOne.lastValidBlockHeight,
          signature: await provider.connection.requestAirdrop(wl_signer.publicKey, lamport_amount)
      });
      console.log("wl_signer balance >>>> ", await provider.connection.getBalance(wl_signer.publicKey));



      



        // --------------
        // Init Whitelist
        // --------------
        // https://solana.stackexchange.com/questions/5138/tryingtoinitpayerasprogramaccount-error-number-4101-you-cannot-should-not-ini
        // https://stackoverflow.com/questions/70406575/what-is-signature-verification-failed-in-solana
        //// we have to pass the signer of the call 
        //// or the payer for account creation and 
        //// the account that we're creating into 
        //// the signers array. here we're initializing
        //// the server account to be the whitelist authority
        //// that can mutate the state and the data on chain
        //// also the payer of this call who must also 
        //// sign the tx call for paying the account creation
        //// and gas fee is wl_signer also we passed the server
        //// account too because every account needs to sign its own creation
        //// although if we want to use the default anchor wallet
        //// there is no need to pass the wl_signer as the first param
        //// since by default provider.wallet.publickey will sign every tx call
        //// but we didn't use that and pass a another account instead.
        //
        //// to init an account other than PDAs cause PDAs has no
        //// private key thus they can't sign, there must be two 
        //// signers the first is the one who must pay for the account 
        //// creation and the second is the account itself that we're 
        //// creating it which must sign with its private key for its 
        //// own creation.
        await program.methods.initializeWhitelist(server.publicKey).accounts({ //// initializing the whitelist state and whitelist data accounts    
            user: wl_signer.publicKey, //// instead of using provider.wallet.publickey this account pays for the server whitelist account creation 
            whitelistData: server.publicKey,
        }).signers([wl_signer, server]).rpc(); //// also server must be the signer because every account needs to sign its own creation





        // ------------
        // Burn Request
        // ------------
        let nft1 = {
          owner: nft_owner.publicKey,
          mint: nft_mint.publicKey,
          metadata: metadata.publicKey,
          token: token.publicKey,
          edition: edition.publicKey,
          splToken: spl_token.publicKey,
          programId: program.programId,
          collectionMetadata: collection_metadata.publicKey
        }
        let nft2 = {
          owner: nft_owner.publicKey,
          mint: nft_mint.publicKey,
          metadata: metadata.publicKey,
          token: token.publicKey,
          edition: edition.publicKey,
          splToken: spl_token.publicKey,
          programId: program.programId,
          collectionMetadata: collection_metadata.publicKey
        }
          await program.methods.burnRequest([nft1, nft2]).accounts({
                user: nft_owner.publicKey, 
            }).signers([nft_owner]).rpc(); //// signer of this call who must pay for the transaction fee is the NFT owner
          



        //// NOTE - invalid account discriminator means 
        ////        that the account we're trying to deserialize it
        ////        might not be existed thus we have to initialize it first
        // ----------------
        // Add to Whitelist
        // ----------------
        // call this after successful burn request
        let pda1 = {
          owner: nft_owner.publicKey,
          nfts: [nft1.mint, nft2.mint]
        }
        let pda2 = {
          owner: nft_owner.publicKey,
          nfts: [nft1.mint, nft2.mint]
        }

        await program.methods.addToWhitelist([pda1, pda2]).accounts({
            authority: server.publicKey, // the signer must the initialized account by the wl_signer which is the server
            whitelistData: server.publicKey,
        }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee is the server account itself which has initiialized and created by the wl_signer account

        const data = program.account.whitelistData.fetch(server.publicKey); 
        console.log("----------- ----------- ----------- ")
        console.log("----------- whitelist data -------- ", (await data).list);
        console.log("----------- whitelist counter -------- ", (await data).counter);
        console.log("----------- whitelist authority -------- ", (await data).authority);
        console.log("----------- ----------- ----------- ")

    });
});
