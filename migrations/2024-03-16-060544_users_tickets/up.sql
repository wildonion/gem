-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_tickets (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  title VARCHAR NOT NULL,
  cname VARCHAR NOT NULL,
  mail VARCHAR NOT NULL,
  cdescription VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users_tickets');