-- Your SQL goes here
CREATE TABLE users_deposits (
  id SERIAL PRIMARY KEY,
  mint_tx_hash VARCHAR NOT NULL,
  nft_id VARCHAR NOT NULL,
  from_cid VARCHAR NOT NULL,
  recipient_screen_cid VARCHAR NOT NULL,
  is_claimed BOOLEAN NOT NULL DEFAULT false,
  amount BigInt NOT NULL,
  tx_signature VARCHAR NOT NULL,
  iat TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);