import * as anchor from "@project-serum/anchor";
import { Program, BorshCoder, EventParser } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { Ognils } from "../target/types/ognils";
import { assert, expect } from "chai";

describe("ognils", () => {

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
  const program = anchor.workspace.Ognils as Program<Ognils>;
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





  
  });
});
