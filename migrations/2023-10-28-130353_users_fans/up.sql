-- Your SQL goes here

CREATE TABLE IF NOT EXISTS users_fans (
  id SERIAL,
  user_screen_cid VARCHAR NOT NULL UNIQUE,
  friends JSONB,
  invitation_requests JSONB,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
  PRIMARY KEY(id, user_screen_cid)
);

SELECT diesel_manage_updated_at('users_fans');