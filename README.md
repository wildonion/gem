


## 🖥 Conse PaaS

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) and Solana blockchain. 

## 🧪 Test Conse Hyper Server

```cargo test --bin conse```

## 🏃 Run Conse Hyper Server

```cargo run --bin conse```

## 🛠️ Production Setup

> To update a user access level to dev first signup the user using `/auth/signup` API then run the binary server like so: `./cosne wildonion 0` or `cargo run --bin conse wildonion 0`.

> Before running the deploy script make sure that you've installed the nodejs and also set the `cluster` field to the `mainnet` or the address of your node on either devnet or mainnet like Alchemy node, inside the `Anchor.toml` besides change the solana cluster using ```solana config set --url mainnet``` or ```solana config set --url <CUSTOM_RPC_ENDPOINT>```.

> Also make sure that your account has enough balance for deploying the program.

> Finally Run ```sudo chmod +x deploy.sh && ./deploy.sh```

## 💳 Setup Solana Wallet 

* Generate a new keypair using ```solana-keygen new``` command, the following sample output is important for us! We'll use this public key as the program authority to deploy the program with it. 

```console
Wrote new keypair to /home/$USER/.config/solana/id.json
================================================================================
pubkey: F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ
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
            Program Id: DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD

            Deploy success
        ```
    * show the deployed program: ```solana program show DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD``` and the output sample would be like:
        ```console
            Program Id: DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD
            Owner: BPFLoaderUpgradeab1e11111111111111111111111
            ProgramData Address: Bq447TCGGXipjaVrQb72TVLrgzVVqD85FYcGDMZeGMgk
            Authority: F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ
            Last Deployed In Slot: 477
            Data Length: 671648 (0xa3fa0) bytes
            Balance: 4.67587416 SOL
        ```
        in which the owner is the BPF loader which is the owner of every upgradable Solana program account, and the upgrade authority is the public key of the generated wallet info whom has deployed this contract.
    * show the account info: ```solana account DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD```
    * remember to change the program id in `declare_id` in `lib.rs` and `[programs.localnet]` `[programs.mainnet]`, `[programs.devnet]` section, the `ticket` field inside the `Anchor.toml` with the deployed address of the contract or the **Program Id** which is the output of the ```anchor deploy``` command; all mentioned three sections must have same public address of the deployed contract which is the wallet info inside the `*-keypair.json` in the `target/deploy` directory. 
    * also you can check the deployed contract address or the **Program Id** with ```solana address -k target/deploy/ticket-keypair.json``` command.

