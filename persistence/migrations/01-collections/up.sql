CREATE TABLE collection(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE cards(
  uuid TEXT NOT NULL,
  collection TEXT NOT NULL,
  quantity INTEGER NOT NULL,
  foilquantity INTEGER NOT NULL,
  timeadded TEXT NULL,
  PRIMARY KEY (uuid, collection)
);
