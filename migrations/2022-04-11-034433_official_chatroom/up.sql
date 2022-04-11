-- Your SQL goes here

-- Convert private to official
ALTER TABLE chatroom RENAME private TO official;
ALTER TABLE chatroom ALTER COLUMN official SET DEFAULT false;
UPDATE chatroom SET official = NOT official;

-- Add new public column
ALTER TABLE chatroom
ADD COLUMN IF NOT EXISTS public BOOLEAN NOT NULL DEFAULT false;
