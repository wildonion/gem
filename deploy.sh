#!/bin/bash
SERVER_IP=$(hostname -I | awk '{print $1}')
sudo docker compose -f docker-compose.yml build --no-cache && sudo docker compose up -d --force-recreate
sudo docker inspect -f '{{.Name}} - {{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $(docker ps -aq)
MONGODB_CONTAINER_ID=$(docker container ls  | grep 'mongodb' | awk '{print $1}')
sudo docker cp devops/conse-collections/roles.json $MONGODB_CONTAINER_ID:/roles.json # root of the container
sudo docker cp devops/conse-collections/sides.json $MONGODB_CONTAINER_ID:/sides.json # root of the container 
sudo docker exec mongodb mongoimport --db conse --collection roles roles.json # roles.json is now inside the root of the mongodb container
sudo docker exec mongodb mongoimport --db conse --collection sides sides.json # sides.json is now inside the root of the mongodb container
sudo docker ps -a && sudo docker compose ps -a && sudo docker images