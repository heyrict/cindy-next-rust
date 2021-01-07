-- This file should undo anything in `up.sql`

DROP TRIGGER IF EXISTS update_dm_modified_trigger ON public.direct_message CASCADE;

ALTER TABLE "direct_message"
DROP COLUMN modified;

DROP TABLE "dm_read";

ALTER TABLE "user"
ADD COLUMN last_read_dm_id INTEGER NULL REFERENCES direct_message(id) DEFERRABLE INITIALLY DEFERRED;
