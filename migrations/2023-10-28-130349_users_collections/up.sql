-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_collections (
  id SERIAL PRIMARY KEY,
  contract_address VARCHAR NOT NULL,
  nfts JSONB,
  col_name VARCHAR NOT NULL,
  symbol VARCHAR NOT NULL,
  owner_screen_cid VARCHAR NOT NULL,
  metadata_updatable BOOLEAN DEFAULT true,
  base_uri VARCHAR NOT NULL,
  royalties_share INTEGER NOT NULL,
  royalties_address_screen_cid VARCHAR NOT NULL,
  collection_background VARCHAR NOT NULL,
  metadata JSONB,
  col_description VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);