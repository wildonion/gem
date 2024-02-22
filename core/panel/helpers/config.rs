


use crate::*;


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Env{
    pub REDIS_HOST: String,  
    pub REDIS_PORT: String,  
    pub REDIS_USERNAME: String,  
    pub REDIS_PASSWORD: String,  
    pub MONGODB_HOST: String,  
    pub MONGODB_PORT: String,  
    pub MONGODB_USERNAME: String,  
    pub MONGODB_PASSWORD: String,  
    pub MONGODB_ENGINE: String,  
    pub DB_ENVIRONMENT: String,  
    pub SECRET_KEY: String,  
    pub WHITELIST_SECRET_KEY: String,  
    pub TWITTER_MAIN_ACCOUNT: String,  
    pub MACHINE_ID: String,  
    pub NODE_ID: String,  
    pub SERENITY_SHARDS: String,  
    pub COMPANY_NAME: String,  
    pub RUST_BACKTRACE: String,  
    pub JWT_SECRET_KEY: String,  
    pub JWT_EXPIRATION: String,  
    pub COOKIE_EXPIRATION_DAYS: String,  
    pub IO_BUFFER_SIZE: String,  
    pub FILE_SIZE: String,  
    pub EVENT_EXPIRATION: String,  
    pub GIFT_CARD_POLYGON_NFT_CONTRACT_ADDRESS: String,  
    pub GIFT_CARD_POLYGON_NFT_OWNER_ADDRESS: String,  
    pub XBOT_KEY: String,  
    pub XCORD_TOKEN: String,  
    pub XCORD_CHANNEL_ID: String,  
    pub BLOCKNATIVE_TOKEN: String,  
    pub OTP_API_TOKEN: String,  
    pub OTP_API_TEMPLATE: String,  
    pub CURRENCY_LAYER_TOKEN: String,  
    pub THESMSWORKS_SECRET_KEY: String,  
    pub THESMSWORKS_JWT: String,  
    pub IPINFO_TOKEN: String,  
    pub SMTP_USERNAME: String,  
    pub SMTP_PASSWORD: String,  
    pub SMTP_SERVER: String,  
    pub OPENAI_API_KEY: String,
    pub GOOGLE_OAUTH_CLIENT_ID: String,  
    pub GOOGLE_OAUTH_CLIENT_SECRET: String,  
    pub GOOGLE_OAUTH_REDIRECT_URL: String,  
    pub GOOGLE_OAUTH_ACCESS_TOKEN_URL: String,
    pub GOOGLE_OAUTH_USER_INFO_URL: String,
    pub STRIPE_TOKEN_IMAGE_URL: String,  
    pub STRIPE_AUTOMATIC_TAX: String,  
    pub STRIPE_PANEL_UPDATE_BALANCE_WEBHOOK_URL: String,  
    pub STRIPE_WEBHOOK_SIGNATURE: String,  
    pub STRIPE_PUBLISHABLE_KEY: String,  
    pub STRIPE_SECRET_KEY: String,  
    pub STRIPE_PAYMENT_SUCCESS_URL: String,  
    pub STRIPE_PAYMENT_CANCEL_URL: String,  
    pub NFTPORT_TOKEN: String,  
    pub INFURA_POLYGON_WS_ENDPOINT: String,  
    pub INFURA_POLYGON_HTTPS_ENDPOINT: String,  
    pub HOST: String,  
    pub RENDEZVOUS_PORT: String,  
    pub PANEL_PORT: String,  
    pub KYC_GRPC_PORT: String,  
    pub XBOT_ENDPOINT: String,  
    pub KYC_GRPC_PANEL_KYC_CALLBACK: String,  
    pub DB_NAME: String,  
    pub DATABASE_URL: String,  
    pub DB_USERNAME: String,  
    pub DB_HOST: String,  
    pub DB_PORT: String,  
    pub DB_PASSWORD: String,  
    pub ENVIRONMENT: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Context<C>{
    vars: C

}
pub trait EnvExt{
    
    type Context;
    fn get_vars(&self) -> Self::Context;
}

impl EnvExt for Env{

    type Context = Context<Self>;

    fn get_vars(&self) -> Self::Context {
        
        let ctx = Context::<Env>{
            vars: Env{
                REDIS_HOST: std::env::var("REDIS_HOST").unwrap(),
                REDIS_PORT: std::env::var("REDIS_PORT").unwrap(),
                REDIS_USERNAME: std::env::var("REDIS_USERNAME").unwrap(),
                REDIS_PASSWORD: std::env::var("REDIS_PASSWORD").unwrap(),
                MONGODB_HOST: std::env::var("MONGODB_HOST").unwrap(),
                MONGODB_PORT: std::env::var("MONGODB_PORT").unwrap(),
                MONGODB_USERNAME: std::env::var("MONGODB_USERNAME").unwrap(),
                MONGODB_PASSWORD: std::env::var("MONGODB_PASSWORD").unwrap(),
                MONGODB_ENGINE: std::env::var("MONGODB_ENGINE").unwrap(),
                DB_ENVIRONMENT: std::env::var("DB_ENVIRONMENT").unwrap(),
                SECRET_KEY: std::env::var("SECRET_KEY").unwrap(),
                WHITELIST_SECRET_KEY: std::env::var("WHITELIST_SECRET_KEY").unwrap(),
                TWITTER_MAIN_ACCOUNT: std::env::var("TWITTER_MAIN_ACCOUNT").unwrap(),
                MACHINE_ID: std::env::var("MACHINE_ID").unwrap_or("1".to_string()),
                NODE_ID: std::env::var("NODE_ID").unwrap_or("1".to_string()),
                SERENITY_SHARDS: std::env::var("SERENITY_SHARDS").unwrap(),
                COMPANY_NAME: std::env::var("COMPANY_NAME").unwrap(),
                RUST_BACKTRACE: std::env::var("RUST_BACKTRACE").unwrap(),
                JWT_SECRET_KEY: std::env::var("JWT_SECRET_KEY").unwrap(),
                JWT_EXPIRATION: std::env::var("JWT_EXPIRATION").unwrap(),
                COOKIE_EXPIRATION_DAYS: std::env::var("COOKIE_EXPIRATION_DAYS").unwrap(),
                IO_BUFFER_SIZE: std::env::var("IO_BUFFER_SIZE").unwrap(),
                FILE_SIZE: std::env::var("FILE_SIZE").unwrap(),
                EVENT_EXPIRATION: env::var("EVENT_EXPIRATION").unwrap(),
                GIFT_CARD_POLYGON_NFT_CONTRACT_ADDRESS: std::env::var("GIFT_CARD_POLYGON_NFT_CONTRACT_ADDRESS").unwrap(),
                GIFT_CARD_POLYGON_NFT_OWNER_ADDRESS: std::env::var("GIFT_CARD_POLYGON_NFT_OWNER_ADDRESS").unwrap(),
                XBOT_KEY: std::env::var("XBOT_KEY").unwrap(),
                XCORD_TOKEN: std::env::var("XCORD_TOKEN").unwrap(),
                XCORD_CHANNEL_ID: std::env::var("XCORD_CHANNEL_ID").unwrap(),
                BLOCKNATIVE_TOKEN: std::env::var("BLOCKNATIVE_TOKEN").unwrap(),
                OTP_API_TOKEN: std::env::var("OTP_API_TOKEN").unwrap(),
                OTP_API_TEMPLATE: std::env::var("OTP_API_TEMPLATE").unwrap(),
                CURRENCY_LAYER_TOKEN: std::env::var("CURRENCY_LAYER_TOKEN").unwrap(),
                THESMSWORKS_SECRET_KEY: std::env::var("THESMSWORKS_SECRET_KEY").unwrap(),
                THESMSWORKS_JWT: std::env::var("THESMSWORKS_JWT").unwrap(),
                IPINFO_TOKEN: std::env::var("IPINFO_TOKEN").unwrap(),
                SMTP_USERNAME: std::env::var("SMTP_USERNAME").unwrap(),
                SMTP_PASSWORD: std::env::var("SMTP_PASSWORD").unwrap(),
                SMTP_SERVER: std::env::var("SMTP_SERVER").unwrap(),
                OPENAI_API_KEY: std::env::var("OPENAI_API_KEY").unwrap(),
                GOOGLE_OAUTH_CLIENT_ID: std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap(),
                GOOGLE_OAUTH_CLIENT_SECRET: std::env::var("GOOGLE_OAUTH_CLIENT_SECRET").unwrap(),
                GOOGLE_OAUTH_REDIRECT_URL: std::env::var("GOOGLE_OAUTH_REDIRECT_URL").unwrap(),
                GOOGLE_OAUTH_ACCESS_TOKEN_URL: std::env::var("GOOGLE_OAUTH_ACCESS_TOKEN_URL").unwrap(),
                GOOGLE_OAUTH_USER_INFO_URL: std::env::var("GOOGLE_OAUTH_USER_INFO_URL").unwrap(),
                STRIPE_TOKEN_IMAGE_URL: std::env::var("STRIPE_TOKEN_IMAGE_URL").unwrap(),
                STRIPE_AUTOMATIC_TAX: std::env::var("STRIPE_AUTOMATIC_TAX").unwrap(),
                STRIPE_PANEL_UPDATE_BALANCE_WEBHOOK_URL: std::env::var("STRIPE_PANEL_UPDATE_BALANCE_WEBHOOK_URL").unwrap(),
                STRIPE_WEBHOOK_SIGNATURE: std::env::var("STRIPE_WEBHOOK_SIGNATURE").unwrap(),
                STRIPE_PUBLISHABLE_KEY: std::env::var("STRIPE_PUBLISHABLE_KEY").unwrap(),
                STRIPE_SECRET_KEY: std::env::var("STRIPE_SECRET_KEY").unwrap(),
                STRIPE_PAYMENT_SUCCESS_URL: std::env::var("STRIPE_PAYMENT_SUCCESS_URL").unwrap(),
                STRIPE_PAYMENT_CANCEL_URL: std::env::var("STRIPE_PAYMENT_CANCEL_URL").unwrap(),
                NFTPORT_TOKEN: std::env::var("NFTPORT_TOKEN").unwrap(),
                INFURA_POLYGON_WS_ENDPOINT: std::env::var("INFURA_POLYGON_WS_ENDPOINT").unwrap(),
                INFURA_POLYGON_HTTPS_ENDPOINT: std::env::var("INFURA_POLYGON_HTTPS_ENDPOINT").unwrap(),
                HOST: std::env::var("HOST").unwrap(),
                RENDEZVOUS_PORT: std::env::var("RENDEZVOUS_PORT").unwrap(),
                PANEL_PORT: std::env::var("PANEL_PORT").unwrap(),
                KYC_GRPC_PORT: std::env::var("KYC_GRPC_PORT").unwrap(),
                XBOT_ENDPOINT: std::env::var("XBOT_ENDPOINT").unwrap(),
                KYC_GRPC_PANEL_KYC_CALLBACK: std::env::var("KYC_GRPC_PANEL_KYC_CALLBACK").unwrap(),
                DB_NAME: std::env::var("DB_NAME").unwrap(),
                DATABASE_URL: std::env::var("DATABASE_URL").unwrap(),
                DB_USERNAME: std::env::var("DB_USERNAME").unwrap(),
                DB_HOST: std::env::var("DB_HOST").unwrap(),
                DB_PORT: std::env::var("DB_PORT").unwrap(),
                DB_PASSWORD: std::env::var("DB_PASSWORD").unwrap(),
                ENVIRONMENT: std::env::var("ENVIRONMENT").unwrap(),
            }
        };

        ctx
        
    }

}