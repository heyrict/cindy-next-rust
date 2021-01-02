SELECT puzzle.id, count(dialogue.id) as dialogue_count from puzzle
INNER JOIN dialogue ON dialogue.puzzle_id = puzzle.id
WHERE puzzle.user_id = ? AND puzzle.yami <> 0
GROUP BY puzzle.id
ORDER BY dialogue_count DESC
LIMIT 1;
