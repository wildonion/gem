


# Deploy

```bash
sudo docker build -t thirdweb -f $(pwd)/infra/docker/thirdweb/Dockerfile . --no-cache
sudo docker run -d --restart unless-stopped --network gem --name thirdweb -p 7651:7650 thirdweb
```