CREATE TABLE card_price_cache (
  source TEXT NOT NULL,
  scryfall_id TEXT NOT NULL,
  usd_cents INTEGER NULL,
  usd_foil_cents INTEGER NULL,
  usd_etched_cents INTEGER NULL,
  fetched_at TEXT NOT NULL,
  PRIMARY KEY (source, scryfall_id)
);

CREATE TABLE card_price_refresh_queue (
  source TEXT NOT NULL,
  scryfall_id TEXT NOT NULL,
  priority INTEGER NOT NULL DEFAULT 0,
  queued_at TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_attempt_at TEXT NULL,
  PRIMARY KEY (source, scryfall_id)
);

ALTER TABLE cards ADD COLUMN purchase_price_cents INTEGER NULL;
ALTER TABLE cards ADD COLUMN purchase_price_source TEXT NULL;
ALTER TABLE cards ADD COLUMN purchase_price_updated_at TEXT NULL;
