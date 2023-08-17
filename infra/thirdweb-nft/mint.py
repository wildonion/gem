

# pip install thirdweb-sdk

from thirdweb import ThirdwebSDK
from thirdweb.types.nft import NFTMetadataInput
from thirdweb.types import SDKOptions


secret_key = ""
private_key = ""
mint_to = "0xDE6D7045Df57346Ec6A70DfE1518Ae7Fe61113f4"

sdk = ThirdwebSDK.from_private_key(private_key, "mumbai", SDKOptions(secret_key=secret_key))
contract = sdk.get_contract("0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6")

# Note that you can customize this metadata however you like
metadata = NFTMetadataInput.from_json({
    "name": "12 MT YouWho NFT",
    "description": "Minting 12 MT for wildonion.yowho",
    "image": open("card3.png", "rb"),
})

# ---------------------------------------------
# -------------- minting process --------------
# ---------------------------------------------
tx = contract.erc721.mint_to(mint_to, metadata)
receipt = tx.receipt
token_id = tx.id
nft = tx.data()
print(nft)


# ---------------------------------------------
# -------------- burning process --------------
# ---------------------------------------------
# token_id = 4
# receipt = contract.erc721.burn(token_id)
# json = dict(receipt)
# print(json["transactionHash"].hex())
