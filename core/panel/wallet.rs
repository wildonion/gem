

/* 
     ---------------------------------------------------------------- 
    |  Asymmetric Crypto Wallet Implementations using ECC Algorithm
    |----------------------------------------------------------------
    | ed25519
    | secp256k1
    | secp256r1
    |

*/



use crate::models::users::NewIdRequest;
use crate::misc;
use crate::*;




#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewSeedRequest{
    pub mail: String,
    pub username: String,
    pub phone_number: String,
    pub paypal_id: String,
    pub account_number: String,
    pub device_id: String,
    pub social_id: String,
}

// https://thalesdocs.com/gphsm/luna/7/docs/network/Content/sdk/using/ecc_curve_cross-reference.htm
#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub secp256k1_secret_key: Option<String>,
    pub secp256k1_public_key: Option<String>,
    pub secp256k1_public_address: Option<String>,
    pub secp256r1_secret_key: Option<String>,
    pub secp256r1_public_key: Option<String>,
    pub ed25519_secret_key: Option<String>,
    pub ed25519_public_key: Option<String>
}

impl Wallet{

    pub fn generate_keccak256_from(pubk: String) -> String{

        let pubk = PublicKey::from_str(&pubk).unwrap();
        let public_key = pubk.serialize_uncompressed();
        let hash = keccak256(&public_key[1..]);
        let addr: Address = Address::from_slice(&hash[12..]);
        let addr_bytes = addr.as_bytes();
        let addr_string = format!("0x{}", hex::encode(&addr_bytes));
        addr_string

    }

    pub fn new_ed25519() -> Self{

        let rng = ring_rand::SystemRandom::new();
        let pkcs8_bytes = ring_signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let keys = ring_signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();

        /* ED25519 keypair */
        let pubkey = keys.public_key().as_ref();
        let prvkey = pkcs8_bytes.as_ref();

        /* converting bytes to hex string */
        let pubkey_string = hex::encode(&pubkey);
        let prvkey_string  = hex::encode(&prvkey);

        Wallet{
            secp256k1_secret_key: None,
            secp256k1_public_key: None,
            secp256k1_public_address: None,
            secp256r1_public_key: None,
            secp256r1_secret_key: None,
            ed25519_public_key: Some(pubkey_string),
            ed25519_secret_key: Some(prvkey_string)
        }

    }

    pub fn new_secp256k1(input_id: NewIdRequest) -> Self{

        /* generating seed from the input id to create the rng for secp256k1 keypair */
        let input_id_string = serde_json::to_string(&input_id).unwrap();
        let input_id_bytes = input_id_string.as_bytes();
        let hashed_input_id = ring::digest::digest(&ring::digest::SHA256, input_id_bytes);
        let hashed_input_id_bytes = hashed_input_id.as_ref();
        
        /* to create the rng we need a 32 bytes seed and we're sure that the hash is 32 bytes cause it's sha256 bits */
        let seed = <[u8; 32]>::try_from(&hashed_input_id_bytes[0..32]).unwrap(); /* creating a 32 bytes from the first 32 bytes of hashed_input_id_bytes */
        let rng = &mut StdRng::from_seed(seed);
        
        /* since the secp is going to be built from an specific seed thus the generated keypair will be the same everytime we request a new one */
        let secp = secp256k1::Secp256k1::new();
        let (prvk, pubk) = secp.generate_keypair(rng);
        let prv_str = prvk.display_secret().to_string();

        Wallet{
            secp256k1_secret_key: Some(prv_str),
            secp256k1_public_key: Some(pubk.to_string()),
            secp256k1_public_address: Some(Self::generate_keccak256_from(pubk.to_string())),
            secp256r1_public_key: None,
            secp256r1_secret_key: None,
            ed25519_public_key: None,
            ed25519_secret_key: None
        }
    }

    pub fn new_secp256r1() -> Self{

        /* ECDSA keypairs */
        let ec_key_pair = gen_ec_key_pair(); // generates a pair of Elliptic Curve (ECDSA) keys
        let (private, public) = ec_key_pair.clone().split();
        let hex_pub = Some(hex::encode(public.as_ref()));
        let hex_prv = Some(hex::encode(private.as_ref()));

        Wallet { 
            secp256k1_secret_key: None, 
            secp256k1_public_key: None, 
            secp256k1_public_address: None, 
            secp256r1_secret_key: hex_prv, 
            secp256r1_public_key: hex_pub,
            ed25519_public_key: None,
            ed25519_secret_key: None,
        }

    }

    pub fn ed25519_sign(data: String, prvkey: String) -> Option<String>{

        /* generating sha25 bits hash of data */
        let mut hasher = Sha256::new();
        hasher.update(data.as_str());
        let hash_data = hasher.finalize();

        /* decoding private key to bytes cause it's in hex format */
        let prvkey_bytes = hex::decode(prvkey).unwrap();
        let ed25519 = Self::retrieve_ed25519_keypair(&prvkey_bytes);

        /* signing the hashed data */
        let signature = ed25519.sign(&hash_data);
        let sig = signature.as_ref().to_vec();
        Some(hex::encode(&sig))

    }

    pub fn verify_ed25519_signature(sig: String, data: String, pubkey: String) -> bool{

        /* decoding sig string to bytes cause it's in hex format */
        let sig_bytes = hex::decode(&sig).unwrap();

        /* generating sha25 bits hash of data */
        let mut hasher = Sha256::new();
        hasher.update(data.as_str());
        let hash_data = hasher.finalize();

        /* creating the public key  */
        let ring_pubkey = ring_signature::UnparsedPublicKey::new(
            &ring_signature::ED25519, 
            pubkey.as_bytes());

        /* 
            Vec<u8> can be coerced to &[u8] slice by taking a reference to it 
            since a pointer to the underlying Vec<u8> means taking a slice of 
            vector with a valid lifetime
        */
        match ring_pubkey.verify(&hash_data, &sig_bytes){ 
            Ok(_) => true,
            Err(_) => false
        }

    }

