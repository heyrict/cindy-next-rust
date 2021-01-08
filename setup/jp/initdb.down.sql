BEGIN;
    DELETE FROM chatroom WHERE name = 'lobby';

    DELETE FROM award;
END;
