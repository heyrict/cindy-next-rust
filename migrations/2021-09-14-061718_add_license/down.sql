-- Drop license column for user
ALTER TABLE public.puzzle
    DROP COLUMN IF EXISTS license_id;

-- Drop license column for puzzle
ALTER TABLE public.user
    DROP COLUMN IF EXISTS default_license_id;

-- Drop license table
DROP TABLE IF EXISTS public.license;
