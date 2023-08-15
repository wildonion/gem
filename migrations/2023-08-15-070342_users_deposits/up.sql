-- Your SQL goes here
CREATE TABLE users_deposits (
  id SERIAL PRIMARY KEY,
  payment_id VARCHAR NOT NULL,
  from_cid VARCHAR NOT NULL,
  recipient_cid VARCHAR NOT NULL,
  amount BigInt NOT NULL,
  tx_signature VARCHAR NOT NULL,
  iat TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);