SELECT user_id, nickname, bool_or("true") as true_answer, count(*) as dialogue_count, count(answeredtime) as answerd_dialogue_count FROM dialogue
LEFT JOIN "user" ON dialogue.user_id = "user".id
WHERE dialogue.puzzle_id = ?
GROUP BY dialogue.user_id, nickname;
