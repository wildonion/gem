

# pip install thirdweb-sdk
# https://thirdweb.com/dashboard/contracts

from thirdweb import ThirdwebSDK
from thirdweb.types.nft import NFTMetadataInput
from thirdweb.types import SDKOptions
from fastapi import FastAPI
from thirdweb.types import ContractPlatformFeeSchema

app = FastAPI()



@app.post("/mint/to/{mint_to}/{amount}")
def mint_nft(mint_to, amount):
    
    thirdweb_secret_key = "BPCpEo42xF34w5o2fdJ6uqv0I_m4jwDGHUzu05pNDd7EBZ9iUW3ULt4tgNqvKZE5WGQR8r42X4CofwDcAd6uzg"
    wallet_private_key = "5fcec999332133dcbca2ed18b83f87669087b9dd9f86691bca68a252aae3da02"
    nft_contract = "0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6"
    market_contract = "0x6bD06CCe2884Ffd5060e211142F6D4EEfCd14296"
    sdk = ThirdwebSDK.from_private_key(wallet_private_key, "mumbai", SDKOptions(secret_key=thirdweb_secret_key))

    nft_contract_handler = sdk.get_contract(nft_contract)
    market_contract_handler = sdk.get_contract(market_contract) # use this for offer, auction, buy and sell

    # Note that you can customize this metadata however you like
    metadata = NFTMetadataInput.from_json({
        "name": f"{amount} YouWho NFT",
        "description": "YouWho NFT Card",
        "image": open("card.png", "rb"),
    })

    # ---------------------------------------------
    # -------------- minting process --------------
    # ---------------------------------------------
    try:
        tx = nft_contract_handler.erc721.mint_to(mint_to, metadata)
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

    thirdweb_secret_key = "BPCpEo42xF34w5o2fdJ6uqv0I_m4jwDGHUzu05pNDd7EBZ9iUW3ULt4tgNqvKZE5WGQR8r42X4CofwDcAd6uzg"
    wallet_private_key = "5fcec999332133dcbca2ed18b83f87669087b9dd9f86691bca68a252aae3da02"
    nft_contract = "0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6"
    market_contract = "0x6bD06CCe2884Ffd5060e211142F6D4EEfCd14296"
    sdk = ThirdwebSDK.from_private_key(wallet_private_key, "mumbai", SDKOptions(secret_key=thirdweb_secret_key))

    nft_contract_handler = sdk.get_contract(nft_contract)
    market_contract_handler = sdk.get_contract(market_contract) # use this for offer, auction, buy and sell
    
    # ---------------------------------------------
    # -------------- burning process --------------
    # ---------------------------------------------
    tx_hash = ""
    try:
        receipt = nft_contract_handler.erc721.burn(int(token_id))
    except:
        receipt = None
    if receipt != None:
        json = dict(receipt)
        tx_hash = json["transactionHash"].hex()
        return {"burn_tx_hash": tx_hash}
    else:
        return {"burn_tx_hash": ""}