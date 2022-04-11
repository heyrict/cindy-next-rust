-- This file should undo anything in `up.sql`

-- Add new public column
ALTER TABLE chatroom DROP COLUMN IF EXISTS public;

-- Convert official to private
UPDATE chatroom SET official = NOT official;
ALTER TABLE chatroom RENAME official TO private;
