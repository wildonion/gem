

twitter bot to verify tasks

## Supported Tasks

- likes

- comments

- retweet

- tweet

## How 2?

step 0. fill the twitter keys inside the code

step 1. deploy the docker container of `xbot` server

step 2. setup DNS record for `https://api.xbot.conse.app` in DNS panel to points where the xbot code is hosted

step 3. configure nginx reverse proxy for above subdomain to point to the docker container on the VPS

step 4. register new SSL for the above subdomain suing ```certbot certonly --standalone -d <SUBDOMAIN>``` command, or you can run `./renew.sh` script to make this work for you