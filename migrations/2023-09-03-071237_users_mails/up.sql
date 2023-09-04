-- Your SQL goes here
CREATE TABLE users_mails (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  mail VARCHAR NOT NULL,
  code VARCHAR NOT NULL,
  exp BigInt NOT NULL DEFAULT 0,
  vat BigInt NOT NULL DEFAULT 0
);