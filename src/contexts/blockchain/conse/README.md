## 💳 Setup Solana Wallet 

* Generate a new keypair using ```solana-keygen new``` command, the following sample output is important for us! We'll use this public key as the program authority to deploy the program with it. 

```console
Wrote new keypair to /home/$USER/.config/solana/id.json
================================================================================
pubkey: 8SzHrPVkDf5xhmjyUJ7W8vDaxhTiGF9XBT9XX2PtiwYF
================================================================================
Save this seed phrase and your BIP39 passphrase to recover your new keypair:
skill divorce afraid nice surface poverty host bright narrow media disorder tuna
================================================================================

```

* You can extract the public key using ```solana address -k /home/$USER/.config/solana/id.json``` command.

* Change the `provider` field inside the `Anchor.toml` file with the proper path of the generated wallet address JSON.

## 🚀 Deploy Ticket Contract on Localnet 

* Fire up a terminal and run a local ledger using ```solana-test-validator``` command.

* In the second terminal:
    * config the solana on the localnet using ```solana config set --url localhost``` command.
    * charge your generated wallet using ```solana airdrop 10``` command or the [faucet](https://solfaucet.com/) site for testnet or devnet.
    * build the contract with ```anchor build --program-name ticket``` command.
    * deploy the contract on the localnet with ```anchor deploy --program-name ticket```
    * the output of the deploy command is something like:
        ```console
            Deploying workspace: http://localhost:8899
            Upgrade authority: /home/$USER/.config/solana/id.json
            Deploying program "ticket"...
            Program path: /home/$USER/Documents/gem/conse/target/deploy/ticket.so...
            Program Id: bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk

            Deploy success
        ```
    * show the deployed program: ```solana program show bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk``` and the output sample would be like:
        ```console
            Program Id: bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk
            Owner: BPFLoaderUpgradeab1e11111111111111111111111
            ProgramData Address: Bq447TCGGXipjaVrQb72TVLrgzVVqD85FYcGDMZeGMgk
            Authority: 8SzHrPVkDf5xhmjyUJ7W8vDaxhTiGF9XBT9XX2PtiwYF
            Last Deployed In Slot: 477
            Data Length: 671648 (0xa3fa0) bytes
            Balance: 4.67587416 SOL
        ```
        in which the owner is the BPF loader which is the owner of every upgradable Solana program account, and the upgrade authority is the public key of the generated wallet info whom has deployed this contract.
    * show the account info: ```solana account bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk```
    * remember to change the program id in `declare_id` in `lib.rs` and `[programs.localnet]` `[programs.mainnet]`, `[programs.devnet]` section, the `ticket` field inside the `Anchor.toml` with the deployed address of the contract or the **Program Id** which is the output of the ```anchor deploy``` command; all mentioned three sections must have same public address of the deployed contract which is the wallet info inside the `*-keypair.json` in the `target/deploy` directory. 
    * also you can check the deployed contract address or the **Program Id** with ```solana address -k target/deploy/ticket-keypair.json``` command.

* In the second one run ```anchor run test-ticket``` command, the output will be:
    ```
    conse-whitelist
    player 1 balance:  10000000000
    sending sol from player to PDA
    >>>> player balance:  4999995000
    >>>> PDA account balance:  5000000000
    ---------------------------------------------
    after game results transfer... 
    player balance after game:  4999995000
    PDA account balance after game:  250000000
    revenue share wallet account balance:  3500000000
    ---------------------------------------------
        ✔ Pda created! (1372ms)


    1 passing (1s)

    Done in 3.27s.
    ```

## 🚀 Deploy Ticket Contract on Devnet

* change the `cluster` field under the `[provider]` section inside the `Anchor.toml` either to `devnet.`

* ```solana config set --url devnet```

* charge your generated wallet using ```solana airdrop 10``` command or the [faucet](https://solfaucet.com/) site for testnet or devnet.

* build the contract with ```anchor build --program-name ticket``` command.
* deploy the contract on the localnet with ```anchor deploy --program-name ticket```
* the output of the deploy command is something like:
    ```console
        Deploying workspace: https://api.devnet.solana.com
        Upgrade authority: /home/$USER/.config/solana/id.json
        Deploying program "ticket"...
        Program path: /home/$USER/Documents/gem/conse/target/deploy/ticket.so...
        Program Id: bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk

        Deploy success
    ```
* show the deployed program: ```solana program show bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk``` and the output sample would be like:
    ```console
        Program Id: bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk
        Owner: BPFLoaderUpgradeab1e11111111111111111111111
        ProgramData Address: Bq447TCGGXipjaVrQb72TVLrgzVVqD85FYcGDMZeGMgk
        Authority: 8SzHrPVkDf5xhmjyUJ7W8vDaxhTiGF9XBT9XX2PtiwYF
        Last Deployed In Slot: 477
        Data Length: 671648 (0xa3fa0) bytes
        Balance: 4.67587416 SOL
    ```
    in which the owner is the BPF loader which is the owner of every upgradable Solana program account, and the upgrade authority is the public key of the generated wallet info whom has deployed this contract.
* show the account info: ```solana account bArDn16ERF32oHbL3Qvbsfz55xkj1CdbPV8VYXJtCtk```
* remember to change the program id in `declare_id` in `lib.rs` and `[programs.localnet]` `[programs.mainnet]`, `[programs.devnet]` section, the `ticket` field inside the `Anchor.toml` with the deployed address of the contract or the **Program Id** which is the output of the ```anchor deploy``` command; all mentioned three sections must have same public address of the deployed contract which is the wallet info inside the `*-keypair.json` in the `target/deploy` directory. 
    * also you can check the deployed contract address or the **Program Id** with ```solana address -k target/deploy/ticket-keypair.json``` command.

## 🍟 Notes

* Before running the deploy script make sure that you've installed the nodejs and also set the `cluster` field to the `mainnet` or the address of your node on either devnet or mainnet like Alchemy node, inside the `Anchor.toml` besides change the solana cluster using ```solana config set --url mainnet``` or ```solana config set --url <CUSTOM_RPC_ENDPOINT>``` also make sure that your account has enough balance for deploying the program.

* once the authority gets changed the program id will be changed too, currently these programs are authorized with `8SzHrPVkDf5xhmjyUJ7W8vDaxhTiGF9XBT9XX2PtiwYF` 

* use ```anchor keys list``` to list all the program ids of each contract.

* if you get error `"*/tsconfig.json" needs an import assertion of type json` just inside the `conse` folder type ```yarn add ts-mocha```.

* to test the whitelist contract run ```anchor run test-whitelist```, just to make sure you have a test validator up and running on your localnet in another terminal.

* currently the program id of the whitelist contract is `2YQmwuktcWmmhXXAzjizxzie3QWEkZC8HQ4ZnRtrKF7p`.

* every time we start a the local node using `solana-test-validator` it search for the existing `test-ledger` folder and if it's not there it'll create a new one, note that in this stage we must deploy the program again to be known by the newly runtime of the local node.  

* currently the program will be deployed on devnet, if you want to deploy on another network just change the `cluster` field under the `[provider]` section inside the `Anchor.toml` either to `mainnet`, `testnet` or your node address.

* after running `anchor build` for the first time a new `keypair.json` will be generated which contains the wallet info the public and private key of the deployed contract in which the program id is the base58 encoded public key address of the deployed contract.

* use ```anchor init NEW_ANCHOR_PROJECT``` to build a new anchor workspace, ```anchor new PROGRAM_NAME``` to create a new program in the workspace, ```anchor build --program-name PROGRAM_NAME``` and ```anchor deploy --program-name PROGRAM_NAME``` to build and deploy the specified program.

* the steps to build and deploy the whitelist contract is the same as the ticket contract, simply run ```anchor build --program-name whitelist``` and then ```anchor build --program-name whitelist```. 

* ```solana balance``` shows the balance of the address inside the `/home/$USER/.config/solana/id.json` on the selected network which is one of the `devnet`, `testnet` or `localhost`.