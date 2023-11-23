-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_chats (
  id SERIAL PRIMARY KEY,
  clp_event_id SERIAL REFERENCES clp_events(id),
  user_id SERIAL REFERENCES users(id),
  content VARCHAR,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users_chats');