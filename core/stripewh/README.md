

## How 2? 

> note that live (test) webhooks only work in a live (test) environment, so make sure you're creating a webhook based on your environment which can be switched in stripe dashboard panel.

step 0. fill the `STRIPE_*` vars inside the `.env` with your live (test) keys

step 1. deploy the docker container of `stripewh` server

step 2. setup DNS record for `https://api.panel.stripewh.conse.app` in DNS panel

step 3. configure nginx reverse proxy for above subdomain to point to the docker container on the VPS

step 4. register new SSL for the above subdomain suing ```sudo certbo --nginx``` command, or you can run `./renew.sh` script to make this work for you

step 5. go to https://dashboard.stripe.com/webhooks/create?endpoint_location=hosted and create a webhook with endpoint `https://api.panel.stripewh.conse.app/webhook` to register checkout webhook events to get the stripe webhook secret, by setting up this webhook, all the stripes events will deliver to this endpoint.

step 6. update the `STRIPE_WEBHOOK_SIGNATURE` field inside the `.env` file with webhook secret.

## Can be easier?

Yes! jut run the following command in a new terminal to get the webhook secret and start the stripe webhook listener to accept incoming webhook events like checkouts in localhost, make sure you have already setup the docker container for the `stripewh` server and is accepting connection on port `4243`, also remember to update the `STRIPE_WEBHOOK_SIGNATURE` field inside the `.env` file.

```bash
cd .. && cd .. && cd scripts
sudo chmod +x stripe.sh
stripe login && sudo pm2 start --name stripe-whebook-listener
```

see the stripe webhook listener logs by running the `sudo pm2 log stripe-whebook-listener` command.