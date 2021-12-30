CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "image" (
    id            UUID DEFAULT uuid_generate_v1mc(),
    user_id       INTEGER NOT NULL  REFERENCES "user"(id) DEFERRABLE INITIALLY DEFERRED,
    puzzle_id     INTEGER NULL  REFERENCES "puzzle"(id) ON DELETE SET NULL,
    created       TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    content_type  VARCHAR(32) NOT NULL,
    PRIMARY KEY (id)
);
