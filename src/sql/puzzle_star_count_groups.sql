SELECT grp.star_count, count(grp.id) as puzzle_count
FROM (
    SELECT puzzle.id, count(star.id) as star_count FROM puzzle
    INNER JOIN star ON star.puzzle_id = puzzle.id
    WHERE puzzle.user_id = ?
    GROUP BY puzzle.id
) as grp
GROUP BY star_count
ORDER BY star_count DESC
