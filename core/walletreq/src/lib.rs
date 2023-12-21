



/** 
 *  ---------------------
 * | RSA based ECC curves
 *  ---------------------
 * - ed25519
 * - secp256k1
 * - secp256r1
 *  ---------------------
 * | HASH METHODS
 *  ---------------------
 * - sha3 keccak256 ---- secp256k1, secp256r1, ed25519
 * - aes256         ---- ed25519
 * - sha2
*/

use web3::types::SignedData;
use std::str::FromStr;
use secp256k1::Secp256k1;
use web3::Web3;
use wallexerr::misc::*;




/* -------------------------- */
/* ------- ed25519 ---------- */
/* -------------------------- */
pub mod ed25519{

    use super::*;
    
    pub fn generate_new_wallet() -> Wallet{

        Wallet::new_ed25519()
    }

    

}

/* -------------------------- */
/* ------- symmetric  ------- */
/* -------------------------- */
pub mod symmteric{

    pub use super::*;

    pub fn get_default_secure_cell_config() -> SecureCellConfig{
        SecureCellConfig::default()
    }

    pub fn get_default_aes256_config() -> Aes256Config{
        Aes256Config::default()
    }

}

/* -------------------------- */
/* ------- sescp256r1 ------- */
/* -------------------------- */
pub mod secp256r1{

    use super::*;

    pub fn generate_new_wallet() -> Wallet{

        Wallet::new_secp256r1()
    }

}

/* -------------------------- */
/* ------- sescp256k1 ------- */
/* -------------------------- */
/*  
    note that web3 is using secp256k1 curve algorithm in its core api thus the generated 
    pub and prv keys are the same one as for Secp256k1 crate:

    web3 keccak256 hash of message 0xa6f6dda07557e9005ed7b3105c48c7c4e472167023d8ce3e1787107697a6d98a
    web3 secret key from secp256k1 DisplaySecret("0194a1673885bfffa5211efc70c01ad85e923dab357300e3c45abe31052eca1a")
    secp256k1 secret key "0194a1673885bfffa5211efc70c01ad85e923dab357300e3c45abe31052eca1a"
    web3 pub key from secp256k1 PublicKey(3415f94c218d5bf04a555d36b951a17116b89f535f836d44b49512d74e0b25f143a17f284fb162c35524f5e747afc0e25febfac37991ab2bf1d1f611424ce029)
    secp256k1 pub key PublicKey(3415f94c218d5bf04a555d36b951a17116b89f535f836d44b49512d74e0b25f143a17f284fb162c35524f5e747afc0e25febfac37991ab2bf1d1f611424ce029)
    web3 hex signature :::: 19e0983f2a7d273291f2aecec039dd30f0b8721bae49008104d0a467c4ed7beb0a93f660e5f4d1611be3563962cab80eee3dd74352402fc849cb4cc292914c111c
    web3 signature :::: 0x19e0…111c
    [2023-11-19T11:35:24Z INFO  panel::models::users] sig :::: 19e0983f2a7d273291f2aecec039dd30f0b8721bae49008104d0a467c4ed7beb0a93f660e5f4d1611be3563962cab80eee3dd74352402fc849cb4cc292914c111c
    [2023-11-19T11:35:24Z INFO  panel::models::users] v :::: 28
    [2023-11-19T11:35:24Z INFO  panel::models::users] r :::: 19e0983f2a7d273291f2aecec039dd30f0b8721bae49008104d0a467c4ed7beb
    [2023-11-19T11:35:24Z INFO  panel::models::users] s :::: 0a93f660e5f4d1611be3563962cab80eee3dd74352402fc849cb4cc292914c11
    [2023-11-19T11:35:24Z INFO  panel::models::users] hash data :::: a6f6dda07557e9005ed7b3105c48c7c4e472167023d8ce3e1787107697a6d98a

    data hash is the keccak256 or sha3 hash of a transaction or data that 
    will be signed using the private key of the sender or signer

    r and s: these are two 256-bit numbers that together form the actual signature.
    v: is known as the recovery id, its purpose is to recover 
        the public key from the signature, in Ethereum, v is often 
        either 27 or 28, and with EIP-155, it can also encode the 
        chain id to prevent replay attacks between different networks.

    in evm world simd ops on u256 bits can be represented as an slice with 4 elements 
    each of type 64 bits or 8 bytes, also 256 bits is 64 chars in hex and 32 bytes of 
    utf8 and  rust doesn't have u256 like: 
        let u256 = web3::types::U256::from_str("0").unwrap().0;
*/
pub mod evm{

    use super::*;
            
    pub fn get_keccak256_from(cid: String) -> String{
        /* >------------------------------------------------------------------
            EVM based public address is derived by taking the last 20 bytes of 
            the Keccak-256 hash of the public key
        */
        Wallet::generate_keccak256_from(cid)
    }

    pub fn get_wallet() -> Wallet{                
        /* >------------------------------------------------------------------
            generaring new ECDSA keypair with secp256k1 curve 
            (compatible with all evm based chains) 
        */
        Wallet::new_secp256k1("", None)

        /* >------------------------------------------------------------------
            generaring ECDSA keypair with secp256k1 curve from
            an existing mnemonic (compatible with all evm based chains) 
            
            let existing_mnemonic_sample = Some("obot glare amazing hip saddle habit soft barrel sell fine document february");
            Wallet::new_secp256k1("", existing_mnemonic_sample)
        */

    }
    
    pub async fn sign(
        wallet: Wallet, 
        data: &str) -> (SignedData, String){

        let endpoint = std::env::var("INFURA_POLYGON_WS_ENDPOINT").unwrap();
        wallexerr::evm::sign(wallet, data, &endpoint).await
    
    }
    
    /* sender must be inside the signature means that signer pubkey must be inside signature */
    pub async fn verify_signature(
        sender: String,
        sig: &str,
        data_hash: &str
    ) -> Result<bool, bool>{
    
        let endpoint = std::env::var("INFURA_POLYGON_WS_ENDPOINT").unwrap();        
        wallexerr::evm::verify(&sender, sig, data_hash, &endpoint).await
    
    }

}


/* -------------------------- */
/* ------- utilities  ------- */
/* -------------------------- */
pub mod exports{
    
    pub use super::*;

    pub fn get_sha256_from(data: &str) -> [u8; 32]{
        Wallet::generate_sha256_from(data)
    }
}


