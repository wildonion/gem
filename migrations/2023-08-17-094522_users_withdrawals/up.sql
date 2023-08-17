-- Your SQL goes here
CREATE TABLE users_withdrawals (
  id SERIAL PRIMARY KEY,
  deposit_id INTEGER NOT NULL,
  burn_tx_signature VARCHAR NOT NULL,
  recipient_cid VARCHAR NOT NULL,
  is_claimed BOOLEAN NOT NULL DEFAULT false,
  tx_signature VARCHAR NOT NULL,
  wat TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);