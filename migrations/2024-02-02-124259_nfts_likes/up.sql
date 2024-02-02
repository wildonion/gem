-- Your SQL goes here
CREATE TABLE IF NOT EXISTS nfts_likes (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  nft_id INTEGER NOT NULL,
  is_upvote BOOLEAN NOT NULL DEFAULT true,
  published_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);