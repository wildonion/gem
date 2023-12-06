#!/bin/bash
TIMESTAMP=$(date +%s)
CONSE_CONTAINER_ID=$(docker container ls  | grep 'conse-panel-pg' | awk '{print $1}')
sudo docker build -t conse-panel-pg-$TIMESTAMP -f $(pwd)/infra/docker/panel/postgres/Dockerfile . --no-cache
sudo docker stop $CONSE_CONTAINER_ID
sudo docker run -d --restart unless-stopped --link postgres --network gem --name conse-panel-pg-$TIMESTAMP -p 7443:7442 -v $(pwd)/assets/:/app/assets -v $(pwd)/infra/logs/:/app/logs conse-panel-pg-$TIMESTAMP
sudo docker rm $CONSE_CONTAINER_ID
