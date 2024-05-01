-- This file should undo anything in `up.sql`
UPDATE archers
SET target = (
  SELECT "target face"
  FROM archer_additions
  WHERE bib = archers.bib
);

ALTER TABLE archer_additions
DROP COLUMN "target face";
