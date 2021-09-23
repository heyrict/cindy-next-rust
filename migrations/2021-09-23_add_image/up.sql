ALTER TABLE public.puzzle
    ADD COLUMN IF NOT EXISTS content_image BYTEA NULL;
