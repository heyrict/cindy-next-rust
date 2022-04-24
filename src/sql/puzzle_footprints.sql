SELECT DISTINCT ON (dialogue.puzzle_id) puzzle.* FROM dialogue
INNER JOIN puzzle ON dialogue.puzzle_id = puzzle.id
WHERE dialogue.user_id = $1
ORDER BY dialogue.puzzle_id DESC NULLS LAST
LIMIT $2
OFFSET $3;
