



use web3::types::SignedData;
use std::str::FromStr;
use secp256k1::Secp256k1;
use web3::Web3;
use wallexerr::*;


pub mod evm{

    use super::*;

    /*  
        note that web3 is using secp256k1 algorithm in its core api thus the generated 
        pub and prv keys are the same one as for secp256k1:

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
    */
            
    pub fn get_keccak256_from(cid: String) -> String{
        /*
            EVM based public address is derived by taking the last 20 bytes of 
            the Keccak-256 hash of the public key
        */
        Wallet::generate_keccak256_from(cid)
    }

    pub fn get_wallet() -> Wallet{

        /* 
            simd ops on u256 bits can be represented as an slice with 4 elements 
            each of type 64 bits or 8 bytes, also 256 bits is 64 chars in hex 
            and 32 bytes of utf8 and  rust doesn't have u256
        */
        let u256 = web3::types::U256::from_str("0").unwrap().0;
                        
        /* 
            generaring new ECDSA keypair with secp256k1 curve 
            (compatible with all evm based chains) 
        */
        Wallet::new_secp256k1("", None)

        /* 
            generaring ECDSA keypair with secp256k1 curve from
            an existing mnemonic (compatible with all evm based chains) 
        */
        // let existing_mnemonic_sample = Some("obot glare amazing hip saddle habit soft barrel sell fine document february");
        // Wallet::new_secp256k1("", existing_mnemonic_sample)

    }
    
    pub async fn sign(wallet: Wallet, data: &str) -> (SignedData, String){

        let endpoint = std::env::var("INFURA_POLYGON_WS_ENDPOINT").unwrap();
        let transport = web3::transports::WebSocket::new(&endpoint).await.unwrap();
        let web3_con = Web3::new(transport);
    
        /* generating secret key instance from secp256k1 secret key */
        let web3_sec = web3::signing::SecretKey::from_str(wallet.secp256k1_secret_key.as_ref().unwrap().as_str()).unwrap();
        let keccak256_hash_of_message = web3_con.accounts().hash_message(data.to_string().as_bytes());
        println!("web3 keccak256 hash of message {:?}", keccak256_hash_of_message); 
    
        /* comparing the secp256k1 keypair with the web3 keypair */
        let secp = Secp256k1::default();
        println!("web3 secret key from secp256k1 {:?}", web3_sec.display_secret()); 
        println!("secp256k1 secret key {:?}", wallet.secp256k1_secret_key.as_ref().unwrap().as_str()); 
        println!("web3 pub key from secp256k1 {:?}", web3_sec.public_key(&secp));
        println!("secp256k1 pub key {:?}", web3_sec.public_key(&secp));
    
        /* signing the keccak256 hash of data */
        let signed_data = web3_con.accounts().sign(
            keccak256_hash_of_message, 
            &web3_sec
        );
    
        /* getting signature of the signed data */
        // signature bytes schema: pub struct Bytes(pub Vec<u8>);
        let sig_bytes = signed_data.signature.0.as_slice();
        let sig_str = hex::encode(sig_bytes);
        println!("web3 hex signature :::: {}", sig_str);

        /* 
            signature is a 520 bits or 65 bytes string which has 
            130 hex chars inside of it and can be divided into 
            two 256 bits or 32 bytes packs of hex string namely as
            r and s.
        */
        let signature = web3::types::H520::from_str(sig_str.as_str()).unwrap(); /* 64 bytes signature */
        println!("web3 signature :::: {}", signature);
        
        let hex_keccak256_hash_of_message = hex::encode(keccak256_hash_of_message.0).to_string();
        (signed_data, hex_keccak256_hash_of_message)
    
    }
    
    pub async fn verify_signature(
        sender: String,
        sig: &str,
        data_hash: &str
    ) -> Result<bool, bool>{
    
        let endpoint = std::env::var("INFURA_POLYGON_WS_ENDPOINT").unwrap();
        let transport = web3::transports::WebSocket::new(&endpoint).await.unwrap();
        let web3_con = Web3::new(transport);
    
        /* recovering public address from signature and keccak256 bits hash of the message */
        let data_hash = match hex::decode(data_hash){
            Ok(hash) => hash,
            Err(e) => return Err(false),
        };
        let rec_msg = web3::types::RecoveryMessage::Data(data_hash.clone());

        /* signature is a 65 bytes or 520 bits hex string contains 64 bytes of r + s (32 byte each) and a byte in the last which is v */
        let rec = web3::types::Recovery::from_raw_signature(rec_msg, hex::decode(sig).unwrap()).unwrap();
        
        println!("web3 recovery object {:?}", rec);
        
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
