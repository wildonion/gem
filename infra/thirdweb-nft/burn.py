


from thirdweb import ThirdwebSDK
from thirdweb.types.nft import NFTMetadataInput
from thirdweb.types import SDKOptions


def burn_nft(token_id):
    
    secret_key = ""
    private_key = ""

    sdk = ThirdwebSDK.from_private_key(private_key, "mumbai", SDKOptions(secret_key=secret_key))
    contract = sdk.get_contract("0xFBF8392fF5E5F2924f0e7Af9121adE9254711cC6")

    # ---------------------------------------------
    # -------------- burning process --------------
    # ---------------------------------------------
    try:
        receipt = contract.erc721.burn(int(token_id))
        json = dict(receipt)
        tx_hash = json["transactionHash"].hex()
    except:
        tx_hash = ""
    return tx_hash