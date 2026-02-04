CREATE TABLE collection(
  name TEXT PRIMARY KEY,
  can_remove BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE cards(
  uuid TEXT NOT NULL,
  collection TEXT NOT NULL,
  quantity INTEGER NOT NULL,
  foilquantity INTEGER NOT NULL,
  timeadded TEXT NULL,
  provider TEXT NOT NULL,
  PRIMARY KEY (uuid, collection)
);

CREATE UNIQUE INDEX idx_cards_uuid_collection ON cards(uuid, collection);

INSERT INTO collection (name, can_remove) VALUES ("Default", FALSE);
