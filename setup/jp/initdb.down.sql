BEGIN;
    DELETE FROM chatroom
    WHERE name = 'lobby';
END;
