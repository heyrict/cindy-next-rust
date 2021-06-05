-- Your SQL goes here
CREATE OR REPLACE VIEW tag_aggr AS (
  SELECT tag.*, COUNT(puzzle_tag.id) AS puzzle_tag_count
  FROM tag
  INNER JOIN puzzle_tag ON tag.id = puzzle_tag.tag_id
  GROUP BY tag.id
)
