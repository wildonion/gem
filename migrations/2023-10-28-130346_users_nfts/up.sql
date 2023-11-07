-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_nfts (
  id SERIAL PRIMARY KEY,
  contract_address VARCHAR NOT NULL,
  current_owner_screen_cid VARCHAR NOT NULL,
  metadata_uri VARCHAR NOT NULL,
  onchain_id VARCHAR DEFAULT NULL,
  nft_name VARCHAR NOT NULL,
  nft_description VARCHAR NOT NULL,
  is_minted BOOLEAN DEFAULT false,
  current_price BIGINT DEFAULT 0,
  is_listed BOOLEAN DEFAULT false,
  freeze_metadata BOOLEAN DEFAULT false,
  extra JSONB,
  attributes JSONB,
  comments JSONB,
  likes JSONB,
  tx_hash VARCHAR DEFAULT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users_nfts');