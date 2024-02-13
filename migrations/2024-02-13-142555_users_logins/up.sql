-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_logins (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  device_id VARCHAR NOT NULL UNIQUE,
  jwt VARCHAR NOT NULL, -- token time per device
  last_login TIMESTAMP WITH TIME ZONE DEFAULT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users_logins');