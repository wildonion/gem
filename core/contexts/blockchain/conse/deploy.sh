#!/bin/bash
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev libssl-dev librust-openssl-dev
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/yarn.gpg
echo "deb [signed-by=/etc/apt/trusted.gpg.d/yarn.gpg] https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update -y && sudo apt install yarn
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
export PATH="/home/$USER/.local/share/solana/install/active_release/bin:$PATH"
avm install 0.27.0
avm use 0.27.0
sudo npm install --force
anchor build --program-name ticket && anchor deploy --program-name ticket
anchor build --program-name whitelist && anchor deploy --program-name whitelist
anchor build --program-name ognils && anchor deploy --program-name ognils