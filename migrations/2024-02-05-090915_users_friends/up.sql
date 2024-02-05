-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_friends (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  friend_id INTEGER NOT NULL,
  is_accepted BOOLEAN NOT NULL DEFAULT true,
  requested_at BigInt NOT NULL
);