The CLI tool allows you to search for Magic: the Gathering cards using various filters.
It supports multiple backends: Scryfall (online), MTG SQLite (local), Riftbound SQLite, and Pokémon SQLite.

![Example of the CLI Tool](https://codeberg.org/morosanmihail/gathers/raw/branch/main/images/cli1.png)

## Running

```bash
cargo run --bin gathers
```

## Setup

Run this command to download the latest version of the MTG SQLite database from www.mtgjson.com.
They are a wonderful service and worth supporting.

```bash
cargo run --bin gathers -- --download
```

## Environment Variables

The CLI uses a single `RETRIEVAL_DB_PATH` environment variable to locate the database file, regardless of which system is selected. If unset, it falls back to a default filename in the current directory.

| Variable | Default | Description |
|---|---|---|
| `RETRIEVAL_DB_PATH` | `AllPrintings.db` / `riftbound.db` / `pokemon.db` | Path to the SQLite database for the selected system |

## Examples

Here are some common ways to use the CLI:

### 1. Basic Search by Name

Search for cards by name:

```bash
cargo run --bin gathers -- --name "Lightning Bolt"
```

### 2. Search with Color Filter

Find cards of a specific color:

```bash
cargo run --bin gathers -- --name "Shock" --color R
```

### 3. Search with Rarity Filter

Find rare cards:

```bash
cargo run --bin gathers -- --name "Black Lotus" --rarity Rare
```

### 4. Search with Set Filter

Find cards from a specific set:

```bash
cargo run --bin gathers -- --name "Force of Will" --set dmr
```

### 5. Search with Multiple Filters

Combine multiple filters for precise searches:

```bash
cargo run --bin gathers -- --name "Counterspell" --color U --rarity Rare
```

### 6. Using SQLite Backend

Search using a local MTG SQLite database:

```bash
cargo run --bin gathers -- --system sql --name "Dark Ritual"
```

### 7. Using Riftbound Backend

Search Riftbound cards by domain:

```bash
cargo run --bin gathers -- --system riftbound-sql --domain Fire
```

### 8. Using Pokémon Backend

Search Pokémon cards by energy type:

```bash
cargo run --bin gathers -- --system pokemon --name "Pikachu" --energy Electric
```

## Available Filters

The CLI supports the following filters:

- `--system <system>`: Choose backend (`scryfall`, `sql`, `riftbound-sql`, `pokemon`). Default: `scryfall`.
- `--name <string>`: Search by card name
- `--color <colors>`: Filter by color (W, U, B, R, G, C) — MTG only
- `--limit <number>`: Number of results to return (default: 10)
- `--offset <number>`: Starting offset for results (default: 0)
- `--set <string>`: Filter by set code
- `--collector-number <string>`: Filter by collector number
- `--artist <string>`: Filter by artist
- `--text <string>`: Filter by card text
- `--rarity <string>`: Filter by rarity (Common, Uncommon, Rare, Mythic, Special, Bonus)
- `--subtype <strings>`: Filter by subtypes
- `--supertype <string>`: Filter by supertypes
- `--types <strings>`: Filter by types
- `--domain <strings>`: Filter by domain — Riftbound only
- `--energy <strings>`: Filter by energy type — Pokémon only
- `--download`: Download/update the MTG database from mtgjson.com
