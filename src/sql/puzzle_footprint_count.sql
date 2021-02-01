SELECT COUNT(DISTINCT "puzzle"."id") AS count FROM "dialogue"
INNER JOIN "puzzle" ON "dialogue"."puzzle_id" = "puzzle"."id"
WHERE "dialogue"."user_id" = $1;
