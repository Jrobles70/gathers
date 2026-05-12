To run the server using Docker, you can use the provided Dockerfile or docker-compose.yml:

## Using Docker Compose (Recommended)

1. Build and start the server:

   ```bash
   docker compose up -d --build
   ```

2. The MTG database will be auto-downloaded on first start if not already present.

3. The web UI will be available at `http://localhost:3000`. The API server listens on port 5234.

4. To stop the server:
   ```bash
   docker compose down
   ```

## Database Persistence

The Docker setup uses a named volume (`gathers-data`) to persist all databases across container restarts.

## First-Run Arguments

On first run, pass at least one retrieval system. The API defaults to port `5234`, so Docker/Unraid post arguments can be as simple as:

```bash
--system sql
```

Supported systems are `scryfall`, `sql`, `riftbound-sql`, and `pokemon-sql`.

The provided `docker-compose.yml` passes `--system sql` to the API container:

```yaml
command: ["--system", "sql"]
```

That enables the MTGJSON SQLite backend and lets a fresh Docker volume bootstrap without manually adding arguments.

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

## Local Setup Notes from 2026-05-08

This is what was done to get the local Docker stack running:

1. Updated `docker-compose.yml` so `gathers-api` starts with `--system sql`. Without this, a fresh container exits on first run because no `server.toml` exists yet and the server requires at least one retrieval system.

2. Built and started both services:

   ```bash
   docker compose up -d --build
   ```

3. Let the API container download the MTGJSON `AllPrintings.sqlite` database into the named Docker volume `gathers-data`, mounted at `/home/app/.local/share/gathers/` in the container. The target database path is:

   ```text
   /home/app/.local/share/gathers/DB/AllPrintings.db
   ```

4. The first MTGJSON download attempt failed with `error decoding response body`, before a verified database was installed. Restarting the API retried the built-in download:

   ```bash
   docker compose restart gathers-api
   ```

5. The retry completed successfully. The API log reported:

   ```text
   AllPrintings.db downloaded and verified (CRC: 7c241ea4af9633ec09e5908c84f163a63b653600c714f580bd697397d565adc1).
   MTG DB ready.
   ```

6. Verified the API loaded the MTG SQLite backend:

   ```bash
   curl -sS http://127.0.0.1:5234/system
   ```

   Expected response:

   ```json
   {"system":"MagicSQLite","systems":["MagicSQLite"],"downloading":{}}
   ```

7. Verified card search against the downloaded database:

   ```bash
   curl -sS -H 'Content-Type: application/json' \
     -d '{"name":"Black Lotus"}' \
     'http://127.0.0.1:5234/mtg/cards/search?limit=3'
   ```

   This returned Black Lotus printings from the MTGJSON SQLite database.

8. Verified the web UI service:

   ```bash
   docker compose ps
   docker exec gathers-gathers-webui-1 wget -q -O - http://127.0.0.1:3000/
   ```

   `docker compose ps` showed:

   ```text
   gathers-gathers-api-1     Up   0.0.0.0:5234->5234/tcp
   gathers-gathers-webui-1   Up (healthy)   0.0.0.0:3000->3000/tcp
   ```

   The in-container web UI check returned the production React bundle, including `main.8c3f103f.js`.
