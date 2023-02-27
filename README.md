


## 🏗 Conse PaaS

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) and Solana blockchain. 

## 🧪 Test Conse Hyper Server

```cargo test --bin conse```

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

## 🛠️ Production Setup

> To update a user access level to dev first signup the user using `/auth/signup` API then run the binary server like so: `./cosne wildonion 0`

> Before running the deploy script make sure that you've installed the nodejs and also set the `cluster` field to the `mainnet` or the address of your node on either devnet or mainnet like Alchemy node, inside the `Anchor.toml`

> Also make sure that your account has enough balance for deploying the program.

> Finally Run ```sudo chmod +x deploy.sh && ./deploy.sh```

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
            Program Id: 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z

            Deploy success
        ```
    * show the deployed program: ```solana program show 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z``` and the output sample would be like:
        ```console
            Program Id: 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z
            Owner: BPFLoaderUpgradeab1e11111111111111111111111
            ProgramData Address: Bq447TCGGXipjaVrQb72TVLrgzVVqD85FYcGDMZeGMgk
            Authority: F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ
            Last Deployed In Slot: 477
            Data Length: 671648 (0xa3fa0) bytes
            Balance: 4.67587416 SOL
        ```
        in which the owner is the BPF loader which is the owner of every upgradable Solana program account, and the upgrade authority is the public key of the generated wallet info whom has deployed this contract.
    * show the account info: ```solana account 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z```
    * remember to change the program id in `declare_id` in `lib.rs` and `[programs.localnet]` `[programs.mainnet]`, `[programs.devnet]` section, the `ticket` field inside the `Anchor.toml` with the deployed address of the contract or the **Program Id** which is the output of the ```anchor deploy``` command; all mentioned three sections must have same public address of the deployed contract which is the wallet info inside the `*-keypair.json` in the `target/deploy` directory. 
    * also you can check the deployed contract address or the **Program Id** with ```solana address -k target/deploy/ticket-keypair.json``` command.

* Stop the first terminal and in the second one run ```anchor test``` command, since anchor will run a local ledger for the test process on its own.

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
        Program Id: 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z

        Deploy success
    ```
* show the deployed program: ```solana program show 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z``` and the output sample would be like:
    ```console
        Program Id: 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z
        Owner: BPFLoaderUpgradeab1e11111111111111111111111
        ProgramData Address: Bq447TCGGXipjaVrQb72TVLrgzVVqD85FYcGDMZeGMgk
        Authority: F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ
        Last Deployed In Slot: 477
        Data Length: 671648 (0xa3fa0) bytes
        Balance: 4.67587416 SOL
    ```
    in which the owner is the BPF loader which is the owner of every upgradable Solana program account, and the upgrade authority is the public key of the generated wallet info whom has deployed this contract.
* show the account info: ```solana account 2dxHAp1hE9R4zieNEAVct4H5gC9xbYzdJ3DJnJ7EU62Z```
* remember to change the program id in `declare_id` in `lib.rs` and `[programs.localnet]` `[programs.mainnet]`, `[programs.devnet]` section, the `ticket` field inside the `Anchor.toml` with the deployed address of the contract or the **Program Id** which is the output of the ```anchor deploy``` command; all mentioned three sections must have same public address of the deployed contract which is the wallet info inside the `*-keypair.json` in the `target/deploy` directory. 
    * also you can check the deployed contract address or the **Program Id** with ```solana address -k target/deploy/ticket-keypair.json``` command.

## 📇 Notes

* frontend must call the `gameResult()` of the contract and pass the `winner` and `instruct` values, the contract will do the rest of the things.

* the third instruction has an special tax amount which is %25 of the deposited amount.

* currently the program will be deployed on devnet, if you want to deploy on another network just change the `cluster` field under the `[provider]` section inside the `Anchor.toml` either to `mainnet`, `testnet` or your node address.

* after running `anchor build` for the first time a new `keypair.json` will be generated which contains the wallet info the public and private key of the deployed contract in which 
we the program id is the public key address of the deployed contract.

* use ```anchor init NEW_ANCHOR_PROJECT``` to build a new anchor workspace, ```anchor build --program-name PROGRAM_NAME``` and ```anchor deploy --program-name PROGRAM_NAME``` to build and deploy the specified program.

* the steps to build and deploy the whitelist contract is the same as the ticket contract. 

* ```solana balance``` shows the balance of the address inside the `/home/$USER/.config/solana/id.json` on the selected network which is one of the `devnet`, `testnet` or `localhost`.

## 🚧 WIP

* complete solana programs inside the `conse` folder using [anchor](https://www.anchor-lang.com/) 

* adding Graphql for realtime streaming using hyper with [juniper](https://graphql-rust.github.io/juniper/master/index.html)

* updating [hyper](https://hyper.rs/) to latest version

* HAProxy, k8s-ing docker containers in `docker-compose.yml` and CI/CD in `deploy.sh` on [xaas](https://xaas.ir/)

* all TODOs inside the app

* communication between Conse and the [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) must be done through the TCP stream since [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) supports TCP stream.

* use an AI model which suggests player the tips and tricks for a new game based on behavioural graph of the player collected by the history of each event's `phases` field

* use [uniXerr](https://github.com/wildonion/uniXerr) coin generation AI model which players get rewarded based on their scores and positions which are collected by each event manager inside the game, then update the balance field of the user based on that
