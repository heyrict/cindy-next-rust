SELECT puzzle.*, count(*) as star_count, sum(star.value) as star_sum from puzzle
INNER JOIN star ON puzzle.id = star.puzzle_id
WHERE puzzle.created >= $1 AND puzzle.created < $2
GROUP BY puzzle.id
ORDER BY star_count DESC, star_sum DESC, puzzle.id DESC
LIMIT $3
OFFSET $4
