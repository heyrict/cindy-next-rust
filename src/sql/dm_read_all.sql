SELECT
  with_users.with_user_id,
  max(with_users.direct_message_id) as direct_message_id,
  dm_read.dm_id
from
  (
    SELECT
      id as direct_message_id,
      (
        CASE
          WHEN direct_message.sender_id = $1 THEN direct_message.receiver_id
          ELSE direct_message.sender_id
        END
      ) as with_user_id
    from
      direct_message
    WHERE
      direct_message.sender_id = $1
      OR direct_message.receiver_id = $1
    ORDER BY
      direct_message_id DESC
  ) AS with_users
LEFT JOIN dm_read ON dm_read.with_user_id = with_users.with_user_id
GROUP BY with_users.with_user_id, dm_read.dm_id
ORDER BY direct_message_id DESC
LIMIT $2
OFFSET $3;
