




sudo docker stop $(sudo docker ps -a -q) && sudo docker-compose down -v && sudo docker system prune -af --volumes
sudo docker-compose -f docker-compose.yml build --no-cache && sudo docker-compose up -d --force-recreate
sudo docker-compose -f docker-compose.yml logs -f
sudo docker stop $(sudo docker ps -a -q)                               --------------------------------> stop all running containers
sudo docker-compose -f docker-compose.yml build --no-cache             --------------------------------> update images, omit the --no-cache if you want to use cache building
sudo docker-compose down -v && sudo docker-compose up --force-recreate --------------------------------> remove and rebuild all containers, you will lose the old ones data
sudo docker-compose up -d --force-recreate                             --------------------------------> omit the --force-recreate if you don't want to recreate all the containers
sudo docker-compose -f docker-compose.yml logs -f                      --------------------------------> see the docker containers logs
sudo docker-compose run -u aranobi web bash                            --------------------------------> accessing bash shell of we service
sudo docker save $(sudo docker images -q) | gzip > avl.tar.gz
sudo docker load -i -o avl.tar
sudo docker network connect <NETWORK_NAME> <CONTAINER/IMAGE_NAME/ID>
sudo docker network create -o com.docker.network.bridge.enable_icc=true -d bridge <NETWORK_NAME>
sudo docker network ls
sudo docker network inspect -f '{{range .Containers}}{{.Name}} {{end}}' <NETWORK_NAME>
sudo docker-compose -f docker-compose.yml build --no-cache
sudo docker-compose up -d --force-recreate
sudo docker-compose -f docker-compose.yml logs -f
sudo docker-compose run -u aranobi <SERVICE_NAME_IN_DOCKER_COMPOSE> bash
sudo docker-compose restart <SERVICE_NAME_IN_DOCKER_COMPOSE>
sudo docker-compose down -v
sudo docker-compose -f docker-compose.yml up --build
sudo docker-compose exec db psql --username=avl --dbname=avl < avl.sql
sudo docker save $(sudo docker images -q) -o docker-utils/avl.tar
sudo docker load -i -o docker-utils/avl.tar
sudo docker ps
sudo docker exec <CONTAINER/IMAGE_NAME/ID>_A ping <CONTAINER/IMAGE_NAME/ID>_B -c2
sudo docker exec -it <CONTAINER/IMAGE_NAME/ID> bash
sudo docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $(sudo docker ps --format '{{.Names}}')
sudo docker inspect -f '{{index .Options "com.docker.network.bridge.enable_icc"}}' <NETWORK_NAME>
sudo docker build -t avl .
sudo docker run -it <IMAGE_NAME> /bin/bash
sudo docker run -d -it -p 8586:8586 avl --network=<NETWORK_NAME>
sudo docker images
sudo docker volume ls
sudo docker volume inspect <CHOOSE_ONE_FROM_ABOVE_COMMAND>
sudo docker commit <CONTAINER/IMAGE_NAME/ID> <NEW_IMAGE_NAME>
sudo docker stop <CONTAINER/IMAGE_NAME/ID>
sudo docker rmi -f <CONTAINER/IMAGE_NAME/ID>
sudo docker image prune -a
sudo docker system prune -a
sudo docker rmi -f $(sudo docker images -a -q)
sudo docker rmi -f $(sudo docker images -f "dangling=true" -q)
sudo docker rm -f $(sudo docker ps -aq)
sudo docker login --username=wildonion --password="password"
sudo docker commit <CONTAINER/IMAGE_NAME/ID> avl
sudo docker cp /home/wildonion/avl/  e4d47a395d07:/home/wildonion/
sudo docker cp 4ba0d2853dd2:/opt/avl/migrations /home/wildonion/utils/
