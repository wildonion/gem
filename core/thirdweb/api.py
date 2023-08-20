

# pip install thirdweb-sdk

from thirdweb import ThirdwebSDK
from thirdweb.types.nft import NFTMetadataInput
from thirdweb.types import SDKOptions
from fastapi import FastAPI

app = FastAPI()



@app.post("/mint/to/{mint_to}/{amount}")
def mint_nft(mint_to, amount):
    
    secret_key = ""
    private_key = ""
    contract = "0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6"

    sdk = ThirdwebSDK.from_private_key(private_key, "mumbai", SDKOptions(secret_key=secret_key))
    contract = sdk.get_contract(contract)

    # Note that you can customize this metadata however you like
    metadata = NFTMetadataInput.from_json({
        "name": f"{amount} MT YouWho NFT",
        "description": "YouWho NFT Card",
        "image": open("card.png", "rb"),
    })

    # ---------------------------------------------
    # -------------- minting process --------------
    # ---------------------------------------------
    try:
        tx = contract.erc721.mint_to(mint_to, metadata)
    except:
        tx = None
    if tx != None:
        receipt = dict(tx.receipt)
        tx_hash = receipt['transactionHash'].hex()
        nft_id = str(tx.data().metadata.id)
        mint_tx_hash = tx_hash
        return {"mint_tx_hash": mint_tx_hash, "token_id": str(nft_id)}
    else:
        return {"mint_tx_hash": "", "token_id": ""} 


@app.post("/burn/{token_id}")
def burn_nft(token_id):

    secret_key = ""
    private_key = ""
    contract = "0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6"

    sdk = ThirdwebSDK.from_private_key(private_key, "mumbai", SDKOptions(secret_key=secret_key))
    contract = sdk.get_contract("0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6")

    # ---------------------------------------------
    # -------------- burning process --------------
    # ---------------------------------------------
    tx_hash = ""
    try:
        receipt = contract.erc721.burn(int(token_id))
    except:
        receipt = None
    if receipt != None:
        json = dict(receipt)
        tx_hash = json["transactionHash"].hex()
        return {"burn_tx_hash": tx_hash}
    else:
        return {"burn_tx_hash": ""}