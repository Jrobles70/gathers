# GatheRs

A card collection management system supporting Magic: The Gathering, Riftbound, and Pokemon. Built with a Rust backend and React web UI.

## Features

- **Multi-game support** — MTG (via Scryfall API or local SQLite), Riftbound, and Pokemon
- **Collection management** — Create and organize multiple collections with subcollection support
- **Quantity tracking** — Track regular and foil quantities per card
- **Price tracking** — Current market prices via Scryfall integration
- **Purchase history** — Record purchase prices and sources
- **Proxy support** — Link alternative printings to cards in your collection
- **CSV import/export** — Bulk import and export collections
- **OpenAPI/Swagger** — Auto-generated API docs at `/swagger`

## Tech Stack

- **Backend**: Rust (Axum, Tokio, SQLite via rusqlite)
- **Frontend**: React 18, React Bootstrap, React Router
- **Database**: SQLite (collections + local card databases)
- **Containerization**: Docker / Docker Compose

## Project Structure

```
gathers/        CLI binary for card searching
server/         REST API backend
models/         Shared data models (MTG, Pokemon, Riftbound)
retrieval/      Card data retrieval (Scryfall, SQLite, Pokemon, Riftbound)
persistence/    Collection storage and CSV import/export
webui/          React frontend
```

## Getting Started

### Docker (recommended)

```bash
docker-compose up
```

- API: `http://localhost:5234`
- Web UI: `http://localhost:3000`

### Local Development

```bash
# Start both server and web UI
npm run start

# Or separately
npm run start-webui
cargo run --bin server -- --system sql --port 5234
```

### CLI

```bash
# Search cards via Scryfall
cargo run --bin gathers -- --system scryfall --name "Black Lotus" --limit 5

# Download/update local MTG database
cargo run --bin gathers -- --system sql --download
```

## Configuration

The server reads from `~/.local/share/gathers/server.toml` on first run:

```toml
system = ["sql"]   # Options: "scryfall", "sql", "riftbound-sql", "pokemon-sql"
port = 5234
mtg_db_path = "~/.local/share/gathers/DB/AllPrintings.db"
riftbound_db_path = "~/.local/share/gathers/DB/riftbound.db"
pokemon_db_path = "~/.local/share/gathers/DB/pokemon.db"
storage_db_path = "~/.local/share/gathers/DB/storage.db"
```

Databases are downloaded automatically on startup. Set `GATHERS_NO_AUTO_UPDATE=1` to skip this.

### Environment Variable Overrides

| Variable | Description |
|---|---|
| `MTG_DB_PATH` | Path to MTG SQLite database |
| `RIFTBOUND_DB_PATH` | Path to Riftbound database |
| `POKEMON_DB_PATH` | Path to Pokemon database |
| `STORAGE_DB_PATH` | Path to collections database |
| `GATHERS_NO_AUTO_UPDATE` | Skip automatic DB downloads |

## API Overview

| Prefix | Description |
|---|---|
| `POST /mtg/cards/search` | Search MTG cards |
| `POST /pokemon/cards/search` | Search Pokemon cards |
| `POST /riftbound/cards/search` | Search Riftbound cards |
| `/collection/*` | Full collection CRUD, import/export, price stats |
| `GET /system` | Active systems and download progress |
| `GET /swagger` | Interactive API documentation |
| `GET /api.json` | OpenAPI spec |
