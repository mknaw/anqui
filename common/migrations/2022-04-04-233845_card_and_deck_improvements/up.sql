ALTER TABLE cards
DROP COLUMN fail_count,
DROP COLUMN hard_count,
DROP COLUMN good_count,
DROP COLUMN easy_count,
ADD COLUMN revision_weight SMALLINT NOT NULL DEFAULT 100;

ALTER TABLE decks
ADD COLUMN user_id INT NOT NULL,
ADD CONSTRAINT fk_user
  FOREIGN key(user_id)
    REFERENCES users(id)
    ON DELETE CASCADE;
