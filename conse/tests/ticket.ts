import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { ConseGemReservation } from "../target/types/conse_gem_reservation";
import { assert, expect } from "chai";

describe("conse-gem-reservation", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  
  const user_one = anchor.web3.Keypair.generate(); // TODO - need wallet handler
  const user_two = anchor.web3.Keypair.generate(); // TODO - it can be the server public key
  const revenue_share_wallet = anchor.web3.Keypair.generate(); // TODO - staking pool account
  
  const lamport_amount = 10_000_000_000;
  const lamport_to_send = 5_000_000_000;
  const lamport_to_send_second = 2_000_000_000;
  
  // https://docs.solana.com/developing/programming-model/accounts#ownership-and-assignment-to-programs
  //// the owner of the program account must 
  //// matches the program id that has been
  //// deployed since the security model enforces 
  //// that an account's data can only be modified 
  //// by the account's owner program and no other 
  //// accounts can call the contract method on their 
  //// own to amend and mutate the instruction data
  //// passed in to that account on the chain because 
  //// because a malicious user could create accounts 
  //// with arbitrary data and then pass these accounts 
  //// to the program in place of valid accounts then 
  //// the arbitrary data could be crafted in a way that 
  //// leads to unexpected or harmful program behavior.
  const program = anchor.workspace.ConseGemReservation as Program<ConseGemReservation>;
  const provider = anchor.AnchorProvider.env();
  
  it("Pda created!", async () => {
  // find pda account
  const [gameStatePDA, bump] = PublicKey
  .findProgramAddressSync(
    [provider.wallet.publicKey.toBuffer(), user_one.publicKey.toBuffer()],
    program.programId
    )
    
    // -------------------------------------------------
    // user one balance account
    const latestBlockHashforUserOne = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction ({
      blockhash: latestBlockHashforUserOne.blockhash,
      lastValidBlockHeight: latestBlockHashforUserOne.lastValidBlockHeight,
      signature: await provider.connection.requestAirdrop(user_one.publicKey, lamport_amount)
    });
    console.log("player 1 balance: ", await provider.connection.getBalance(user_one.publicKey));
    // user two balance account
    const latestBlockHashforUserTwo = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction ({
      blockhash: latestBlockHashforUserTwo.blockhash,
      lastValidBlockHeight: latestBlockHashforUserTwo.lastValidBlockHeight,
      signature: await provider.connection.requestAirdrop(user_two.publicKey, lamport_amount),
    });
    console.log("player 2 balance: ", await provider.connection.getBalance(user_two.publicKey));
    // -------------------------------------------------
    



    
    // -------------------------------------------------
    // user 1 send solana to pda 
    let tx_data = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
      fromPubkey: user_one.publicKey,
      toPubkey: gameStatePDA,
      lamports: lamport_to_send,
    }));
    
    await anchor.web3.sendAndConfirmTransaction(provider.connection, tx_data, [user_one]);
    let balance_user_one = await provider.connection.getBalance(user_one.publicKey);
    let balance_pda_account = await provider.connection.getBalance(gameStatePDA);
    console.log("sending sol from player 1 to pda... ");
    console.log("player 1 balance: ", balance_user_one);
    console.log("pda account balance: ", balance_pda_account);
    console.log("---------------------------------------------");
    // -------------------------------------------------



    
    // -------------------------------------------------
    // Start game function - init pda program 
    await program.methods.startGame(new anchor.BN(5_000_000_000), bump).accounts({user: provider.wallet.publicKey, gameState: gameStatePDA, playerOne: user_one.publicKey}).rpc();
    let currentAccountAmount = await program.account.gameState.fetch(gameStatePDA);
    assert.equal(5_000_000_000, currentAccountAmount.amount.toNumber());
    // user 2 send solana to pda 
    let second_tx_data = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
      fromPubkey: user_two.publicKey,
      toPubkey: gameStatePDA,
      lamports: lamport_to_send_second,
    }));
    await anchor.web3.sendAndConfirmTransaction(provider.connection, second_tx_data, [user_two]);
    // -------------------------------------------------





    // -------------------------------------------------
    let balance_user_two_second_function = await provider.connection.getBalance(user_one.publicKey);
    let balance_pda_account_second_function = await provider.connection.getBalance(gameStatePDA);
    console.log("sending sol from player 2 to pda... ");
    console.log("player 2 balance: ", balance_user_two_second_function);
    console.log("pda account balance: ", balance_pda_account_second_function);
    console.log("---------------------------------------------");
    // -------------------------------------------------
    


    
    // -------------------------------------------------
    // Second player function
    await program.methods.secondPlayer(new anchor.BN(2_000_000_000)).accounts({user: provider.wallet.publicKey, gameState: gameStatePDA, playerTwo: user_two.publicKey}).rpc();
    let secondAccountAmount = await program.account.gameState.fetch(gameStatePDA);
    assert.equal(7_000_000_000, secondAccountAmount.amount.toNumber());
    // if the first param is 1 means player two 
    // the second param in gameResult() method is the event with special tax which is 25 percent of the deposited amount 
    // NOTE - gameResult method must be called from the server side since the signer must be the server account
    await program.methods.gameResult(1, 3).accounts({user: provider.wallet.publicKey, gameState: gameStatePDA, playerOne: user_one.publicKey, playerTwo: user_two.publicKey, revenueShareWallet: revenue_share_wallet.publicKey}).rpc();
    // -------------------------------------------------




    // -------------------------------------------------
    let balance_user_two = await provider.connection.getBalance(user_two.publicKey);
    let balance_pda_account_after = await provider.connection.getBalance(gameStatePDA);
    let balance_user_one_after = await provider.connection.getBalance(user_one.publicKey);
    let balance_revenue_share_wallet = await provider.connection.getBalance(revenue_share_wallet.publicKey);
    console.log("after game results transfer... ")
    console.log("user1 balance after game: ", balance_user_one_after);
    console.log("user2 balance after game: ", balance_user_two);
    console.log("pda account balance after game: ", balance_pda_account_after);
    console.log("revenue share account balance after result: ", balance_revenue_share_wallet);
    console.log("---------------------------------------------");
  });
});
