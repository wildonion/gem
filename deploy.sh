#!/bin/bash
sudo rm .env && sudo mv .env.prod .env
sudo chmod 666 /var/run/docker.sock && docker system prune --all
export SERVER_IP=$(hostname -I | awk '{print $1}')
sudo docker network create -d bridge gem
sudo docker run -d --network gem --name redis --restart unless-stopped -p 6379:6379 -v ./infra/data/redis/:/data redis/redis-stack-server:latest
sudo docker run -d --network gem --name mongodb --restart unless-stopped -e PUID=1000 -e PGID=1000 -p 27017:27017 -v ./infra/data/mongodb/:/data/db mongo
sudo docker run -d --network gem --name postgres --restart unless-stopped -p 5432:5432 -v ./infra/data/postgres:/var/lib/postgresql/data -e POSTGRES_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70 -e POSTGRES_USER=conse -e PGDATA=/var/lib/postgresql/data/pgdata postgres
sudo docker run -d --link postgres --network gem --name adminer -p 7543:8080 adminer
sudo docker run -d --link mongodb --network gem --name mongo-express --restart always -p 7544:8081 -e ME_CONFIG_MONGODB_SERVER=mongodb mongo-express

sudo docker build -t conse-panel infra/docker/panel/
sudo docker run -d --link postgres --network gem --name conse-panel -p 7443:7442 conse-panel

sudo docker build -t catchup-bot infra/docker/bot/
sudo docker run -d --link redis --network gem --name catchup-bot -v ./infra/data/dis-bot-logs:/usr/src/app/logs/ catchup-bot

sudo docker build -t conse infra/docker/conse/
sudo docker run -d --link mongodb --network gem --name -p 7439:7438 conse conse

sudo docker run -d --link conse --link conse-panel --network gem --name haproxy --restart unless-stopped -p 443:443 -p 80:80 -p 8404:8404 -p 7440:7440 -p 7444:7444 -v ./infra/conf/haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg -v ./infra/cert/conse.pem:/usr/local/etc/conse.pem -e SERVER_IP=$SERVER_IP haproxytech/haproxy-alpine:2.4

MONGODB_CONTAINER_ID=$(docker container ls  | grep 'mongodb' | awk '{print $1}')
sudo docker cp infra/conse-collections/roles.json $MONGODB_CONTAINER_ID:/roles.json # root of the container
sudo docker cp infra/conse-collections/sides.json $MONGODB_CONTAINER_ID:/sides.json # root of the container 

sudo docker exec mongodb mongoimport --db conse --collection roles roles.json # roles.json is now inside the root of the mongodb container
sudo docker exec mongodb mongoimport --db conse --collection sides sides.json # sides.json is now inside the root of the mongodb container

sudo docker ps -a && sudo docker compose ps -a && sudo docker images