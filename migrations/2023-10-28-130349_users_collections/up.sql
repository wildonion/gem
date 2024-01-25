-- Your SQL goes here
CREATE TABLE IF NOT EXISTS users_collections (
  id SERIAL PRIMARY KEY,
  contract_address VARCHAR NOT NULL,
  nfts JSONB,
  col_name VARCHAR NOT NULL,
  symbol VARCHAR NOT NULL,
  owner_screen_cid VARCHAR NOT NULL,
  metadata_updatable BOOLEAN DEFAULT true,
  freeze_metadata BOOLEAN DEFAULT false,
  base_uri VARCHAR NOT NULL,
  royalties_share INTEGER NOT NULL,
  royalties_address_screen_cid VARCHAR NOT NULL,
  collection_background VARCHAR NOT NULL,
  extra JSONB,
  col_description VARCHAR NOT NULL,
  contract_tx_hash VARCHAR DEFAULT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

SELECT diesel_manage_updated_at('users_collections');

INSERT INTO "users_collections" ("contract_address", "nfts", "col_name", "symbol", "owner_screen_cid", "metadata_updatable", "freeze_metadata", "base_uri", "royalties_share", "royalties_address_screen_cid", "collection_background", "extra", "col_description", "contract_tx_hash", "created_at", "updated_at") VALUES
('0x35e81902dd457f44bae08112c386d9104f1e1ad4',	NULL,	'Conse Gift Card',	'CGC',	'0xB3E106F72E8CB2f759Be095318F70AD59E96bfC2',	't',	'f',	'',	250,	'0xB3E106F72E8CB2f759Be095318F70AD59E96bfC2',	'',	NULL,	'Conse Gift Card Collection',	'0xf35b2e15d2671610c795659f37479204d6e16daa98768f10c57adbecba71e497',	'2024-01-24 18:35:06.540236+00',	'2024-01-24 18:35:06.540236+00');