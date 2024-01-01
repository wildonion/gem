-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_clps (
  id SERIAL PRIMARY KEY, -- this is the actual primary key 
  clp_event_id SERIAL REFERENCES clp_events(id),
  user_id SERIAL REFERENCES users(id),
  entry_amount BIGINT,
  registered_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users_clps');