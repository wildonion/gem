#!/bin/bash
sudo apt update && sudo apt install -y apt-transport-https ca-certificates curl software-properties-common curl gnupg
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg
echo \
  "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
sudo systemctl status docker

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo install diesel_cli --no-default-features --features postgres

curl -sL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs && sudo apt install -y npm && sudo npm install pm2@latest -g
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo apt update -y && sudo apt upgrade && sudo apt install -y libpq-dev pkg-config build-essential libudev-dev libssl-dev librust-openssl-dev
sudo apt install -y snapd && sudo snap install core; sudo snap refresh core
sudo snap install --classic certbot && sudo ln -s /snap/bin/certbot /usr/bin/certbot
cargo install sqlant && sudo apt install -y openjdk-11-jdk && sudo apt install -y graphviz

# --- for docker setup ---
cd ..
sudo rm .env && sudo mv .env.prod .env
sudo mv twitter-accounts.prod.json twitter-accounts.json
echo ">>> Please fill up the 'twitter-accounts.json' without your twitter dev account keys using the conse panel API with admin access"

echo "[?] Enter SMS API Token: "
read SMS_API_TOKEN
echo SMS_API_TOKEN=$SMS_API_TOKEN >> .env

echo "[?] Enter SMS API Template: "
read SMS_API_TAMPLATE
echo SMS_API_TAMPLATE=$SMS_API_TAMPLATE >> .env

echo "[?] Enter Machine Id: "
read MACHINE_ID
echo MACHINE_ID=$MACHINE_ID >> .env
echo "[?] Enter Node Id: "
read NODE_ID
echo NODE_ID=$NODE_ID >> .env

sudo docker network create -d bridge gem || true