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

sudo rm .env && sudo mv .env.prod .env
echo "[?] Enter OpenAI token: "
read OPENAI_TOKEN
echo "[?] Enter Discord token: "
read DISCORD_TOKEN
echo OPENAI_KEY=$OPENAI_TOKEN >> .env
echo DISCORD_TOKEN=$DISCORD_TOKEN >> .env
echo "[?] Enter Twitter keys: "
echo "\t>> bearer token: "
read TWITTER_BEARER_TOKEN
echo TWITTER_BEARER_TOKEN=$TWITTER_BEARER_TOKEN >> .env

echo "\t>> access token: "
read TWITTER_ACCESS_TOKEN
echo TWITTER_ACCESS_TOKEN=$TWITTER_ACCESS_TOKEN >> .env

echo "\t>> access token secret: "
read TWITTER_ACCESS_TOKEN_SECRET
echo TWITTER_ACCESS_TOKEN_SECRET=$TWITTER_ACCESS_TOKEN_SECRET >> .env

echo "\t>> consumer key: "
read TWITTER_CONSUMER_KEY
echo TWITTER_CONSUMER_KEY=$TWITTER_CONSUMER_KEY >> .env

echo "\t>> consumer secret: "
read TWITTER_CONSUMER_SECRET
echo TWITTER_CONSUMER_SECRET=$TWITTER_CONSUMER_SECRET >> .env

echo "\t>> client id: "
read TWITTER_CLIENT_ID
echo TWITTER_CLIENT_ID=$TWITTER_CLIENT_ID >> .env

echo "\t>> client secret: "
read TWITTER_CLIENT_SECRET
echo TWITTER_CLIENT_SECRET=$TWITTER_CLIENT_SECRET >> .env

sudo chmod 666 /var/run/docker.sock && docker system prune --all
export SERVER_IP=$(hostname -I | awk '{print $1}')
export PASSEORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70
sudo docker network create -d bridge gem || true