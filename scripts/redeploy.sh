#!/bin/bash

cd ..

sudo chmod 666 /var/run/docker.sock && sudo docker system prune --all
export SERVER_IP=$(hostname -I | awk '{print $1}')
export PASSEORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70

echo "[?] Wanna Redeploy Infrastructure? "
read REDPLOY_INFRASTRUCTURE

if [[ $REDPLOY_INFRASTRUCTURE == "Y" || $REDPLOY_INFRASTRUCTURE == "y" ]]; then

    echo "> Redeploying Infrastructure Only"
    echo "☕ Okay, sit back and drink your coffee :)"

    docker stop mongodb && docker rm -f mongodb
    docker stop postgres && docker rm -f postgres
    docker stop adminer && docker rm -f adminer
    docker stop nginx && docker rm -f nginx
    docker stop redis && docker rm -f redis

    docker run -d \
    -h redis \
    -e REDIS_PASSWORD=$PASSEORD \
    -v $(pwd)/infra/data/redis/:/data \
    -p 6379:6379 \
    --name redis \
    --network gem \
    --restart always \
    redis:latest /bin/sh -c 'redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}'


    sudo docker run -d --network gem --name mongodb --restart unless-stopped -e PUID=1000 -e PGID=1000 -p 27017:27017 -v $(pwd)/infra/data/mongodb/:/data/db mongo
    MONGODB_CONTAINER_ID=$(docker container ls  | grep 'mongodb' | awk '{print $1}')
    sudo docker cp $(pwd)/infra/mafia-collections/roles.json $MONGODB_CONTAINER_ID:/roles.json # root of the container
    sudo docker cp $(pwd)/infra/mafia-collections/sides.json $MONGODB_CONTAINER_ID:/sides.json # root of the container 
    sudo docker cp $(pwd)/infra/mafia-collections/users.json $MONGODB_CONTAINER_ID:/users.json # root of the container 
    sudo docker exec mongodb mongoimport --db conse --collection roles roles.json # roles.json is now inside the root of the mongodb container
    sudo docker exec mongodb mongoimport --db conse --collection users users.json # users.json is now inside the root of the mongodb container
    sudo docker exec mongodb mongoimport --db conse --collection sides sides.json # sides.json is now inside the root of the mongodb container

    sudo docker run -d --network gem --name postgres --restart unless-stopped -p 5432:5432 -v $(pwd)/infra/data/postgres/:/var/lib/postgresql/data -e POSTGRES_PASSWORD=$PASSEORD -e POSTGRES_USER=postgres -e PGDATA=/var/lib/postgresql/data/pgdata postgres
    sudo docker run -d --link postgres --network gem --name adminer -p 7543:8080 adminer
    diesel setup && diesel migration run
    sqlant postgresql://postgres:$PASSEORD@localhost/conse > $(pwd)/infra/panel.uml
    java -jar $(pwd)/infra/plantuml.jar $(pwd)/infra/panel.uml

    # If you use the host network mode for a container, 
    # that container’s network stack is not isolated from the 
    # Docker host (the container shares the host’s networking namespace), 
    # and the container does not get its own IP-address allocated. 
    # For instance, if you run a container which binds to port 80 and 
    # you use host networking, the container’s application is available 
    # on port 80 on the host’s IP address thus we use the host network 
    # so we can access containers on 127.0.0.1 with their exposed ports 
    # inside the nginx conf without their dns name or ip address. 
    sudo docker stop nginx
    sudo docker rm -f nginx
    sudo docker build -t --no-cache nginx -f $(pwd)/infra/docker/nginx/Dockerfile .
    sudo docker run -d -it -p 80:80 -v $(pwd)/infra/data/nginx/confs/:/etc/nginx -v $(pwd)/infra/data/nginx/wwws/:/usr/share/nginx/ -p 443:443 --name nginx --network host nginx

    jobs="jobs/*"
    for f in $jobs
    do
        crontab $f
    done  
    crontab -u root -l 

    sudo docker ps -a && sudo docker compose ps -a && sudo docker images
  
else
    echo "> Redeploying Rust Services Only"
    echo "☕ Okay, sit back and drink your coffee :)"

    docker stop conse-panel && docker rm -f conse-panel
    docker stop conse-catchup-bot && docker rm -f conse-catchup-bot
    docker stop conse-mafia && docker rm -f conse-mafia
    docker stop twiscord && docker rm -f twiscord

    sudo docker build -t conse-panel -f $(pwd)/infra/docker/panel/Dockerfile . --no-cache
    sudo docker run -d --link postgres --network gem --name conse-panel -p 7443:7442 conse-panel

    sudo docker build -t conse-catchup-bot -f $(pwd)/infra/docker/catchup-bot/Dockerfile . --no-cache
    sudo docker run -d --link redis --network gem --name conse-catchup-bot -v $(pwd)/infra/data/catchup-bot-logs/:/usr/src/app/logs/ conse-catchup-bot

    sudo docker build -t twiscord -f $(pwd)/infra/docker/twiscord/Dockerfile . --no-cache
    sudo docker run -d --link redis --network gem --name twiscord -v $(pwd)/infra/data/twiscord-logs/:/usr/src/app/logs/ twiscord

    sudo docker build -t conse-mafia -f $(pwd)/infra/docker/mafia/Dockerfile . --no-cache
    sudo docker run -d --link mongodb --network gem --name conse-mafia -p 7439:7438 conse-mafia
fi
