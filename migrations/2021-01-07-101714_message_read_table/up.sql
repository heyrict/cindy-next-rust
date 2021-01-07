-- Your SQL goes here
ALTER TABLE "user"
DROP COLUMN last_read_dm_id RESTRICT;

CREATE TABLE "dm_read" (
    id            SERIAL,
    user_id       INTEGER NOT NULL  REFERENCES "user"(id) DEFERRABLE INITIALLY DEFERRED,
    with_user_id  INTEGER NOT NULL  REFERENCES "user"(id) DEFERRABLE INITIALLY DEFERRED,
    dm_id         INTEGER NOT NULL  REFERENCES direct_message(id) DEFERRABLE INITIALLY DEFERRED,
    PRIMARY KEY (id),
    UNIQUE(user_id, with_user_id)
);

ALTER TABLE "direct_message"
ADD COLUMN modified TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now();

CREATE TRIGGER update_dm_modified_trigger BEFORE UPDATE ON public.direct_message FOR EACH ROW EXECUTE PROCEDURE public.update_chatmessage_modified();
