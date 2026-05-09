To run the server using Docker, you can use the provided Dockerfile or docker-compose.yml:

## Using Docker Compose (Recommended)

1. Build and start the server:

   ```bash
   docker-compose up -d
   ```

2. The MTG database will be auto-downloaded on first start if not already present.

3. The web UI will be available at `http://localhost:3000`. The API server listens on port 5234.

4. To stop the server:
   ```bash
   docker-compose down
   ```

## Database Persistence

The Docker setup uses a named volume (`gathers-data`) to persist all databases across container restarts.

## First-Run Arguments

On first run, pass at least one retrieval system. The API defaults to port `5234`, so Docker/Unraid post arguments can be as simple as:

```bash
--system sql
```

Supported systems are `scryfall`, `sql`, `riftbound-sql`, and `pokemon-sql`.

## Environment Variables

The Docker container supports the following environment variables (all set by default in docker-compose.yml):

| Variable | Default (in container) | Description |
|---|---|---|
| `MTG_DB_PATH` | `/home/app/.local/share/gathers/DB/AllPrintings.db` | MTG SQLite database |
| `RIFTBOUND_DB_PATH` | `/home/app/.local/share/gathers/DB/riftbound.db` | Riftbound SQLite database |
| `POKEMON_DB_PATH` | `/home/app/.local/share/gathers/DB/pokemon.db` | Pokémon SQLite database |
| `STORAGE_DB_PATH` | `/home/app/.local/share/gathers/DB/storage.db` | User collection database |

## Ports

- `3000`: Web UI
- `5234`: API server

## Building Manually

If you want to build the Docker image manually:

```bash
docker build -t gathers-server .
```
