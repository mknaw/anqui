-- Your SQL goes here

CREATE TYPE flip_mode AS ENUM ('front', 'back', 'both');
ALTER TABLE decks ADD COLUMN flip_mode flip_mode;
UPDATE decks SET flip_mode = 'front';
ALTER TABLE decks ALTER COLUMN flip_mode SET NOT NULL;
