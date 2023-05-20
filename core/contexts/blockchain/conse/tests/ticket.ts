import * as anchor from "@project-serum/anchor";
import { Program, BorshCoder, EventParser } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { Ticket } from "../target/types/ticket";
import { assert, expect } from "chai";

describe("conse ticket", () => {

  // TODO - use a real provider or connection like testnet or devnet
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  



  const player = anchor.web3.Keypair.generate(); // TODO - wallet handler
  const server = anchor.web3.Keypair.generate(); // TODO - server public key
  const revenue_share_wallet = anchor.web3.Keypair.generate(); // TODO - staking pool account
  


  const lamport_amount = 10_000_000_000;
  const bet_amount = 1_000_000_000;
  const reserve_amount = 5_000_000_000; //// the amount of ticket




  // https://solana.stackexchange.com/questions/2057/what-is-the-relation-between-signers-wallets-in-testing?rq=1
  const program = anchor.workspace.Ticket as Program<Ticket>;
  const provider = anchor.AnchorProvider.env(); 

  
  
  it("PDAs created!", async () => {


    // find pda account for game account
    const [gameStatePDA, bump] = PublicKey
    .findProgramAddressSync(
        [server.publicKey.toBuffer(), player.publicKey.toBuffer()],
        program.programId
      )


      // find pda for the ticket reservation account
    const [ticketStatsPDA, _bump] = PublicKey
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
      console.log("player balance: ", await provider.connection.getBalance(player.publicKey));

      //----------------------------
      // server charging account
      //----------------------------
      const _latestBlockHashforUserOne = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction ({
        blockhash: latestBlockHashforUserOne.blockhash,
        lastValidBlockHeight: _latestBlockHashforUserOne.lastValidBlockHeight,
        signature: await provider.connection.requestAirdrop(server.publicKey, lamport_amount)
      });
      console.log("server balance: ", await provider.connection.getBalance(server.publicKey));



      
      ///////////////////////////////
      /////// STEP 1
      ///////////////////////////////
      
      console.log("sending sol from player and server to PDA");
      //----------------------------
      // sending from player to PDA
      //----------------------------
      let tx_data = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
        fromPubkey: player.publicKey,
        toPubkey: gameStatePDA,
        lamports: bet_amount,    
      }));
      await anchor.web3.sendAndConfirmTransaction(provider.connection, tx_data, [player]);

      //----------------------------
      // sending from server to PDA
      //----------------------------
      let _tx_data = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
        fromPubkey: server.publicKey,
        toPubkey: gameStatePDA,
        lamports: bet_amount,    
      }));
      await anchor.web3.sendAndConfirmTransaction(provider.connection, _tx_data, [server]);

     
        



      ///////////////////////////////
      /////// STEP 2
      ///////////////////////////////

      //--------------------------------
      // PDA, server and player balance
      //--------------------------------
      let balance_player = await provider.connection.getBalance(player.publicKey);
      let server_player = await provider.connection.getBalance(server.publicKey);
      let balance_pda_account = await provider.connection.getBalance(gameStatePDA);
      console.log(">>>> player balance: ", balance_player);
      console.log(">>>> server balance: ", server_player);
      console.log(">>>> PDA account balance: ", balance_pda_account);
      console.log("---------------------------------------------");
      
      
      
      ///////////////////////////////
      /////// STEP 3
      ///////////////////////////////

      
      //-----------------------------------------------
      // starting the game by the server as the signer
      //-----------------------------------------------
      // Start game function - init pda account to mutate it later
      let match_id = 23;
      await program.methods.startGame(new anchor.BN(2_000_000_000), bump, match_id) //// 10_000_000_000 must be the total deposited amount (server + player) 
        .accounts({user: server.publicKey, gameState: gameStatePDA, player: player.publicKey
          }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server
      
      
      //--------------------------------------------------
      // getting the match info of the PDA 
      //--------------------------------------------------
      //--------------------------------------------------
      // getting the match info of the PDA 
      //--------------------------------------------------
      let match_before_start = await program.account.gameState.fetch(gameStatePDA);
      //// PDA account balance must be 10 since player and server each one sent 5 to it
      assert.equal(2_000_000_000, match_before_start.amount.toNumber());
      match_before_start.deck;




      //--------------------------------------------------
      // withdraw from the PDA 
      //--------------------------------------------------
      await program.methods.withdraw()
        .accounts({signer: server.publicKey, gameState: gameStatePDA, server: server.publicKey
      }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server


      //--------------------------------------------------
      // generate card on chain 
      //--------------------------------------------------
      let server_commit = "seed";
      await program.methods.generateCard(server_commit)
        .accounts({user: server.publicKey, gameState: gameStatePDA, server: server.publicKey
      }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server

      

      //------------------------------------------------------
      // meanwhilte, reserving ticket using the built in PDA 
      //------------------------------------------------------

      // const latestBlockHashforPlayer = await provider.connection.getLatestBlockhash();
      // await provider.connection.confirmTransaction ({
      //   blockhash: latestBlockHashforPlayer.blockhash,
      //   lastValidBlockHeight: latestBlockHashforPlayer.lastValidBlockHeight,
      //   signature: await provider.connection.requestAirdrop(player.publicKey, lamport_amount)
      // });
      // console.log("player balance: ", await provider.connection.getBalance(player.publicKey));
      // let ticket_tx_data = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
      //   fromPubkey: player.publicKey,
      //   toPubkey: ticketStatsPDA,
      //   lamports: reserve_amount,    
      // }));
      // await anchor.web3.sendAndConfirmTransaction(provider.connection, ticket_tx_data, [player]);
      // await program.methods.reserveTicket(new anchor.BN(5_000_000_000), "<some_user_id_from_db>", _bump) //// 5_000_000_000 must be the total deposited amount inside the ticketStatsPDA 
      //   .accounts({user: player.publicKey, ticketStats: ticketStatsPDA, server: server.publicKey, satkingPool: revenue_share_wallet.publicKey
      //   }).signers([player]).rpc(); //// signer of this call who must pay for the transaction fee which is the player or user
      // let _currentAccountAmount = await program.account.gameState.fetch(ticketStatsPDA);
      // assert.equal(0, _currentAccountAmount.amount.toNumber()); //// it must 0 in PDA since we withdraw all the deposited amounts from PDA and send them to the revenue share wallet after reservation






      ///////////////////////////////
      /////// STEP 4
      ///////////////////////////////

      //----------------------------------------------------
      // calling the game result by the server as the signer
      //----------------------------------------------------
      let deck = [24, 24]
      // the second param in gameResult() method is the event with special tax which is 25 percent of the deposited amount 
      await program.methods.gameResult(3, 0, deck)
        .accounts({user: server.publicKey, gameState: gameStatePDA, player: player.publicKey, server: server.publicKey, revenueShareWallet: revenue_share_wallet.publicKey
      }).signers([server]).rpc(); //// signer of this call who must pay for the transaction fee which is the server
    
      let balance_pda_account_after = await provider.connection.getBalance(gameStatePDA);
      let balance_server_account_after = await provider.connection.getBalance(server.publicKey);
      let balance_user_one_after = await provider.connection.getBalance(player.publicKey);
      let balance_revenue_share_wallet = await provider.connection.getBalance(revenue_share_wallet.publicKey);
      
      // NOTE - just make sure that the PDA has enough lamports to check its state for deserialization
      console.log("after game results transfer... ")
      console.log("player balance after game: ", balance_user_one_after);
      console.log("PDA account balance after game: ", balance_pda_account_after);
      console.log("server account balance after game: ", balance_server_account_after);
      console.log("revenue share wallet account balance: ", balance_revenue_share_wallet);
      console.log("---------------------------------------------");
    

  
  });
});
