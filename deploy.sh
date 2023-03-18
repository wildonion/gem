#!/bin/bash
SERVER_IP=hostname -I | awk '{print $1}'
sudo apt update
sudo apt install apt-transport-https ca-certificates curl software-properties-common
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu focal stable"
apt-cache policy docker-ce
sudo apt install docker-ce
sudo systemctl status docker
sudo docker compose -f  docker-compose.yml build --no-cache
sudo docker compose up -d --force-recreate
sudo docker inspect -f '{{.Name}} - {{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $(docker ps -aq)
sudo docker exec -it mongodb mongod --bind_ip $SERVER_IP ########## allow only the server ip access the db
sudo docker run -d --name haproxy --net gem -v devops/conf/haproxy.cfg:/usr/local/etc/haproxy -p 8404:8404 -p 7439:7439 -e SERVER_IP=$SERVER_IP haproxytech/haproxy-alpine:2.4 
sudo docker ps -a && sudo docker compose ps -a
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
sudo chown -R root:root . && sudo chmod -R 777 .
sudo chown -R www-data:www-data . && sudo chmod -R 777 .
sudo chmod +x /root && sudo chown -R root:root /root && sudo chmod -R 777 /root
sudo chmod +x /root && sudo chown -R www-data:www-data /root && sudo chmod -R 777 /root
sudo apt update && sudo apt upgrade -y
curl -sL https://deb.nodesource.com/setup_16.x | sudo -E bash -
sudo apt install -y nodejs && sudo apt install -y npm
npm install pm2@latest -g
sudo apt install -y snapd
sudo snap install core; sudo snap refresh core
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
wget -qO - https://www.mongodb.org/static/pgp/server-6.0.asc | sudo apt-key add -
sudo apt-get install gnupg
wget -qO - https://www.mongodb.org/static/pgp/server-6.0.asc | sudo apt-key add -
echo "deb [ arch=amd64,arm64 ] https://repo.mongodb.org/apt/ubuntu bionic/mongodb-org/6.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-6.0.list
sudo apt-get update -y && sudo apt-get install -y mongodb-org
sudo mkdir -p /data/db && sudo chown -R $USER /data/db && sudo systemctl restart nginx
mongoimport --db conse --collection roles devops/conse-collections/roles.json
mongoimport --db conse --collection sides devops/conse-collections/sides.json
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/yarn.gpg
echo "deb [signed-by=/etc/apt/trusted.gpg.d/yarn.gpg] https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
sudo apt update -y && sudo apt install yarn
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
export PATH="/home/$USER/.local/share/solana/install/active_release/bin:$PATH"
sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev libssl-dev
avm install latest
avm use latest
echo "[?] Deploy What? (programs || gem)"
read BUILDFOR
if [[ $BUILDFOR == "programs" ]]; then
    solana-keygen new
    cd conse && npm install --force
    echo "[?] Build What? (conse || whitelist)"
    read PROGRAM
    if [[ $PROGRAM == "conse" ]]; then
        anchor build --program-name conse
        anchor deploy --program-name conse
    elif [[ $BUILDFOR == "whitelist" ]]; then
        anchor build --program-name whitelist
        anchor deploy --program-name whitelist
    if
elif [[ $BUILDFOR == "gem" ]]; then
    echo "[+] Building Conse PaaS"
    cargo build --bin conse --release
    sudo rm /usr/bin/conse
    sudo cp target/release/conse /usr/bin/conse && sudo chmod +x /usr/bin/conse 
    sudo cp .env /usr/bin/ && sudo cp nfts.json /usr/bin/ 
    sudo pm2 delete conse && cd /usr/bin/
    sudo pm2 start conse --name conse
    sudo pm2 status
fi