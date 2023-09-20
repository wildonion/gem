#!/bin/bash

cd ..

sudo rm .env && sudo mv .env.prod .env
sudo mv twitter-accounts.prod.json twitter-accounts.json
echo \t">>> Please fill up the 'twitter-accounts.json' without your twitter dev account keys"
echo \t"using the conse panel API with admin access"

echo "[?] Enter SMS API Token: "
read OTP_API_TOKEN
echo OTP_API_TOKEN=$OTP_API_TOKEN >> .env

echo "[?] Enter SMS API Template: "
read OTP_API_TEMPLATE
echo OTP_API_TEMPLATE=$OTP_API_TEMPLATE >> .env

echo "[?] Enter Messagebird Access Key: "
read MESSAGEBIRD_ACCESS_KEY
echo MESSAGEBIRD_ACCESS_KEY=$MESSAGEBIRD_ACCESS_KEY >> .env

echo "[?] Enter Ip Info Access Key: "
read IPINFO_TOKEN
echo IPINFO_TOKEN=$IPINFO_TOKEN >> .env

echo "[?] Enter Currency Layer Token: "
read CURRENCY_LAYER_TOKEN
echo CURRENCY_LAYER_TOKEN=$CURRENCY_LAYER_TOKEN >> .env

echo "[?] Enter PayPal Token: "
read PAYPAL_TOKEN
echo PAYPAL_TOKEN=$PAYPAL_TOKEN >> .env

echo "[?] Enter Nft Port Token: "
read NFTYPORT_TOKEN
echo NFTYPORT_TOKEN=$NFTYPORT_TOKEN >> .env

echo "[?] Infra WS Polygon: "
read INFRA_POLYGON_WS_ENDPOINT
echo INFRA_POLYGON_WS_ENDPOINT=$INFRA_POLYGON_WS_ENDPOINT >> .env

echo "[?] Infra HTTPS Polygon: "
read INFRA_POLYGON_HTTPS_ENDPOINT
echo INFRA_POLYGON_HTTPS_ENDPOINT=$INFRA_POLYGON_HTTPS_ENDPOINT >> .env

echo "[?] Enter SMTP Username: "
read SMTP_USERNAME
echo SMTP_USERNAME=$SMTP_USERNAME >> .env

echo "[?] Enter SMTP App Password: "
read SMTP_PASSWORD
echo SMTP_PASSWORD=$SMTP_PASSWORD >> .env

echo "[?] Enter SMTP Server: "
read SMTP_SERVER
echo SMTP_SERVER=$SMTP_SERVER >> .env

echo "[?] Enter Machine Id: "
read MACHINE_ID
echo MACHINE_ID=$MACHINE_ID >> .env
echo "[?] Enter Node Id: "
read NODE_ID
echo NODE_ID=$NODE_ID >> .env

sudo chmod 666 /var/run/docker.sock
export SERVER_IP=$(hostname -I | awk '{print $1}')
export PASSWORD=geDteDd0Ltg2135FJYQ6rjNYHYkGQa70

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
    -e REDIS_PASSWORD=$PASSWORD \
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
    sudo docker cp $(pwd)/infra/mafia-collections/last_moves.json $MONGODB_CONTAINER_ID:/last_moves.json # root of the container 
    sudo docker exec mongodb mongoimport --db conse --collection roles roles.json # roles.json is now inside the root of the mongodb container
    sudo docker exec mongodb mongoimport --db conse --collection users users.json # users.json is now inside the root of the mongodb container
    sudo docker exec mongodb mongoimport --db conse --collection sides sides.json # sides.json is now inside the root of the mongodb container
    sudo docker exec mongodb mongoimport --db conse --collection last_moves last_moves.json # last_moves.json is now inside the root of the mongodb container

    sudo docker run -d --network gem --name postgres --restart unless-stopped -p 5432:5432 -v $(pwd)/infra/data/postgres/:/var/lib/postgresql/data -e POSTGRES_PASSWORD=$PASSWORD -e POSTGRES_USER=postgres -e PGDATA=/var/lib/postgresql/data/pgdata postgres
    sudo docker run -d --link postgres --network gem --name adminer -p 7543:8080 adminer
    diesel setup && diesel migration run
    sqlant postgresql://postgres:$PASSWORD@localhost/conse > $(pwd)/infra/panel.uml
    java -jar $(pwd)/infra/plantuml.jar $(pwd)/infra/panel.uml

    jobs="jobs/*"
    for f in $jobs
    do
        crontab $f
    done  
    crontab -u root -l 

    sudo docker ps -a && sudo docker compose ps -a && sudo docker images
  
else
    echo "> Redeploying Rust Services Only"\n

    docker stop conse-panel-pg && docker rm -f conse-panel-pg
    docker stop conse-panel-mongo && docker rm -f conse-panel-mongo
    docker stop conse-mafia && docker rm -f conse-mafia
    docker stop nftport && docker rm -f nftport

    sudo docker build -t nftport -f $(pwd)/infra/docker/nftport/Dockerfile . --no-cache
    sudo docker run -d --restart unless-stopped --network gem --name nftport -p 7651:7650 nftport

    sudo docker build -t conse-mafia -f $(pwd)/infra/docker/mafia/Dockerfile . --no-cache
    sudo docker run -d --restart unless-stopped --link mongodb --network gem --name conse-mafia -p 7439:7438 conse-mafia
    
    echo \t"🪣 Which Db Storage You Want To Use for Conse Panel Service? [postgres/mongodb] > "
    read CONSE_PANEL_DB_STORAGE

    if [[ $CONSE_PANEL_DB_STORAGE == "postgres" ]]; then
        echo \n"> 🛢 Building Conse Panel With postgres Db Storage"
        sudo docker build -t conse-panel-pg -f $(pwd)/infra/docker/panel/postgres/Dockerfile . --no-cache
        sudo docker run -d --restart unless-stopped --link postgres --network gem --name conse-panel-pg -p 7443:7442 -v $(pwd)/infra/assets/:/usr/src/app/assets -v $(pwd)/infra/logs/:/usr/src/app/logs conse-panel-pg
    else
        echo \n"> 🛢 Building Conse Panel With mongo Db Storage"
        echo \t"--[make sure you're matching over storage.clone().unwrap().get_mongodb() in your code]--"
        sudo docker build -t conse-panel-mongo -f $(pwd)/infra/docker/panel/mongodb/Dockerfile . --no-cache
        sudo docker run -d --restart unless-stopped --link postgres --network gem --name conse-panel-mongo -p 7444:7442 -v $(pwd)/infra/assets/:/usr/src/app/assets  -v $(pwd)/infra/logs/:/usr/src/app/logs conse-panel-mongo
    fi

fi
