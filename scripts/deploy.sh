#!/bin/bash


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

echo "☕ Okay, sit back and drink your coffee :)"

sudo chmod 666 /var/run/docker.sock && docker system prune --all
export SERVER_IP=$(hostname -I | awk '{print $1}')
export PASSEORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70
sudo docker network create -d bridge gem || true

docker run -d \
  -h redis \
  -e REDIS_PASSWORD=$PASSEORD \
  -v ./infra/data/redis/:/data \
  -p 6379:6379 \
  --name redis \
  --network gem \
  --restart always \
  redis:5.0.5-alpine /bin/sh -c 'redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}'


sudo docker run -d --network gem --name mongodb --restart unless-stopped -e PUID=1000 -e PGID=1000 -p 27017:27017 -v ./infra/data/mongodb/:/data/db mongo
MONGODB_CONTAINER_ID=$(docker container ls  | grep 'mongodb' | awk '{print $1}')
sudo docker cp infra/conse-collections/roles.json $MONGODB_CONTAINER_ID:/roles.json # root of the container
sudo docker cp infra/conse-collections/sides.json $MONGODB_CONTAINER_ID:/sides.json # root of the container 
sudo docker exec mongodb mongoimport --db conse --collection roles roles.json # roles.json is now inside the root of the mongodb container
sudo docker exec mongodb mongoimport --db conse --collection sides sides.json # sides.json is now inside the root of the mongodb container

sudo docker run -d --network gem --name postgres --restart unless-stopped -p 5432:5432 -v ./infra/data/postgres:/var/lib/postgresql/data -e POSTGRES_PASSWORD=$PASSEORD -e POSTGRES_USER=postgres -e PGDATA=/var/lib/postgresql/data/pgdata postgres
sudo docker run -d --link postgres --network gem --name adminer -p 7543:8080 adminer
diesel setup && diesel migration run

sudo docker build -t conse-panel -f infra/docker/panel/Dockerfile . --no-cache
sudo docker run -d --link postgres --network gem --name conse-panel -p 7443:7442 conse-panel

sudo docker build -t conse-catchup-bot -f infra/docker/dis-bot/Dockerfile . --no-cache
sudo docker run -d --link redis --network gem --name conse-catchup-bot -v ./infra/data/dis-bot-logs:/usr/src/app/logs/ conse-catchup-bot

sudo docker build -t conse -f infra/docker/conse/Dockerfile . --no-cache
sudo docker run -d --link mongodb --network gem --name conse -p 7439:7438 conse

# If you use the host network mode for a container, 
# that container’s network stack is not isolated from the 
# Docker host (the container shares the host’s networking namespace), 
# and the container does not get its own IP-address allocated. 
# For instance, if you run a container which binds to port 80 and 
# you use host networking, the container’s application is available 
# on port 80 on the host’s IP address thus we use the host network 
# so we can access containers on 127.0.0.1 with their exposed ports 
# inside the nginx conf without their dns name or ip address. 
sudo docker build -t --no-cache nginx -f infra/docker/nginx/Dockerfile .
sudo docker stop nginx
sudo docker rm -f nginx
sudo docker run -d -it -p 80:80 -p 443:443 --name nginx --network host nginx

sudo docker ps -a && sudo docker compose ps -a && sudo docker images


jobs="/$USER/gem/jobs/*"
for f in $jobs
do
  crontab $f
done  
crontab -u root -l 
