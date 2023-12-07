-- Your SQL goes here
CREATE OR REPLACE FUNCTION notify_trigger() RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('on_update', 'update');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_update_trigger
AFTER UPDATE ON users
FOR EACH ROW EXECUTE PROCEDURE notify_trigger();