#!/bin/bash
sudo chown -R root:root . && sudo chmod -R 777 . && sudo chmod -R 777 .
sudo chmod +x /root && sudo chown -R root:root /root && sudo chmod -R 777 /root
if [[ ! -f "devops/cert/conse_cert.pem" ]] && [[ ! -f "devops/cert/conse_key.pem" ]]
then
    echo "cert files doesn't exist creating new SSL certificate and key files for conse using certbot,
    ensure that you have a domain that points to this machine and that it can accepts inbound connections 
    from the internet"
    echo "[?] enter domain to put ssl on it: "  
    read DOMAIN  
    if [ -z "$DOMAIN" ]
    then
        echo "domain not entered!"
    else
        sudo certbot certonly && sudo cp /etc/letsencrypt/live/$domain/fullchain.pem devops/cert/conse.pem
    fi
fi
# --------------------------------------------------------------------------------------------------------------------------------
# ------------------------------------------------------ DOCKER INSTALL START ------------------------------------------------------
# --------------------------------------------------------------------------------------------------------------------------------
sudo apt update && sudo apt install apt-transport-https ca-certificates curl software-properties-common
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu focal stable"
apt-cache policy docker-ce && sudo apt install docker-ce && sudo systemctl status docker
# --------------------------------------------------------------------------------------------------------------------------------
# ------------------------------------------------------- DOCKER INSTALL END -------------------------------------------------------
# -------------------------------------------------------------------------------------------------------------------------------- 
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
sudo apt update && sudo apt upgrade -y
curl -sL https://deb.nodesource.com/setup_16.x | sudo -E bash -
sudo apt install -y nodejs && sudo apt install -y npm && sudo npm install pm2@latest -g
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
sudo apt-get update && sudo apt-get upgrade && sudo apt-get install -y pkg-config build-essential libudev-dev libssl-dev librust-openssl-dev