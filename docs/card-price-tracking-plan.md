# Card Price Tracking Plan

## Goals

- Show MTG card prices without slowing search or card loading.
- Use Scryfall price data for MTG cards.
- Store only USD prices: regular, foil, and etched.
- Avoid making a Scryfall request every time a card appears.
- Batch Scryfall requests so the app does not spam the API.
- Keep the database from growing without bound.

## Current State

- Scryfall card responses include current prices in `prices.usd`, `prices.usd_foil`, and `prices.usd_etched`.
- The app already has MTG Scryfall IDs available through card identifiers.
- The UI already reserves a price footer with `$-`.
- The current card models and API responses do not expose price data.
- Collection storage currently tracks card id, collection, quantity, foil quantity, added time, and provider, but not price.

## Data Model

Use SQLite for the latest known price cache. This is durable app data and should live near the existing persistence/card data instead of requiring Redis.

```sql
CREATE TABLE card_price_cache (
  source TEXT NOT NULL,
  scryfall_id TEXT NOT NULL,
  usd_cents INTEGER NULL,
  usd_foil_cents INTEGER NULL,
  usd_etched_cents INTEGER NULL,
  fetched_at TEXT NOT NULL,
  PRIMARY KEY (source, scryfall_id)
);
```

Use a queue table if refresh work should survive server restarts and be shared by search warming and scheduled refreshes.

```sql
CREATE TABLE card_price_refresh_queue (
  source TEXT NOT NULL,
  scryfall_id TEXT NOT NULL,
  priority INTEGER NOT NULL DEFAULT 0,
  queued_at TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_attempt_at TEXT NULL,
  PRIMARY KEY (source, scryfall_id)
);
```

Do not add a history table for the first version. Add a `card_price_snapshots` table later only if charts or historical collection values become part of the feature.

## Search Flow

Search should never block on Scryfall.

1. Run the normal local card search.
2. Collect the Scryfall IDs from returned MTG cards.
3. Read cached prices with a single local SQL lookup using `WHERE scryfall_id IN (...)`.
4. Attach any cached prices to the API response.
5. Detect missing or stale prices.
6. Enqueue stale or missing Scryfall IDs for background refresh.
7. Return the search response immediately.

For search results, treat prices as fresh for one week. If a price is older than one week, return the stale cached value if available and enqueue a refresh in the background.

## Batching Strategy

All Scryfall work should go through one shared background worker.

The worker should:

- Deduplicate refresh work by `source + scryfall_id`.
- Wait briefly, around 250-500ms, before draining the queue so nearby search requests can combine into larger batches.
- Pull up to 75 Scryfall IDs at a time.
- Use Scryfall's collection endpoint with one request per batch:

```http
POST https://api.scryfall.com/cards/collection
```

```json
{
  "identifiers": [
    { "id": "scryfall-id-1" },
    { "id": "scryfall-id-2" }
  ]
}
```

- Parse each returned card's `prices.usd`, `prices.usd_foil`, and `prices.usd_etched`.
- Convert string prices to integer cents before storing.
- Upsert rows into `card_price_cache`.
- Remove successfully refreshed rows from `card_price_refresh_queue`.
- Increment attempts and keep failures queued for later retry.

Use one global rate limiter for all Scryfall price refreshes. Start conservatively at 1-2 batch requests per second. Scryfall asks clients to keep API traffic under 10 requests per second, so staying well below that gives the app room for other Scryfall calls.

On `429 Too Many Requests`, the worker should pause and back off. It should not retry aggressively.

## Scheduled Refresh

A daily refresh job can keep owned collection prices warm.

The job should:

1. Find distinct MTG cards in collections.
2. Resolve their Scryfall IDs.
3. Skip cards with fresh prices.
4. Enqueue stale cards into the same refresh queue used by search.

The daily job should not call Scryfall directly. It should share the same dedupe, batching, and rate limiting path as search warming.

For very large collections, consider using Scryfall bulk data instead of card-by-card refreshes. Bulk data aligns better with Scryfall guidance for large workloads.

## Redis Consideration

Redis is optional.

Recommended split:

- SQLite stores latest known prices.
- Redis can later store queue, dedupe, and rate-limit state if the app runs multiple server containers.

For the current single Unraid deployment, SQLite-only is simpler and avoids a required network dependency. If Redis is already desired, a useful Redis shape would be a sorted set:

```text
price_refresh_due
member = scryfall_id
score = next_refresh_timestamp
```

Even with Redis, the actual price cache should remain in SQLite so prices are durable and easy to query with card results.

## Cleanup

The cache table should stay small because it has one row per Scryfall card, not one row per refresh.

Cleanup rules:

- `card_price_cache` uses `PRIMARY KEY (source, scryfall_id)`, so refreshes upsert instead of append.
- Delete cache rows older than 90-180 days if the card is not in any collection.
- Keep cache rows for cards that are currently in a collection, even if old.
- `card_price_refresh_queue` uses `PRIMARY KEY (source, scryfall_id)`, so repeated searches cannot create duplicate queue rows.
- Delete queue rows after successful refresh.
- Delete failed queue rows after too many attempts or after 7-14 days.

Example cache pruning shape:

```sql
DELETE FROM card_price_cache
WHERE source = 'scryfall'
  AND fetched_at < datetime('now', '-180 days')
  AND scryfall_id NOT IN (
    SELECT DISTINCT ci.scryfallId
    FROM collection_cards_or_equivalent owned
    JOIN cardIdentifiers ci ON ci.uuid = owned.uuid
  );
```

The exact join will need to match the app's collection database and MTG card database layout.

## Implementation Phases

### Phase 1: Cache and API Surface

- Add `card_price_cache`.
- Add a USD-only price type to the backend API response.
- Join cached prices into MTG card results.
- Replace the `$-` UI placeholder when cached price data exists.

### Phase 2: Background Batch Worker

- Add `card_price_refresh_queue`.
- Enqueue missing or stale prices from search.
- Implement the shared Scryfall batch worker.
- Use the Scryfall collection endpoint with batches up to 75 IDs.
- Add rate limiting and `429` backoff.

### Phase 3: Scheduled Refresh

- Add a daily refresh task for distinct cards in collections.
- Enqueue stale collection cards through the same queue.
- Add cleanup for old cache and abandoned queue rows.

## Open Questions

- Should collection totals use regular price for normal quantity and foil price for foil quantity only?
- Should etched prices be displayed in the UI now, or only stored for future finish-specific quantity tracking?
- Should the first version use a durable SQLite queue or an in-memory queue to reduce initial schema work?
