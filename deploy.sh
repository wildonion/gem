#!/bin/bash
sudo rm .env && sudo mv .env.prod .env
sudo chmod 666 /var/run/docker.sock && docker system prune --all
export SERVER_IP=$(hostname -I | awk '{print $1}')
sudo docker network create -d bridge gem || true

docker run -d \
  -h redis \
  -e REDIS_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70 \
  -v ./infra/data/redis/:/data \
  -p 6379:6379 \
  --name redis \
  --restart always \
  redis:5.0.5-alpine /bin/sh -c 'redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}'


sudo docker run -d --network gem --name mongodb --restart unless-stopped -e PUID=1000 -e PGID=1000 -p 27017:27017 -v ./infra/data/mongodb/:/data/db mongo
MONGODB_CONTAINER_ID=$(docker container ls  | grep 'mongodb' | awk '{print $1}')
sudo docker cp infra/conse-collections/roles.json $MONGODB_CONTAINER_ID:/roles.json # root of the container
sudo docker cp infra/conse-collections/sides.json $MONGODB_CONTAINER_ID:/sides.json # root of the container 
sudo docker exec mongodb mongoimport --db conse --collection roles roles.json # roles.json is now inside the root of the mongodb container
sudo docker exec mongodb mongoimport --db conse --collection sides sides.json # sides.json is now inside the root of the mongodb container

sudo docker run -d --network gem --name postgres --restart unless-stopped -p 5432:5432 -v ./infra/data/postgres:/var/lib/postgresql/data -e POSTGRES_PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70 -e POSTGRES_USER=postgres -e PGDATA=/var/lib/postgresql/data/pgdata postgres
sudo docker run -d --link postgres --network gem --name adminer -p 7543:8080 adminer
diesel setup && diesel migration run

sudo docker build -t conse-panel -f infra/docker/panel/Dockerfile .
sudo docker run -d --link postgres --network gem --name conse-panel -p 7443:7442 conse-panel

sudo docker build -t conse-catchup-bot -f infra/docker/bot/Dockerfile .
sudo docker run -d --link redis --network gem --name conse-catchup-bot -v ./infra/data/dis-bot-logs:/usr/src/app/logs/ conse-catchup-bot

sudo docker build -t conse -f infra/docker/conse/Dockerfile .
sudo docker run -d --link mongodb --network gem --name conse -p 7439:7438 conse

sudo docker run -d --link conse --link conse-panel --network gem --name haproxy --restart unless-stopped -p 443:443 -p 80:80 -p 8404:8404 -p 7440:7440 -p 7444:7444 -v ./infra/conf/haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg -v ./infra/cert/conse.pem:/usr/local/etc/haproxy/conse.pem -e SERVER_IP=$SERVER_IP haproxytech/haproxy-alpine:2.4

sudo docker ps -a && sudo docker compose ps -a && sudo docker images