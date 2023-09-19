


from fastapi import FastAPI
import requests


app = FastAPI()


@app.post("/upload/{access_token}")
def upload(access_token):

    file = open("card.png", "rb")

    response = requests.post(
        "https://api.nftport.xyz/v0/files",
        headers={"Authorization": access_token},
        files={"file": file}
    )
    
    print("response coming from nftport: ", response.json())
    
    return {"res": response.json}
