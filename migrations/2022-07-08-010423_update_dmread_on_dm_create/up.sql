-- Your SQL goes here
CREATE OR REPLACE FUNCTION update_dm_read() RETURNS trigger AS $$
BEGIN
    INSERT INTO dm_read (id, user_id, with_user_id, dm_id) VALUES (
        DEFAULT, NEW.sender_id, NEW.receiver_id, NEW.id
    )
    ON CONFLICT ON CONSTRAINT dm_read_user_id_with_user_id_key DO UPDATE SET dm_id = NEW.id
    WHERE dm_read.user_id = NEW.sender_id AND dm_read.with_user_id = NEW.receiver_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_dm_read_on_dm_create AFTER INSERT ON public.direct_message FOR EACH ROW EXECUTE PROCEDURE public.update_dm_read();
