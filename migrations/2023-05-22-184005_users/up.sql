-- Your SQL goes here

CREATE TYPE UserRole AS ENUM ('admin', 'dev', 'user');

CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  region VARCHAR DEFAULT NULL,
  username VARCHAR NOT NULL UNIQUE,
  bio VARCHAR DEFAULT NULL,
  avatar VARCHAR DEFAULT NULL,
  banner VARCHAR DEFAULT NULL,
  wallet_background VARCHAR DEFAULT NULL,
  activity_code VARCHAR NOT NULL,
  twitter_username VARCHAR DEFAULT NULL UNIQUE,
  facebook_username VARCHAR DEFAULT NULL UNIQUE,
  discord_username VARCHAR DEFAULT NULL UNIQUE,
  identifier VARCHAR DEFAULT NULL UNIQUE,
  mail VARCHAR DEFAULT NULL UNIQUE,
  is_mail_verified BOOLEAN NOT NULL DEFAULT false,
  is_phone_verified BOOLEAN NOT NULL DEFAULT false,
  phone_number VARCHAR DEFAULT NULL UNIQUE,
  paypal_id VARCHAR DEFAULT NULL UNIQUE,
  account_number VARCHAR DEFAULT NULL UNIQUE,
  device_id VARCHAR DEFAULT NULL UNIQUE,
  social_id VARCHAR DEFAULT NULL UNIQUE,
  cid VARCHAR DEFAULT NULL UNIQUE,
  screen_cid VARCHAR DEFAULT NULL UNIQUE,
  snowflake_id BigInt DEFAULT NULL UNIQUE,
  stars BigInt DEFAULT NULL,
  user_role UserRole NOT NULL DEFAULT 'user',
  pswd VARCHAR NOT NULL,
  token_time BigInt DEFAULT NULL,
  balance BigInt DEFAULT NULL,
  last_login TIMESTAMP WITH TIME ZONE DEFAULT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users');

-- insert first dev and admin ever :)
INSERT INTO users (username, activity_code, user_role, pswd) 
VALUES
  ('devdevy', '', 'dev', '$argon2i$v=19$m=4096,t=3,p=1$aW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$JidlFUDSXcEMIx+kuB4Tdu7WpdT3oFcAMk0S/JWrJYQ'),
  ('adminy', '', 'admin', '$argon2i$v=19$m=4096,t=3,p=1$aW5zZWN1cmUtOTgwbzM3XiEzZnUpa3pibzV6KGtybTJzXl5ibzFuKi1udnkoNis4MiklNjB5cGRtLXU$wC2qklhQYpL31/FjbwMMCfZSdJ6pQjrXCXha49KxCKM');