    pub fn retrieve_ed25519_keypair(prv_key: &[u8]) -> Ed25519KeyPair{

        /* constructing keypair from the private key */
        let private_key = hex::decode(&prv_key).unwrap();
        let generated_ed25519_keys = Ed25519KeyPair::from_pkcs8(private_key.as_ref()).unwrap();
        generated_ed25519_keys

    }

    pub fn verify_secp256k1_signature(data: String, sig: Signature, pk: PublicKey) -> Result<(), secp256k1::Error>{

        /* 
            data is required to be passed to the method since we'll compare
            the hash of it with the one inside the signature 
        */
        let data_bytes = data.as_bytes();
        let hashed_data = Message::from_hashed_data::<sha256::Hash>(data_bytes);
            
        /* message is an sha256 bits hashed data */
        let secp = Secp256k1::verification_only();
        secp.verify_ecdsa(&hashed_data, &sig, &pk)

    }

    pub fn retrieve_secp256k1_keypair(secret_key: &[u8], public_key: &[u8]) -> (PublicKey, SecretKey){

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(secret_key).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        (public_key, secret_key)
    }

    pub fn secp256k1_sign(signer: String, data: String) -> Signature{

        let secret_key = SecretKey::from_str(&signer).unwrap();
        let data_bytes = data.as_bytes();
        let hashed_data = Message::from_hashed_data::<sha256::Hash>(data_bytes);
        
        /* message is an sha256 bits hashed data */
        let secp = Secp256k1::new();

        /* signing the hashed data */
        secp.sign_ecdsa(&hashed_data, &secret_key)

    }

    pub fn retrieve_secp256r1_keypair(hex_pubkey: &str, hex_prvkey: &str) -> themis::keys::KeyPair{

        /* building ECDSA keypair from pubkey and prvkey slices */
        let pubkey_bytes = hex::decode(hex_pubkey).unwrap();
        let prvkey_bytes = hex::decode(hex_prvkey).unwrap();
        let ec_pubkey = EcdsaPublicKey::try_from_slice(&pubkey_bytes).unwrap();
        let ec_prvkey = EcdsaPrivateKey::try_from_slice(&prvkey_bytes).unwrap();
        let generated_ec_keypair = ThemisKeyPair::try_join(ec_prvkey, ec_pubkey).unwrap();
        generated_ec_keypair

    }

    pub fn secp256r1_sign(signer: String, data: String) -> Option<String>{

        /* building the signer from the private key */
        let prvkey_bytes = hex::decode(signer).unwrap();
        let ec_prvkey = EcdsaPrivateKey::try_from_slice(&prvkey_bytes).unwrap();
        let ec_signer = SecureSign::new(ec_prvkey.clone());
        
        /* json stringifying the json_input value */
        let inputs_to_sign = serde_json::to_string(&data).unwrap(); 

        /* generating sha256 bits hash of data */
        let mut hasher = Sha256::new();
        hasher.update(inputs_to_sign.as_str());
        let hash_data = hasher.finalize();
    
        /* generating signature from the hashed data */
        let ec_sig = ec_signer.sign(&hash_data).unwrap();
        
        /* converting the signature bytes into hex string */
        Some(hex::encode(&ec_sig))

    }

    pub fn verify_secp256r1_signature(signature: &[u8], pubkey: &[u8]) -> Result<Vec<u8>, themis::Error>{

        /* building the public key from public key bytes */
        let Ok(ec_pubkey) = EcdsaPublicKey::try_from_slice(pubkey) else{
            let err = EcdsaPublicKey::try_from_slice(pubkey).unwrap_err();
            return Err(err); /* can't build pubkey from the passed in slice */
        };

        /* building the verifier from the public key */
        let ec_verifier = SecureVerify::new(ec_pubkey.clone());

        /* verifying the signature byte which returns the hash of data in form of vector of utf8 bytes */
        let encoded_data = ec_verifier.verify(signature);

        encoded_data

    }
    
}

pub struct Contract{
    pub wallet: Wallet,
    pub iat: i64,
    pub owner: &'static str
}

impl Contract{

    pub fn new(owner: &str) -> Self{
        
        let static_owner = misc::string_to_static_str(owner.to_string());
        let wallet = Wallet::new_ed25519();

        Self { 
            wallet,
            iat: chrono::Local::now().timestamp_nanos(), 
            owner: static_owner 
        }
        
    }

}


#[cfg(test)]
pub mod tests{

    use super::*;

    pub fn ed25519_test() -> Result<(), ()>{
        
        #[derive(Serialize, Deserialize)]
        struct Data{
            pub repo: String,
            pub commits: u16,
            pub budget: u16 
        }
        let data = Data{
            repo: "github repo containing the code".to_string(), 
            commits: 0u16,
            budget: 50
        };
        let stringify_data = serde_json::to_string_pretty(&data).unwrap();

        let contract = Contract::new("wildonion");
        
        let signature_hex = Wallet::ed25519_sign(stringify_data.clone(), contract.wallet.ed25519_secret_key.unwrap());
        
        let is_verified = Wallet::verify_ed25519_signature(signature_hex.unwrap(), stringify_data, contract.wallet.ed25519_public_key.unwrap());

        match is_verified{
            true => Ok(()),
            false => Err(())
        }

    }


}
