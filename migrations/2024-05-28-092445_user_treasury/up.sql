-- Your SQL goes here
CREATE TABLE IF NOT EXISTS user_treasury (
  id SERIAL PRIMARY KEY,
  user_id integer NOT NULL,
  done_at BigInt not null,
  amount BigInt NOT NULL,
  tx_type text not null,
  treasury_type text not null
);

SELECT diesel_manage_updated_at('user_treasury');