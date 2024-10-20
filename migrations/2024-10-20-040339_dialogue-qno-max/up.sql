-- Your SQL goes here
CREATE OR REPLACE FUNCTION public.add_dialogue_qno_before_dialogue_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
declare
  new_qno integer;
begin
  select coalesce(max(qno), 0) + 1 from dialogue where puzzle_id = NEW.puzzle_id into new_qno;
  NEW.qno = new_qno;
  return NEW;
end;
$$;
