-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_galleries (
  id SERIAL PRIMARY KEY,
  owner_screen_cid VARCHAR NOT NULL,
  collections JSONB,
  gal_name VARCHAR NOT NULL UNIQUE,
  gal_description VARCHAR NOT NULL,
  invited_friends TEXT[] DEFAULT ARRAY[]::TEXT[],  -- ARRAY of TEXT
  extra JSONB,
  gallery_background VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users_galleries');