-- Your SQL goes here
CREATE TABLE IF NOT EXISTS clp_events (
  id SERIAL PRIMARY KEY,
  contract_address VARCHAR NOT NULL,
  event_name VARCHAR NOT NULL,
  symbol VARCHAR NOT NULL,
  max_supply INTEGER NOT NULL,
  team_reserve INTEGER NOT NULL,
  mint_price BIGINT NOT NULL,
  presale_mint_price BIGINT NOT NULL,
  tokens_per_mint INTEGER NOT NULL,
  owner_screen_cid VARCHAR NOT NULL,
  treasury_address VARCHAR NOT NULL,
  public_mint_start_date VARCHAR NOT NULL,
  metadata_updatable BOOLEAN DEFAULT true,
  freeze_metadata BOOLEAN DEFAULT false,
  base_uri VARCHAR NOT NULL,
  presale_mint_start_date VARCHAR NOT NULL,
  presale_whitelisted_addresses TEXT[] DEFAULT ARRAY[]::TEXT[],
  prereveal_token_uri VARCHAR NOT NULL,
  royalties_share INTEGER NOT NULL,
  royalties_address_screen_cid VARCHAR NOT NULL,
  event_background VARCHAR NOT NULL,
  extra JSONB,
  event_description VARCHAR NOT NULL,
  contract_tx_hash VARCHAR DEFAULT NULL,
  start_at BigInt NOT NULL DEFAULT 0,
  expire_at BigInt NOT NULL DEFAULT 0,
  is_locked BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('clp_events');