#!/bin/bash
sudo apt update && sudo apt install apt-transport-https ca-certificates curl software-properties-common
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu focal stable"
apt-cache policy docker-ce && sudo apt install docker-ce && sudo systemctl status docker

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo install diesel_cli --no-default-features --features postgres

curl -sL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs && sudo apt install -y npm && sudo npm install pm2@latest -g
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo apt update -y && sudo apt upgrade && sudo apt install -y libpq-dev pkg-config build-essential libudev-dev libssl-dev librust-openssl-dev
sudo apt install snapd && sudo snap install core; sudo snap refresh core
sudo snap install --classic certbot && sudo ln -s /snap/bin/certbot /usr/bin/certbot
cargo install sqlant && sudo apt install openjdk-11-jdk && sudo apt install graphviz