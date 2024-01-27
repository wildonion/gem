-- Your SQL goes here
CREATE TABLE IF NOT EXISTS token_stats (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  usd_token_price BigInt NOT NULL,
  requested_tokens BigInt NOT NULL,
  requested_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);