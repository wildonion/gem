import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { Ticket } from "../target/types/ticket";
import { assert, expect } from "chai";

describe("nds-transaction", () => {

  // TODO - use a real provider or connection like testnet or devnet
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  



  const player = anchor.web3.Keypair.generate(); // TODO - wallet handler
  const server = anchor.web3.Keypair.generate(); // TODO - server keys
  const revenue_share_wallet = anchor.web3.Keypair.generate(); // TODO - staking pool account
  


  const lamport_amount = 10_000_000_000;
  const lamport_to_send = 5_000_000_000;
  



  // https://solana.stackexchange.com/questions/2057/what-is-the-relation-between-signers-wallets-in-testing?rq=1
  const program = anchor.workspace.Ticket as Program<Ticket>;
  const provider = anchor.AnchorProvider.env(); 

  
  
  it("Pda created!", async () => {
  // find pda account
  const [gameStatePDA, bump] = PublicKey
  .findProgramAddressSync(
      [server.publicKey.toBuffer(), player.publicKey.toBuffer()],
      program.programId
    )



    ///////////////////////////////
    /////// STEP 0
    ///////////////////////////////

    //----------------------------
    // player one charging account
    //----------------------------
    const latestBlockHashforUserOne = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction ({
      blockhash: latestBlockHashforUserOne.blockhash,
      lastValidBlockHeight: latestBlockHashforUserOne.lastValidBlockHeight,
      signature: await provider.connection.requestAirdrop(player.publicKey, lamport_amount)
    });
    console.log("player 1 balance: ", await provider.connection.getBalance(player.publicKey));




    ///////////////////////////////
    /////// STEP 1
    ///////////////////////////////

    //----------------------------
    // sending from player to PDA
    //----------------------------
    let tx_data = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
      fromPubkey: player.publicKey,
      toPubkey: gameStatePDA,
      lamports: lamport_to_send,    
    }));
    await anchor.web3.sendAndConfirmTransaction(provider.connection, tx_data, [player]);
    



    ///////////////////////////////
    /////// STEP 2
    ///////////////////////////////

    //----------------------------
    // PDA and player balance
    //----------------------------
    let balance_player = await provider.connection.getBalance(player.publicKey);
    let balance_pda_account = await provider.connection.getBalance(gameStatePDA);
    console.log("sending sol from player to PDA");
    console.log(">>>> player balance: ", balance_player);
    console.log(">>>> PDA account balance: ", balance_pda_account);
    console.log("---------------------------------------------");
    
    
    
    ///////////////////////////////
    /////// STEP 3
    ///////////////////////////////

    //-----------------------------------------------
    // starting the game by the server as the signer
    //-----------------------------------------------
    // Start game function - init pda program
    await program.methods.startGame(new anchor.BN(5_000_000_000), bump)
      .accounts({user: server.publicKey, gameState: gameStatePDA, player: player.publicKey
      }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee is the server
    let currentAccountAmount = await program.account.gameState.fetch(gameStatePDA);
    //// PDA account balance must be 5 since player has sent 5 to it
    assert.equal(5_000_000_000, currentAccountAmount.amount.toNumber());




    ///////////////////////////////
    /////// STEP 4
    ///////////////////////////////

    //----------------------------------------------------
    // calling the game result by the server as the signer
    //----------------------------------------------------
    // the second param in gameResult() method is the event with special tax which is 25 percent of the deposited amount 
    await program.methods.gameResult(1, 3)
      .accounts({user: server.publicKey, gameState: gameStatePDA, player: player.publicKey, server: server.publicKey, revenueShareWallet: revenue_share_wallet.publicKey
    }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee is the server
  
    let balance_pda_account_after = await provider.connection.getBalance(gameStatePDA);
    let balance_user_one_after = await provider.connection.getBalance(player.publicKey);
    let balance_revenue_share_wallet = await provider.connection.getBalance(revenue_share_wallet.publicKey);
    console.log("after game results transfer... ")
    console.log("player balance after game: ", balance_user_one_after);
    console.log("PDA account balance after game: ", balance_pda_account_after);
    console.log("revenue share wallet account balance: ", balance_revenue_share_wallet);
    console.log("---------------------------------------------");
  
  
  
  
  
  
  
  
  });
});
