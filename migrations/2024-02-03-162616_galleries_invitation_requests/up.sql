-- Your SQL goes here
CREATE TABLE IF NOT EXISTS galleries_invitation_requests (
  id SERIAL PRIMARY KEY,
  invitee_id INTEGER NOT NULL,
  from_user_id INTEGER NOT NULL,
  gal_id INTEGER NOT NULL,
  is_accepted BOOLEAN NOT NULL DEFAULT true,
  requested_at BigInt NOT NULL
);