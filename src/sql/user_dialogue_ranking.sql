SELECT "user".id, count(*) as value_count from "user"
INNER JOIN dialogue ON "user".id = dialogue.user_id
WHERE dialogue.created >= $1 AND dialogue.created < $2
GROUP BY "user".id
ORDER BY value_count DESC, "user".nickname ASC
LIMIT $3
OFFSET $4
