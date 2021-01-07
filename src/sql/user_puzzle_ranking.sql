SELECT "user".*, count(*) as value_count from "user"
INNER JOIN puzzle ON "user".id = puzzle.user_id
WHERE puzzle.created >= $1 AND puzzle.created < $2
GROUP BY "user".id
ORDER BY value_count DESC, "user".nickname ASC
LIMIT $3
OFFSET $4
