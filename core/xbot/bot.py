from fastapi import FastAPI
from pydantic import BaseModel
from typing import Union
from fastapi import Depends, FastAPI, HTTPException
from typing import List, Dict
from sqlalchemy.orm import Session
import utils

app = FastAPI()

class UserExistance(BaseModel):
    username: str

class UserVerification(BaseModel):
    username: str
    code: str # this can be an intiger as well
    
    
class Collector(BaseModel):
    username: str
    type: str
    tweet_id: Union[str, None] = None
    main_account: Union[str, None] = None
    hashtag: Union[str, None] = None
    text: Union[str, None] = None
    twitter_url: Union[str, None] = None
    
class ViewsInfo(BaseModel):
    tweet_id: str
    username: str
    
@app.post("/user-existance/{key}")
def check_user_existance(key: str, request: UserExistance):
    if key != "{8eN~PF=xyqz0s^":
        return {"data":  {"status": "key is not valid"}}
    print(":::::: {} requested for verification: ", request.username)
    result = utils.user_exist(username=request.username)
    if result == "403":
        raise HTTPException(status_code=403, detail="too many request")
    return {"data": {"status": result}}
    # return { "data": {"status": user_existance(request.username)} }


@app.post("/user-verification/{key}")
def user_verification(key: str, request: UserVerification):
    if key != "{8eN~PF=xyqz0s^":
        return {"data":  {"status": "key is not valid"}}
    result = utils.user_verify(username=request.username, code=request.code)
    if result == "403":
        raise HTTPException(status_code=403, detail="too many request")
    return { "data": { "status": result}}


@app.get("/test/likes/of/{key}/{username}")
def test_like(username: str, key: str):
    if key != "j018WdNq3J26":
        return {"data":  {"status": "key is not valid"}}
    return utils.test_like(str(username))

@app.get("/test/tweet/of/{key}/{username}")
def test_tweet(username: str, key: str):
    if key != "j018WdNq3J26":
        return {"data":  {"status": "key is not valid"}}
    return utils.test_tweet(str(username))

@app.get("/get-ratelimit-info")
def get_rl_info(ratelimit_info: List[Dict[str, any]] = Depends(utils.get_ratelimit_info)):
    return ratelimit_info

@app.get("/get-app-ratelimit-info")
def get_app_rl_info(app_ratelimit_info: List[Dict[str, any]] = Depends(utils.get_app_ratelimit_info)):
    return app_ratelimit_info

@app.post("/check/{key}")
def check(key: str, request: Collector):
    if key != "j018WdNq3J26":
        return {"data":  {"status": "key is not valid"}}
    if request.type == "hashtag":
        print(f"type: {request.type}, hashtag: {request.hashtag}, from user: {request.username}")
        result = utils.scrape_hashtag(username=request.username, hashtag=request.hashtag)
        if result == "403":
            raise HTTPException(status_code=403, detail="too many request")
        return {"data": { "status": result}}
    elif request.type == "like":
        print(f"type: {request.type}, tweet id: {request.tweet_id}, from user: {request.username}")
        result = utils.scrape_like(username=request.username, tweet_id=request.tweet_id)
        if result == "403":
            raise HTTPException(status_code=403, detail="too many request")
        return { "data": { "status": result}}
    elif request.type == "retweet":
        if request.main_account == None or request.main_account == "string":
            result = utils.scrape_retweet(username=request.username, tweet_id=request.tweet_id)
        else:     
            result = utils.scrape_retweet(username=request.username, tweet_id=request.tweet_id)
        if result == "403":
            raise HTTPException(status_code=403, detail="too many request")
        return { "data": {"status": result}}
    elif request.type == "tweet":
        result = utils.scrape_tweet(username=request.username, text=request.text)
        if result == "403":
            raise HTTPException(status_code=403, detail="too many request")
        return {"data": {"status": result}}
    else:
        return {"data":  {"status": "type is not valid"}}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=4245)