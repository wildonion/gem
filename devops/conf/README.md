


## Setup Ayoub APIs Reverse Proxy

### Install Nginx

```console
sudo apt install nginx && sudo apt install certbot python3-certbot-nginx
```

### Available Service Config File 

```console
sudo cp api /etc/nginx/sites-available/api
```

### Enable Service Config File 

```console
sudo ln -s /etc/nginx/sites-available/api /etc/nginx/sites-enabled/api
```

## Setup SSL for APIs

```console
sudo systemctl restart nginx && sudo certbot --nginx
```

## NOTE

> Remember to enable `ufw` and allow all in/out going requests through the ayoub port using `sudo ufw allow 7439`, `sudo ufw allow 80` and `sudo ufw 443` commands.

> Don't use *.conf to configure the nginx for the backend code since in backend the we've used `middlewares::cors::allow` method to handle the CORS issue.  