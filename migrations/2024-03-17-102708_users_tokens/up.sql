-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_tokens (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  current_balance BigInt DEFAULT NULL,
  last_balance BigInt DEFAULT NULL,
  charged_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);