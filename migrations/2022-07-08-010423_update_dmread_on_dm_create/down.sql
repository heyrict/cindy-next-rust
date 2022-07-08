-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS update_dm_read_on_dm_create ON public.direct_message;

DROP FUNCTION update_dm_read;
