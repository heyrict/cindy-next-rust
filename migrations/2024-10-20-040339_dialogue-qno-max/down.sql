-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION public.add_dialogue_qno_before_dialogue_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
declare
  qno integer;
begin
  select count(id) + 1 as qno from dialogue where puzzle_id = NEW.puzzle_id into qno;
  NEW.qno = qno;
  return NEW;
end;
$$;
