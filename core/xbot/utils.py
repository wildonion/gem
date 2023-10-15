import tweepy
from datetime import datetime, timezone
import requests
import time


clients = []


client_id = ""
client_secret = ""
bearer_token = ""
access_token = ""
access_token_secret = ""
api_secret = ""
api_key = ""

client1 = tweepy.Client(
    bearer_token, 
    api_key, 
    api_secret, 
    access_token, 
    access_token_secret,
    return_type=requests.Response
)

clients.append(client1)



client_id = ""
client_secret = ""
bearer_token = ""
access_token = ""
access_token_secret = ""
api_secret = ""
api_key = ""


client2 = tweepy.Client(
    bearer_token, 
    api_key, 
    api_secret, 
    access_token, 
    access_token_secret,
    return_type=requests.Response
)

clients.append(client2)
from typing import List, Dict


class RateLimitInfo:
    data: List[Dict[str, any]] = []

def get_ratelimit_info() -> List[Dict[str, any]]:
    return RateLimitInfo.data


class AppRateLimitInfo:
    data: List[Dict[str, any]] = [{}, {}]

def get_app_ratelimit_info() -> List[Dict[str, any]]:
    return AppRateLimitInfo.data

  
        
def user_exist(username):
    try:
        # account in here is client
        
        for idx in range(len(clients)):
            try:
                resp = clients[idx].get_user(username=username, user_fields=['created_at', 'public_metrics'])
                headers = resp.headers
                result = resp.json().get('data')
                
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_user',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }
                
                account_creation_date = result["created_at"]
                account_creation_date = datetime.strptime(account_creation_date, "%Y-%m-%dT%H:%M:%S.%fZ")
                now = datetime.utcnow()
                dif_time = now - account_creation_date
                if dif_time.days > 7:
                    if result["public_metrics"]["followers_count"] > 10:
                        return True 
                return False
            except tweepy.errors.TooManyRequests:
                continue
        print("time is up")
        return "403"
    except:
        print("User Doesnt exist")
        return False

def user_verify(username, code):
    try:
        # account in here is client
        
        for idx in range(len(clients)):
            try:
                
                resp = clients[idx].get_user(username=username)
                headers = resp.headers
                user = resp.json().get('data')['id']
                
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_user',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }
            
                
                resp1 = clients[idx].get_users_tweets(id=int(user))
                headers = resp1.headers
                response = resp1.json().get('data')
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_users_tweets',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }
                
                for tweet in response:
                    if code in tweet['text']:
                            return True
                return False
            except tweepy.errors.TooManyRequests:
                continue
        print("time is up")
        return "403"
    except:
        print("User Doesnt exist")
        return False

def scrape_hashtag(username, hashtag):
    try:
        # account in here is client
        
        for idx in range(len(clients)):
            try:
                
                
                resp = clients[idx].get_user(username=username)
                headers = resp.headers
                user = resp.json().get('data')['id']
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_user',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }
                
                resp1 = clients[idx].get_users_tweets(id=int(user))
                headers = resp1.headers
                response = resp1.json().get('data')
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_users_tweets',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }

                for tweet in response:
                    if hashtag in tweet['text']:
                        return True
                return False
            except tweepy.errors.TooManyRequests:
                continue
        print("time is up")
        return "403"
        
    except:
        print("Hashtag Doesnt exist")
        return False

def scrape_like(username, tweet_id):
    try:

        for idx in range(len(clients)):
            try:
                
                
                resp = clients[idx].get_user(username=username)
                headers = resp.headers
                user = resp.json().get('data')['id']
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_user',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }

                
                resp1 = clients[idx].get_liked_tweets(id=int(user))
                headers = resp1.headers
                user_likings = resp1.json().get('data')
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_liked_tweets',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }

                
                for likes in user_likings:
                    if int(likes['id']) == int(tweet_id):
                        return True
                return False
            except tweepy.errors.TooManyRequests:
                continue
        print("time is up")
        return "403"
    except:
        print("didnt found any user")
        #either user didnt found or list of users who liked are empty
        return False

def scrape_retweet(username, tweet_id, main_account="ConseGemNFT"):
    try:
        for idx in range(len(clients)):
            try:
                
                resp = clients[idx].get_user(username=username)
                headers = resp.headers
                user = resp.json().get('data')['id']
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_user',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }

                
                resp1 = clients[idx].get_users_tweets(id=int(user))
                headers = resp1.headers
                tweets = resp1.json().get('data')
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_users_tweets',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }
                
                resp2 = clients[idx].get_tweet(tweet_id)
                headers = resp2.headers
                tweet = resp2.json().get('data')
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_tweet',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }

                    
                tweet_text = f"RT @{main_account}: {tweet['text']}"
                for tweet in tweets:
                    
                    if str(tweet['text']).startswith(f"RT @{main_account}"):
                        slice_user_tweet_text = tweet['text'].replace("…", "")
                        user_tweet_len = len(slice_user_tweet_text)
                        slice_tweet_text = tweet_text[:user_tweet_len]
                        print("current user twee >>>> ", slice_user_tweet_text)
                        print("must be retweeted >>>> ", slice_tweet_text)
                        print("==================================")
                        if slice_user_tweet_text == slice_tweet_text:
                            return True
                return False
            except tweepy.errors.TooManyRequests:
                continue
        print("time is up")
        return "403"
    
    except:
        print("error")
        return False


def scrape_tweet(username, text):
    try:
        
        print("inside scrape tweet")
        for idx in range(len(clients)):
            try:
                
                resp = clients[idx].get_user(username=username)
                headers = resp.headers
                user = resp.json().get('data')['id']
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_user',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }

                
                resp1 = clients[idx].get_users_tweets(id=int(user))
                headers = resp1.headers
                tweets = resp1.json().get('data')
                RateLimitInfo.data.append(
                    {
                        'username': username,
                        'route': 'get_users_tweets',
                        'rl_info': {
                            'bot': str(idx+1),
                            'x_rate_limit_remaining': headers.get('x-rate-limit-remaining'),
                            'x_rate_limit_limit': headers.get('x-rate-limit-limit'),
                            'x_rate_limit_reset': headers.get('x-rate-limit-reset'),
                            'x_app_limit_24hour_limit': headers.get('x-app-limit-24hour-limit'),
                            'x_app_limit_24hour_reset': headers.get('x-app-limit-24hour-reset'),
                            'x_app_limit_24hour_remaining': headers.get('x-app-limit-24hour-remaining')
                        },
                        'request_at': str(int(time.time()))
                        
                    }
                )
                
                AppRateLimitInfo.data[idx] = {
                    "bot": str(idx+1),
                    "x_app_limit_24hour_remaining": headers.get('x-app-limit-24hour-remaining'),
                    "x_app_limit_24hour_reset": headers.get('x-app-limit-24hour-reset'),
                }
                

                for tweet in tweets:
                    print("must contains in tweet >>> ", text)
                    print("user tweet >>>>", tweet['text'])
                    
                    if text in tweet['text']:
                        return True
                return False
            except tweepy.errors.TooManyRequests:
                continue
        print("time is up")
        return "403"
        
    except:
        print("error")
        return False