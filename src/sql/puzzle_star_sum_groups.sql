SELECT grp.star_sum as group, count(grp.id) as puzzle_count
FROM (
    SELECT puzzle.id, sum(star.value) as star_sum FROM puzzle
    INNER JOIN star ON star.puzzle_id = puzzle.id
    WHERE puzzle.user_id = $1
    GROUP BY puzzle.id
) as grp
GROUP BY star_sum
ORDER BY star_sum DESC
