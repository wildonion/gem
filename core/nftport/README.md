


## 🌋 Deploy

```bash
sudo docker build -t nftport -f $(pwd)/infra/docker/nftport/Dockerfile . --no-cache
sudo docker run -d --restart unless-stopped --network gem --name nftport -p 7651:7650 nftport
```