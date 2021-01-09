SELECT
  DISTINCT with_users.with_user_id,
  dm_read.dm_id
from
  (
    SELECT
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
      direct_message.id DESC
  ) AS with_users
  LEFT JOIN dm_read ON dm_read.with_user_id = with_users.with_user_id
  LIMIT $2
  OFFSET $3;
