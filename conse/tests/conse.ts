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
  const tax_account = anchor.web3.Keypair.generate(); // TODO - staking pool account
  
  const lamport_amount = 10_000_000_000;
  const lamport_to_send = 5_000_000_000;
  const lamport_to_send_second = 2_000_000_000;
  
  const program = anchor.workspace.ConseGemReservation as Program<ConseGemReservation>;
  const provider = anchor.AnchorProvider.env();
  
  it("Pda created!", async () => {
  // find pda account
  const [gameStatePDA, bump] = PublicKey
  .findProgramAddressSync(
    [provider.wallet.publicKey.toBuffer(), user_one.publicKey.toBuffer()],
    program.programId
    )
    
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
    
    // TODO - Error: failed to send transaction: Transaction simulation failed: This program may not be used for executing instructions
    
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
    
    let balance_user_two_second_function = await provider.connection.getBalance(user_one.publicKey);
    let balance_pda_account_second_function = await provider.connection.getBalance(gameStatePDA);
    console.log("sending sol from player 2 to pda... ");
    console.log("player 2 balance: ", balance_user_two_second_function);
    console.log("pda account balance: ", balance_pda_account_second_function);
    console.log("---------------------------------------------");
    // Second player function
    await program.methods.secondPlayer(new anchor.BN(2_000_000_000)).accounts({user: provider.wallet.publicKey, gameState: gameStatePDA, playerTwo: user_two.publicKey}).rpc();
    let secondAccountAmount = await program.account.gameState.fetch(gameStatePDA);
    assert.equal(7_000_000_000, secondAccountAmount.amount.toNumber());
    // send solana from pda to account
    await program.methods.gameResult(1, 1).accounts({user: provider.wallet.publicKey, gameState: gameStatePDA, playerOne: user_one.publicKey, playerTwo: user_two.publicKey, taxAccount: tax_account.publicKey}).rpc();

    let balance_user_two = await provider.connection.getBalance(user_two.publicKey);
    let balance_pda_account_after = await provider.connection.getBalance(gameStatePDA);
    let balance_user_one_after = await provider.connection.getBalance(user_one.publicKey);
    let balance_tax_account = await provider.connection.getBalance(tax_account.publicKey);

    console.log("after game results transfer... ")
    console.log("user1 balance after game: ", balance_user_one_after);
    console.log("user2 balance after game: ", balance_user_two);
    console.log("pda account balance after game: ", balance_pda_account_after);
    console.log("tax account balance: ", balance_tax_account);
    console.log("---------------------------------------------");
  });
});
