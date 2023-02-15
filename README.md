


# 🏗 Conse PaaS

Conse is an AI based Crypto Game Event Manager Platform on top of [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) and Solana blockchain. 

> To update a user access level to dev first signup the user using `/auth/signup` API then run the binary like so: `./app wildonion 0`

## 🧪 Test Conse Hyper Server

```cargo test --bin conse```

## 💳 Setup Solana Wallet 

* Generate a new keypair using ```solana-keygen new``` command, the following sample output is important for us! 

```console
Wrote new keypair to /home/wildonion/.config/solana/id.json
================================================================================
pubkey: F3Ngjacvfd37nitEDZMuSV9Ckv5MHBdaB3iMhPiUaztQ
================================================================================
Save this seed phrase and your BIP39 passphrase to recover your new keypair:
skill divorce afraid nice surface poverty host bright narrow media disorder tuna
================================================================================

```

* You can extract the public key using `solana address -k /home/$USER/.config/solana/id.json` command.

* Change the `provider` field inside the `Anchor.toml` file with the proper path of the generated wallet address JSON.

## 🛠️ Production Setup

> Before running the deploy script make sure that you've installed the nodejs and also set the cluster to the `mainnet` API inside the `Anchor.toml`

> Also make sure that your account has enough balance for deploying the program.

```sudo chmod +x deploy.sh && ./deploy.sh```

## 🚀 Deploy Contract on Localnet 

* Fire up a terminal and run a local ledger using ```solana-test-validator``` command.

* In the second terminal:
    * config the solana on the localnet using ```solana config set --url localhost``` command.
    * charge your generated wallet using ```solana airdrop 10``` command.
    * build the contract with ```anchor build``` command.
    * deploy the contract on the localnet with ```anchor deploy```
    * remember to change the program id in `declare_id` in `lib.rs` and `[programs.localnet]` section, the `conse` field inside the `Anchor.toml` with the deployed address of the contract which is the output of the ```anchor deploy``` command.
    * also you can check the deployed contract address with ```solana address -k target/deploy/conse-keypair.json``` command.

* Stop the first terminal and in the second one run ```anchor test``` command, since this command will run a local ledger for the test proces on its own.

## 🚧 WIP

* complete conse solana programs inside the `conse` folder using [anchor](https://www.anchor-lang.com/) 

* adding Graphql for realtime streaming using hyper with [juniper](https://graphql-rust.github.io/juniper/master/index.html)

* updating [hyper](https://hyper.rs/) to latest version

* HAProxy, k8s-ing docker containers in `docker-compose.yml` and CI/CD in `deploy.sh` on [xaas](https://xaas.ir/)

* all TODOs inside the app

* communication between Conse and the [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) must be done through the TCP stream since [coiniXerr](https://github.com/wildonion/uniXerr/tree/master/infra/valhalla/coiniXerr) supports TCP stream.

* use an AI model which suggests player the tips and tricks for a new game based on behavioural graph of the player collected by the history of each event's `phases` field

* use [uniXerr](https://github.com/wildonion/uniXerr) coin generation AI model which players get rewarded based on their scores and positions which are collected by each event manager inside the game, then update the balance field of the user based on that
