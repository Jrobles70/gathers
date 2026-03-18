ALTER TABLE cards ADD COLUMN timeupdated TEXT NULL;
UPDATE cards SET timeupdated = timeadded;
