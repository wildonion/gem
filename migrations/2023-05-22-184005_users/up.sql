-- Your SQL goes here

CREATE TYPE UserRole AS ENUM ('admin', 'dev', 'user');

CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  username VARCHAR NOT NULL,
  activity_code VARCHAR NOT NULL,
  twitter_username VARCHAR DEFAULT NULL,
  facebook_username VARCHAR DEFAULT NULL,
  discord_username VARCHAR DEFAULT NULL,
  wallet_address VARCHAR DEFAULT NULL,
  user_role UserRole NOT NULL DEFAULT 'user',
  pswd VARCHAR NOT NULL,
  token_time BigInt DEFAULT NULL,
  last_login TIMESTAMP WITH TIME ZONE DEFAULT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- insert first dev and admin ever :)
INSERT INTO users (username, activity_code, user_role, pswd) 
VALUES
  ('devdevy', '', 'dev', '$argon2i$v=19$m=4096,t=3,p=1$aW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$JidlFUDSXcEMIx+kuB4Tdu7WpdT3oFcAMk0S/JWrJYQ'),
  ('adminy', '', 'admin', '$argon2i$v=19$m=4096,t=3,p=1$aW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$wC2qklhQYpL31/FjbwMMCfZSdJ6pQjrXCXha49KxCKM');
