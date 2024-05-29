-- Your SQL goes here
CREATE TABLE IF NOT EXISTS sys_treasury (
  id SERIAL PRIMARY KEY,
  airdrop BigInt NOT NULL,
  debit BigInt NOT NULL,
  paid_to INTEGER NOT NULL,
  current_networth BigInt NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('sys_treasury');