-- Your SQL goes here
create EXTENSION tcn;
create trigger users_tcn_trigger
after insert or update or delete on users
for each row execute function triggered_change_notification();