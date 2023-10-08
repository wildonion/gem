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
read INFURA_POLYGON_HTTPS_ENDPOINT
echo INFURA_POLYGON_HTTPS_ENDPOINT=$INFRA_POLYGON_HTTPS_ENDPOINT >> .env

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

    sudo docker stop mongodb && sudo docker rm -f mongodb
    sudo docker stop postgres && sudo docker rm -f postgres
    sudo docker stop adminer && sudo docker rm -f adminer
    sudo docker stop nginx && sudo docker rm -f nginx
    sudo docker stop redis && sudo docker rm -f redis
    sudo docker stop jenkins-docker && sudo docker rm -f jenkins-docker
    sudo docker stop jenkins-blueocean && sudo docker rm -f jenkins-blueocean
    sudo docker stop portainer && sudo docker rm -f portainer

    sudo docker run --name jenkins-docker --rm --detach \
    --privileged --network gem --network-alias docker \
    --env DOCKER_TLS_CERTDIR=/certs \
    --volume jenkins-docker-certs:/certs/client \
    --volume jenkins-data:/var/jenkins_home \
    --publish 2376:2376 \
    docker:dind --storage-driver overlay2

    sudo docker build -t jenkins-blueocean:lts -f $(pwd)/infra/docker/jenkins/Dockerfile . --no-cache

    sudo docker run --name jenkins-blueocean --restart=on-failure --detach \
    --network gem --env DOCKER_HOST=tcp://docker:2376 \
    --env DOCKER_CERT_PATH=/certs/client --env DOCKER_TLS_VERIFY=1 \
    --publish 8080:8080 --publish 50000:50000 \
    --volume jenkins-data:/var/jenkins_home \
    --volume jenkins-docker-certs:/certs/client:ro \
    jenkins-blueocean:lts

    echo "🚨 Please use `sudo docker logs -f jenkins-blueocean` or 
    `sudo docker exec jenkins-blueocean cat /var/jenkins_home/secrets/initialAdminPassword` 
    command to get the jenkins admin password!"

    docker volume create portainer_data
    docker run -d \
    -p 8000:8000 \
    -p 9443:9443 \
    --name portainer \
    --restart=always \
    --volume /var/run/docker.sock:/var/run/docker.sock \
    --volume portainer_data:/data \
    portainer/portainer-ce:latest

    sudo docker run -d \
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

    ANY_CONSE_PANEL_PG_CONTAINER_ID=$(docker container ls  | grep 'conse-panel-pg-*' | awk '{print $1}')
    ANY_CONSE_PANEL_MONGO_CONTAINER_ID=$(docker container ls  | grep 'conse-panel-mongo-*' | awk '{print $1}')
    ANY_CONSE_MAFIA_CONTAINER_ID=$(docker container ls  | grep 'conse-mafia-*' | awk '{print $1}')

    sudo docker stop $ANY_CONSE_PANEL_PG_CONTAINER_ID && sudo docker rm -f $ANY_CONSE_PANEL_PG_CONTAINER_ID
    sudo docker stop $ANY_CONSE_PANEL_MONGO_CONTAINER_ID && sudo docker rm -f $ANY_CONSE_PANEL_MONGO_CONTAINER_ID
    sudo docker stop $ANY_CONSE_MAFIA_CONTAINER_ID && sudo docker rm -f $ANY_CONSE_MAFIA_CONTAINER_ID

    TIMESTAMP=$(date +%s)

    sudo docker build -t conse-mafia-$TIMESTAMP -f $(pwd)/infra/docker/mafia/Dockerfile . --no-cache
    sudo docker run -d --restart unless-stopped --link mongodb --network gem --name conse-mafia-$TIMESTAMP -p 7439:7438 conse-mafia-$TIMESTAMP
    
    echo \t"🪣 Which Db Storage You Want To Use for Conse Panel Service? [postgres/mongodb] > "
    read CONSE_PANEL_DB_STORAGE

    if [[ $CONSE_PANEL_DB_STORAGE == "postgres" ]]; then
        echo \n"> 🛢 Building Conse Panel With postgres Db Storage"
        sudo docker build -t conse-panel-pg-$TIMESTAMP -f $(pwd)/infra/docker/panel/postgres/Dockerfile . --no-cache
        sudo docker run -d --restart unless-stopped --link postgres --network gem --name conse-panel-pg-$TIMESTAMP -p 7443:7442 -v $(pwd)/assets/:/app/assets -v $(pwd)/infra/logs/:/app/logs conse-panel-pg-$TIMESTAMP
    else
        echo \n"> 🛢 Building Conse Panel With mongo Db Storage"
        echo \t"--[make sure you're matching over storage.clone().unwrap().get_mongodb() in your code]--"
        sudo docker build -t conse-panel-mongo-$TIMESTAMP -f $(pwd)/infra/docker/panel/mongodb/Dockerfile . --no-cache
        sudo docker run -d --restart unless-stopped --link postgres --network gem --name conse-panel-mongo-$TIMESTAMP -p 7444:7442 -v $(pwd)/assets/:/app/assets  -v $(pwd)/infra/logs/:/app/logs conse-panel-mongo-$TIMESTAMP
    fi

fi
