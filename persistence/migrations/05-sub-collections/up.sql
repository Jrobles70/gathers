ALTER TABLE collection ADD COLUMN parent TEXT REFERENCES collection(name) NULL;
