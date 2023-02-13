#!/bin/bash
sh -c "$(curl -sSfL https://release.solana.com/v1.15.1/install)"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev libssl-dev
avm install latest
avm use latest
sudo chown -R root:root . && sudo chmod -R 777 .
cargo build --bin conse --release
sudo rm conse
sudo cp target/release/conse ./conse
sudo pm2 delete conse
sudo pm2 start conse --name conse