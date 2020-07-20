-- Your SQL goes here
ALTER TABLE public.user
    ALTER first_name SET DEFAULT '',
    ALTER last_name SET DEFAULT '',
    ALTER email SET DEFAULT '',
    ALTER profile SET DEFAULT '',
    ALTER is_active SET DEFAULT true,
    ALTER is_staff SET DEFAULT false,
    ALTER is_superuser SET DEFAULT false,
    ALTER date_joined SET DEFAULT now(),
    ALTER last_login SET DEFAULT now(),
    ALTER hide_bookmark SET DEFAULT false,
    ALTER current_award_id SET DEFAULT NULL,
    ALTER last_read_dm_id SET DEFAULT NULL;
