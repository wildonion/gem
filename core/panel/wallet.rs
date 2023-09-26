


use wallexerr::Wallet;
use web3::types::SignedData;
use crate::*;



pub mod evm{

    use super::*;

    /* 
    
        data hash is the keccak256 hash of a transaction or data that 
        will be signed using the private key of the sender or signer
    
        r and s: these are two 256-bit numbers that together form the actual signature.
        
        v: is known as the recovery id, its purpose is to recover 
            the public key from the signature, in Ethereum, v is often 
            either 27 or 28, and with EIP-155, it can also encode the 
            chain id to prevent replay attacks between different networks.
    
        EVM based public address is derived by taking the last 20 bytes of 
        the Keccak-256 hash of the public key
    */

    pub fn get_wallet() -> Wallet{
        /* 
            ECDSA with secp256k1 curve keypairs 
            (compatible with all evm based chains) 
        */
        Wallet::new_secp256k1("")
    }
    
    pub async fn sign(wallet: Wallet, data: &str) -> (SignedData, String){

        let endpoint = env::var("INFURA_POLYGON_WS_ENDPOINT").unwrap();
        let transport = transports::WebSocket::new(&endpoint).await.unwrap();
        let web3_con = Web3::new(transport);
    
        /* generating secret key instance from secp256k1 secret key */
        let web3_sec = web3::signing::SecretKey::from_str(wallet.secp256k1_secret_key.as_ref().unwrap().as_str()).unwrap();
        let keccak256_hash_of_message = web3_con.accounts().hash_message(data.to_string().as_bytes());
        info!("web3 keccak256 hash of message {:?}", keccak256_hash_of_message); 
    
        /* comparing the secp256k1 keypair with the web3 keypair */
        let secp = Secp256k1::default();
        info!("web3 secret key from secp256k1 {:?}", web3_sec.display_secret()); 
        info!("secp256k1 secret key {:?}", wallet.secp256k1_secret_key.as_ref().unwrap().as_str()); 
        info!("web3 pub key from secp256k1 {:?}", web3_sec.public_key(&secp));
        info!("secp256k1 pub key {:?}", web3_sec.public_key(&secp));
    
        /* signing the keccak256 hash of data */
        let signed_data = web3_con.accounts().sign(
            keccak256_hash_of_message, 
            &web3_sec
        );
    
        /* getting signature of the signed data */
        // signature bytes schema: pub struct Bytes(pub Vec<u8>);
        let sig_bytes = signed_data.signature.0.as_slice();
        let sig_str = hex::encode(sig_bytes);
        info!("web3 hex signature :::: {}", sig_str);

        /* 
            signature is a 520 bits or 65 bytes string which has 
            130 hex chars inside of it and can be divided into 
            two 256 bits or 32 bytes packs of hex string namely as
            r and s.
        */
        let signature = web3::types::H520::from_str(sig_str.as_str()).unwrap(); /* 64 bytes signature */
        info!("web3 signature :::: {}", signature);
        
        let hex_keccak256_hash_of_message = hex::encode(keccak256_hash_of_message.0).to_string();
        (signed_data, hex_keccak256_hash_of_message)
    
    }
    
    pub async fn verify_signature(
        sender: String,
        sig: &str,
        data_hash: &str
    ) -> Result<bool, bool>{
    
        let endpoint = env::var("INFURA_POLYGON_WS_ENDPOINT").unwrap();
        let transport = transports::WebSocket::new(&endpoint).await.unwrap();
        let web3_con = Web3::new(transport);
    
        /* recovering public address from signature and keccak256 bits hash of the message */
        let data_hash = match hex::decode(data_hash){
            Ok(hash) => hash,
            Err(e) => return Err(false),
        };
        let rec_msg = web3::types::RecoveryMessage::Data(data_hash.clone());

        /* signature is a 65 bytes or 520 bits hex string contains 64 bytes of r + s (32 byte each) and a byte in the last which is v */
        let rec = web3::types::Recovery::from_raw_signature(rec_msg, hex::decode(sig).unwrap()).unwrap();
        
        info!("web3 recovery object {:?}", rec);
        
        /* recovers the EVM based public address or screen_cid which was used to sign the given data */
        if web3_con.accounts().recover(rec.clone()).is_err(){
            return Err(false);
        }
        let recovered_screen_cidh160 = web3_con.accounts().recover(rec).unwrap().to_fixed_bytes();
        let recovered_screen_cid_hex = format!("0x{}", hex::encode(&recovered_screen_cidh160));
    
        if sender == recovered_screen_cid_hex{
            Ok(true)
        } else{
            Err(false)
        }
    
    }

}