* Stop the first terminal and in the second one run ```anchor run test-ticket``` command, since anchor will run a local ledger for the test process on its own if the `cluster` field under the `[provider]` section is set to `localnet`, the output will be:
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
        Program Id: DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD

        Deploy success
    ```
* show the deployed program: ```solana program show DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD``` and the output sample would be like:
    ```console
        Program Id: DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD
        Owner: BPFLoaderUpgradeab1e11111111111111111111111
        ProgramData Address: Bq447TCGGXipjaVrQb72TVLrgzVVqD85FYcGDMZeGMgk
        Authority: F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ
        Last Deployed In Slot: 477
        Data Length: 671648 (0xa3fa0) bytes
        Balance: 4.67587416 SOL
    ```
    in which the owner is the BPF loader which is the owner of every upgradable Solana program account, and the upgrade authority is the public key of the generated wallet info whom has deployed this contract.
* show the account info: ```solana account DseCcTkkVGWnHnt6s8uMdcb5EDaduxaKxVfEu6aVkfLD```
* remember to change the program id in `declare_id` in `lib.rs` and `[programs.localnet]` `[programs.mainnet]`, `[programs.devnet]` section, the `ticket` field inside the `Anchor.toml` with the deployed address of the contract or the **Program Id** which is the output of the ```anchor deploy``` command; all mentioned three sections must have same public address of the deployed contract which is the wallet info inside the `*-keypair.json` in the `target/deploy` directory. 
    * also you can check the deployed contract address or the **Program Id** with ```solana address -k target/deploy/ticket-keypair.json``` command.

## 📇 Notes

* once the authority gets changed the program id will be changed too, currently these programs are authorized with `F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ` 

* use ```anchor keys list``` to list all the program ids of each contract.

* if you get error `"*/tsconfig.json" needs an import assertion of type json` just inside the `conse` folder type ```yarn add ts-mocha```.

* to test the whitelist contract run ```anchor run test-whitelist```, just to make sure you have a test validator up and running on your localnet in another terminal.

* currently the program id of the whitelist contract is `4ZdXCpgo5wZTVbh1QV2yjcsiX1jCSzsqkfWeYwXwcAU2`.

* server must call and sign the `StartGame()` to start the game so if player hit the start game button on frontend, an API call must be invoked to the server which will call the contract real method or `StartGame()`.

* server must call and sign the `gameResult()` of the contract and pass the `winner` either 0 or 1 and `instruct` values between 0 up to 4, the contract will do the rest of the things.

* the third instruction has an special tax amount which is %25 of the deposited amount.

* currently the program will be deployed on devnet, if you want to deploy on another network just change the `cluster` field under the `[provider]` section inside the `Anchor.toml` either to `mainnet`, `testnet` or your node address.

* after running `anchor build` for the first time a new `keypair.json` will be generated which contains the wallet info the public and private key of the deployed contract in which 
we the program id is the public key address of the deployed contract.

* use ```anchor init NEW_ANCHOR_PROJECT``` to build a new anchor workspace, ```anchor new PROGRAM_NAME``` to create a new program in the workspace, ```anchor build --program-name PROGRAM_NAME``` and ```anchor deploy --program-name PROGRAM_NAME``` to build and deploy the specified program.

* the steps to build and deploy the whitelist contract is the same as the ticket contract, simply run ```anchor build --program-name whitelist``` and then ```anchor build --program-name whitelist```. 

* ```solana balance``` shows the balance of the address inside the `/home/$USER/.config/solana/id.json` on the selected network which is one of the `devnet`, `testnet` or `localhost`.

* in order to use docker containers inside another one by its DNS name, all of them must be inside the same network bridge.

* clean all the docker cache using ```sudo docker buildx prune --all``` command.

* register push notification strategy: client `<--hyper REST-->` register a push notif route using redis client `<--REDIS SERVER-->` register pubsub topic on redis server.

* subscribing to push notification strategy: client `<--gql subscription-->` redis published topics inside the server.

* subscribing to realtiming chat strategy: client `<--gql subscription ws-->` hyper gql ws server contains redis and mongodb clients setup `<--REDIS & MONGODB SERVER-->` store data on redis. 

## 🚧 WIP

* reserve ticket contract tests and update to the latest `Anchor` version also fix the whitelist contract issue in `initializeWhitelist` method.

* `ed25519` keypair for server verification, updating app and time hash based locking api, also see the one inside the [payma](https://github.com/wildonion/payma) repo.

* handle different versions of [hyper](https://hyper.rs/) in `main.rs` using its env var.

* complete graphql, redis and websocket routes setup for realtime streaming like chatapp and push notification, also add redis docker image inside the `docker-compose.yml`.

* balance the loads between docker services and images using `k8s` on `AWS` cloud also CI/CD configuration files based on the latest commits. 

* all TODOs inside the app

* backend design pattern sketch using freeform.

* communication between Conse and the [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) must be done through the TCP stream since [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) supports TCP stream.

* use an AI model like [STEM](https://github.com/wildonion/stem) which suggests player the tips and tricks for a new game based on behavioural graph of the player collected by the history of each event's `phases` field

* use [uniXerr](https://github.com/wildonion/uniXerr) coin generation AI model which players get rewarded based on their scores and positions which are collected by each event manager inside the game, then update the balance field of the user based on that
