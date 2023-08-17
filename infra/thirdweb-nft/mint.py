

# pip install thirdweb-sdk

from thirdweb import ThirdwebSDK
from thirdweb.types.nft import NFTMetadataInput
from thirdweb.types import SDKOptions


def mint_nft(to):
    secret_key = ""
    private_key = ""

    sdk = ThirdwebSDK.from_private_key(private_key, "mumbai", SDKOptions(secret_key=secret_key))
    contract = sdk.get_contract("0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6")

    try:
        # Note that you can customize this metadata however you like
        metadata = NFTMetadataInput.from_json({
            "name": "12 MT YouWho NFT",
            "description": "Minting 12 MT for wildonion.yowho",
            "image": open("card3.png", "rb"),
        })

        # ---------------------------------------------
        # -------------- minting process --------------
        # ---------------------------------------------
        tx = contract.erc721.mint_to(to, metadata)
        receipt = dict(tx.receipt)
        tx_hash = receipt['transactionHash'].hex()
    except:
        tx_hash = ""
    return tx_hash