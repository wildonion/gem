


Before deploying the docker container:

## Live production 

> after setting up a DNS record for `https://api.panel.stripewh.conse.app` go to https://dashboard.stripe.com/webhooks/create?endpoint_location=hosted and create a webhook with endpoint `https://api.panel.stripewh.conse.app/webhook` to register checkout webhook events to get the stripe webhook secret, by setting up this webhook, all the stripes events will deliver to this endpoint.

## Test Development 

> run the following command in a new terminal to get the webhook secret and start listening on incoming stripe webhook events like checkouts in localhost.

```bash
cd .. && cd .. && cd scripts
sudo chmod +x stripe.sh
stripe login && sudo pm2 start --name stripe-whebook-listener
```

## Run

> start the webhook server to receive stripe events on checkout payment process events.

```bash
python3 -m flask run --host=0.0.0.0 --port=4242
```

## Notes

* live (test) webhooks only work in a live (test) environment.

* we can also create an **https** webhook endpoint in test mode in stripe dashboard instead of loading a local webhook listener.

* see the stripe webhook listener logs by running the `sudo pm2 log stripe-whebook-listener` command.

* note that in both ways you have to update the `STRIPE_WEBHOOK_SIGNATURE` field inside the `.env` file.

* also make sure that the webhook endpoint in stripe dashboard panel is secured and SSL-ed.