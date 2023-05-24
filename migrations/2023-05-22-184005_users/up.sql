-- Your SQL goes here

CREATE TYPE UserRole AS ENUM ('admin', 'dev', 'user');

CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL,
  twitter_username VARCHAR NOT NULL,
  facebook_username VARCHAR NOT NULL,
  discord_username VARCHAR NOT NULL,
  wallet_address VARCHAR NOT NULL,
  user_role UserRole NOT NULL DEFAULT 'user',
  pswd VARCHAR NOT NULL,
  last_login TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
)

INSERT INTO users (username, twitter_username, facebook_username, discord_username, wallet_address, user_role, pswd) 
VALUES
  ("bejoedev", "", "", "", "", "dev", "$argon2i$v=19$m=4096,t=3,p=1$aW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$JidlFUDSXcEMIx+kuB4Tdu7WpdT3oFcAMk0S/JWrJYQ"),
  ("bejoeadmin", "", "", "", "", "admin", "$argon2i$v=19$m=4096,t=3,p=1$aW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$wC2qklhQYpL31/FjbwMMCfZSdJ6pQjrXCXha49KxCKM");