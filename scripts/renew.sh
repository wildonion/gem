#!/bin/bash


echo "-> add domain? "
read is_new_domain
DIRECTORY=..
REALPTH_GEM=TRUEDIR=$(cd -- "$DIRECTORY" && pwd)

if [[ $is_new_domain == "Y" || $is_new_domain == "y" ]]; then
    echo "creating new SSL certificate and key files for $DOMAIN using certbot,
    ensure that you have a domain that points to this machine and it can accepts inbound connections 
    from the internet"
    echo "-> enter domain? "
    read DOMAIN
    NGINX_CONTAINER_ID=$(docker container ls  | grep 'nginx' | awk '{print $1}')
    sudo docker stop $NGINX_CONTAINER_ID && sudo certbot certonly --standalone -d $DOMAIN && sudo docker start $NGINX_CONTAINER_ID
    sudo cp /etc/letsencrypt/live/$DOMAIN/fullchain.pem $REALPTH_GEM/infra/cert/cert-conse.pem && sudo cp /etc/letsencrypt/live/$DOMAIN/fullchain.pem $REALPTH_GEM/infra/docker/nginx/cert-conse.pem
    sudo cp /etc/letsencrypt/live/$DOMAIN/privkey.pem $REALPTH_GEM/infra/cert/key-conse.pem && sudo cp /etc/letsencrypt/live/$DOMAIN/privkey.pem $REALPTH_GEM/infra/docker/nginx/key-conse.pem
    echo "okay now you can use $REALPTH_GEM/infra/docker/nginx/key-conse.pem and $REALPTH_GEM/infra/docker/nginx/cert-conse.pem in your nginx conf"
else
    echo "if it's not about adding domain, maybe a new config file is going to be added into the nginx docker, i don't know! 🤔"
fi
# If you use the host network mode for a container, 
# that container’s network stack is not isolated from the 
# Docker host (the container shares the host’s networking namespace), 
# and the container does not get its own IP-address allocated. 
# For instance, if you run a container which binds to port 80 and 
# you use host networking, the container’s application is available 
# on port 80 on the host’s IP address thus we use the host network 
# so we can access containers on 127.0.0.1 with their exposed ports 
# inside the nginx conf without their dns name or ip address. 
echo "[🛰] redeploying nginx docker container"
docker system prune --all
sudo docker build -t --no-cache nginx -f infra/docker/nginx/Dockerfile .
sudo docker stop nginx
sudo docker rm -f nginx
sudo docker run -d -it -p 80:80 -p 443:443 --name nginx --network host nginx
sudo docker ps -a && sudo docker compose ps -a && sudo docker images