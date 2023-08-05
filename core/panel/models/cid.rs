


use crate::*;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DepositMetadata{
    pub from: UserId,
    pub recipient: UserId,
    pub amount: u64,
    pub signature: String, /* this must be generated inside the client by signing the operation using the client private key */
    pub iat: i64, // deposited at
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WithdrawMetadata{
    pub deposit_metadata: DepositMetadata,
    pub signature: String, /* this must be generated inside the client by signing the operation using the client private key */
    pub cat: i64, // claimed at
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewIdRequest{
    pub paypal_id: String,
    pub account_number: String,
    pub device_id: String,
    pub social_id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Id{
    pub paypal_id: String,
    pub account_number: String,
    pub device_id: String,
    pub social_id: String,
    pub username: String,
    pub snowflake_id: Option<i64>,
    pub unique_id: Option<String>, /* pubkey */
    pub signer: Option<String>, /* prvkey */
    pub signature: Option<String>, /* this is the proof of a valid data signed by the generated private key, will be returned to the user */
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UserId{
    pub paypal_id: String,
    pub account_number: String,
    pub device_id: String,
    pub social_id: String,
    pub username: String,
    pub snowflake_id: Option<i64>,
    pub unique_id: Option<String>, /* pubkey */
}

impl Id{


    pub fn new(mut id_: NewIdRequest) -> Id{

        /* ECDSA keypair */
        let ec_key_pair = gen_ec_key_pair(); // generates a pair of Elliptic Curve (ECDSA) keys
        let (private, public) = ec_key_pair.clone().split();
        let unique_id = Some(hex::encode(public.as_ref()));
        let signer = Some(hex::encode(private.as_ref()));

        /* generating snowflake id */
        let machine_id = std::env::var("MACHINE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
        let node_id = std::env::var("NODE_ID").unwrap_or("1".to_string()).parse::<i32>().unwrap();
        let mut id_generator_generator = SnowflakeIdGenerator::new(machine_id, node_id);
        let snowflake_id = id_generator_generator.real_time_generate();
        let snowflake_id = Some(snowflake_id);

        Id { 
            paypal_id: id_.paypal_id, 
            account_number: id_.account_number, 
            device_id: id_.device_id, 
            social_id: id_.social_id, 
            username: id_.username, 
            snowflake_id,
            unique_id,
            signer,
            signature: None,
        }

    }

    pub fn retrieve_keypair(&self) -> themis::keys::KeyPair{

        /* building ECDSA keypair from pubkey and prvkey slices */
        let pubkey_bytes = hex::decode(self.unique_id.as_ref().unwrap()).unwrap();
        let prvkey_bytes = hex::decode(self.signer.as_ref().unwrap()).unwrap();
        let ec_pubkey = EcdsaPublicKey::try_from_slice(&pubkey_bytes).unwrap();
        let ec_prvkey = EcdsaPrivateKey::try_from_slice(&prvkey_bytes).unwrap();
        let generated_ec_keypair = ThemisKeyPair::try_join(ec_prvkey, ec_pubkey).unwrap();
        generated_ec_keypair

    }

    pub fn sign(&mut self) -> Self{

        /* building the signer from the private key */
        let prvkey_bytes = hex::decode(self.signer.as_ref().unwrap()).unwrap();
        let ec_prvkey = EcdsaPrivateKey::try_from_slice(&prvkey_bytes).unwrap();
        let ec_signer = SecureSign::new(ec_prvkey.clone());

        /* stringifying the object_id instance to generate the signature */
        let json_input = serde_json::json!({
            "paypal_id": self.paypal_id,
            "account_number": self.account_number,
            "social_id": self.social_id,
            "username": self.username,
            "snowflake_id": self.snowflake_id,
            "device_id": self.device_id.clone(),
            "unique_id": self.unique_id.as_ref().unwrap(), /* unwrap() takes the ownership of self thus we've used as_ref() to prevent from moving */
        });
        
        /* json stringifying the json_input value */
        let inputs_to_sign = serde_json::to_string(&json_input).unwrap(); 
    
        /* generating signature from the input data */
        let ec_sig = ec_signer.sign(inputs_to_sign.as_bytes()).unwrap();
        
        /* converting the signature byte into hex string */
        self.signature = Some(hex::encode(&ec_sig)); 

        /* returning the cloned instance of the self to prevent the self from moving */
        self.clone()

    }

    pub fn verify(&self) -> bool{

        /* building the verifier from the public key */
        let pubkey_bytes = hex::decode(self.unique_id.as_ref().unwrap()).unwrap();
        let ec_pubkey = EcdsaPublicKey::try_from_slice(&pubkey_bytes).unwrap();
        let ec_verifier = SecureVerify::new(ec_pubkey.clone());

        /* converting the signature hex string into bytes */
        let signature_bytes = hex::decode(self.signature.as_ref().unwrap()).unwrap();

        /* verifying the signature byte which returns the data itself in form of utf8 bytes */
        let encoded_id_instance = ec_verifier.verify(&signature_bytes).unwrap();
        
        /* decoding the utf8 into an Id instance */
        let decoded_id_instance = serde_json::from_slice::<Id>(&encoded_id_instance).unwrap();

        let this_without_sig = Id{
            paypal_id: self.paypal_id.clone(),
            account_number: self.account_number.clone(),
            device_id: self.device_id.clone(),
            social_id: self.social_id.clone(),
            username: self.username.clone(),
            snowflake_id: self.snowflake_id.clone(),
            unique_id: self.unique_id.clone(),
            /* since the signer and signature fields were None while we were signing the data */
            signature: None,
            signer: None,
        };

        if this_without_sig == decoded_id_instance{
            true
        } else{
            false
        }

    }

    pub fn get_id_for_redis(&self) -> UserId{

        UserId { 
            paypal_id: self.paypal_id.clone(), 
            account_number: self.account_number.clone(), 
            device_id: self.device_id.clone(), 
            social_id: self.social_id.clone(), 
            username: self.username.clone(), 
            snowflake_id: self.snowflake_id.clone(), 
            unique_id: self.unique_id.clone()
        }

    }

}