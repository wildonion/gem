-- Your SQL goes here
CREATE TABLE users_checkouts (
  id SERIAL PRIMARY KEY,
  user_cid VARCHAR NOT NULL,
  product_id VARCHAR NOT NULL,
  price_id VARCHAR NOT NULL,
  payment_status VARCHAR NOT NULL,
  payment_intent VARCHAR NOT NULL,
  c_status VARCHAR NOT NULL,
  checkout_session_url VARCHAR NOT NULL,
  checkout_session_id VARCHAR NOT NULL,
  checkout_session_expires_at BigInt NOT NULL,
  tokens BigInt NOT NULL,
  usd_token_price BigInt NOT NULL,
  tx_signature VARCHAR NOT NULL,
  iat TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);