SERVER_IP=$(hostname -I | awk '{print $1}')
sudo docker compose -f docker-compose.yml build --no-cache
sudo docker compose up -d --force-recreate