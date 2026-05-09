![Example of the UI](https://codeberg.org/morosanmihail/hometg/raw/branch/main/images/ui20230628.jpg)

# How To Start

The following command spins up both the React webUI, as well as the backend axum server.

```bash
npm start
```

You can start them individually by doing the following for the webui:

```bash
npm run start-webui
```

Or the following for the backend server:

```bash
cargo run --bin server -- --system sql
```

The `--system` flag may be specified multiple times to enable multiple retrieval backends simultaneously. Supported values: `scryfall`, `sql`, `riftbound-sql`, `pokemon-sql`.

```bash
# Enable both MTG and Riftbound
cargo run --bin server -- --system sql --system riftbound-sql
```

You can then access the webui at `http://localhost:3000`.

## Config File

On first run with `--system`, the server writes a config file to `~/.local/share/gathers/server.toml`. The default port is `5234`; pass `--port` to use a different port. On subsequent runs, this config is loaded automatically and `--system`/`--port` are not required.

## Environment Variables

Environment variables override the paths set in the config file:

| Variable | Default | Description |
|---|---|---|
| `MTG_DB_PATH` | `~/.local/share/gathers/DB/AllPrintings.db` | MTG SQLite database (`sql` system) |
| `RIFTBOUND_DB_PATH` | `~/.local/share/gathers/DB/riftbound.db` | Riftbound SQLite database (`riftbound-sql` system) |
| `POKEMON_DB_PATH` | `~/.local/share/gathers/DB/pokemon.db` | Pokémon SQLite database (`pokemon-sql` system) |
| `STORAGE_DB_PATH` | `~/.local/share/gathers/DB/storage.db` | User collection database |

## Retrieval Database

The `sql` system requires the MTG database from www.mtgjson.com. You can trigger a background download via the `/mtg/update` endpoint:

```bash
curl http://localhost:5234/mtg/update -H "Accept: application/json"
```

Similarly, the Riftbound database can be updated via `/riftbound/update`.
