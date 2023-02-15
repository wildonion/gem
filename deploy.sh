#!/bin/bash
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/yarn.gpg
echo "deb [signed-by=/etc/apt/trusted.gpg.d/yarn.gpg] https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update && sudo apt install yarn
sh -c "$(curl -sSfL https://release.solana.com/v1.15.1/install)"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
export PATH="/home/$USER/.local/share/solana/install/active_release/bin:$PATH"
solana-keygen new
echo "airdropping on devnet"
solana config set --url https://api.devnet.solana.com
for i in {1..50} # airdropping inside the loop due to the solana rate limit
do
    solana airdrop 0.1 /home/wildonion/.config/solana/id.json --url devnet
done
sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev libssl-dev
avm install 0.24.2
avm use 0.24.2
npm install --force
anchor test
anchor deploy
echo "[+] Building Conse PaaS"
sudo chown -R root:root . && sudo chmod -R 777 .
cargo build --bin conse --release
sudo rm conse
sudo cp target/release/conse ./conse
sudo pm2 delete conse
sudo pm2 start conse --name conse
sudo pm2 status