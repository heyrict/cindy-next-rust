-- This file should undo anything in `up.sql`
ALTER TABLE public.user
    ALTER first_name DROP DEFAULT,
    ALTER last_name DROP DEFAULT,
    ALTER email DROP DEFAULT,
    ALTER profile DROP DEFAULT,
    ALTER is_active DROP DEFAULT,
    ALTER is_staff DROP DEFAULT,
    ALTER is_superuser DROP DEFAULT,
    ALTER date_joined DROP DEFAULT,
    ALTER last_login DROP DEFAULT,
    ALTER hide_bookmark DROP DEFAULT,
    ALTER current_award_id DROP DEFAULT,
    ALTER last_read_dm_id DROP DEFAULT;
