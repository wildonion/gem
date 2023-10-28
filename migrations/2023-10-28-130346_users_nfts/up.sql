-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_nfts (
  id SERIAL PRIMARY KEY,
  contract_address VARCHAR NOT NULL,
  current_owner_screen_cid VARCHAR NOT NULL,
  img_url VARCHAR NOT NULL,
  onchain_id VARCHAR DEFAULT NULL,
  nft_name VARCHAR NOT NULL,
  nft_description VARCHAR NOT NULL,
  is_minted BOOLEAN DEFAULT false,
  current_price BIGINT DEFAULT 0,
  is_listed BOOLEAN DEFAULT true,
  metadata JSONB,
  comments JSONB,
  likes JSONB,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);