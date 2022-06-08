-- This file should undo anything in `up.sql`

ALTER TABLE decks DROP COLUMN flip_mode;
DROP TYPE flip_mode;
