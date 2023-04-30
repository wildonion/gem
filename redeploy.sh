sudo docker stop conse
sudo docker rm -f conse
sudo docker build -t conse .
sudo docker run --network=gem -d --name conse
sudo docker stop bot
sudo docker rm -f bot
sudo docker build -t bot .
sudo docker run --network=gem -d bot