SELECT genre, count(id) as puzzle_count FROM puzzle
WHERE puzzle.user_id = $1
GROUP BY genre
