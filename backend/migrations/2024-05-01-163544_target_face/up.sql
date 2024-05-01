-- Your SQL goes here
ALTER TABLE archer_additions
ADD "target face" TEXT;

UPDATE archer_additions
SET "target face" = (
  SELECT target
  FROM archers
  WHERE bib = archer_additions.bib
);
UPDATE archers
SET "target" = "";
