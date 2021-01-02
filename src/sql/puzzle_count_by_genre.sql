SELECT genre, count(id) as count FROM puzzle
WHERE puzzle.user_id = ?
GROUP BY genre
