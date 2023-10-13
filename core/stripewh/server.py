#! /usr/bin/env python3.6

import requests
import stripe
import json
import os

from flask import Flask, render_template, jsonify, request, send_from_directory, redirect
from dotenv import load_dotenv, find_dotenv

# Setup Stripe python client library.
load_dotenv(find_dotenv())


stripe.api_version = '2020-08-27'
stripe.api_key = os.getenv('STRIPE_TEST_SECRET_KEY')
app = Flask(__name__, static_url_path="")


'''
stripe event handler and webhook subscriber to the checkout payment events 
webhook means once an event gets triggered an api call will be invoked to 
notify (it's like a notification to the server) server about the event happend 
as a result of handling another process in some where like a payment result in 
which server subscribes to incoming event type and can publish it to redispubsub 
so other app, threads and scopes can also subscribe to it 
receiving async stripe payment events, remember to register a webhook
in stripe dashboard so stripe knows where to deliver events
'''

@app.route('/webhook', methods=['POST'])
def webhook_received():
    # You can use webhooks to receive information about asynchronous payment events.
    # For more about our webhook events check out https://stripe.com/docs/webhooks.
    webhook_secret = os.getenv('STRIPE_WEBHOOK_SIGNATURE')
    panel_webhook_url = os.getenv('STRIPE_PANEL_UPDATE_BALANCE_WEBHOOK_URL')
    request_data = json.loads(request.data)

    if webhook_secret:
        # Retrieve the event by verifying the signature using the raw body and secret if webhook signing is configured.
        signature = request.headers.get('stripe-signature')
        try:
            event = stripe.Webhook.construct_event(
                payload=request.data, sig_header=signature, secret=webhook_secret)
            data = event['data']
        except Exception as e:
            return e
        # Get the type of webhook event sent - used to check the status of PaymentIntents.
        event_type = event['type']
    else:
        data = request_data['data']
        event_type = request_data['type']
    data_object = data['object']

    print('event ' + event_type)

    if event_type == 'checkout.session.completed':
                
        print('🔔 Payment succeeded!')
        
        session_id = data_object["id"]
        payment_intent = data_object["payment_intent"]
        
        panel_docker = os.getenv('PANEL_DOCKER')
        url = f"{panel_docker}:7443/{panel_webhook_url}/{session_id}/{payment_intent}"

        headers = {
            "stripe-signature": webhook_secret,
        }
        
        response = requests.post(url, headers=headers)

        print('🔔 Update user balance webhook response!', response.json())
        
    if event_type == 'checkout.session.checkout.session.async_payment_succeeded':
        print('🔔 Async Payment Success!')
        
    if event_type == 'checkout.session.expired':
        print('🔔 Payment expired!')
    
    if event_type == 'checkout.session.async_payment_failed':
        print('🔔 Async Payment failed!')

    return jsonify({'status': 'success'})

if __name__ == '__main__':
    app.run(port=4242, debug=True)
