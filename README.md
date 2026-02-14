# gathers

Collection of Rust crates and binaries to help one search for and manage Magic: the Gathering (TM) cards. 

## retrieval

Find cards using search criteria.

Backends available:
- mtgjson SQLite DB
- Scryfall.com

## persistence

Persist a collection of cards to a database.

Also allows you to manipulate the collections.

## server

REST server to leverage the `retrieval` and `persistence` crates.

## webui

A React web ui to interact with the server.
It is quite ancient, from when `gathers` was actually `hometg` and written in C#. 

## cli

Toy application to leverage the `retrieval` crates.

# Codeberg vs Github

This repo is both on Codeberg and on Github. 

I will read issues from Github, but Github is only a mirror.
Main development happens on Codeberg.
Support small tech!

# CLI Usage

The CLI tool allows you to search for Magic: the Gathering cards using various filters. It supports two backends: Scryfall (online) and SQLite (local database).

## Installation

First, build the CLI binary:

```bash
cargo build --release --bin cli
```

Or run it directly:

```bash
cargo run --bin cli
```

## Setup

Run this command to download the latest version of the Sqlite database from www.mtgjson.com. 
They are a wonderful service and worth supporting. 

```bash
cargo run --bin cli -- --download
```

I am still stabilising "where stuff gets saved". Bear with me please.

## Examples

Here are some common ways to use the CLI:

### 1. Basic Search by Name

Search for cards by name:

```bash
cargo run --bin cli -- --name "Lightning Bolt"
```

### 2. Search with Color Filter

Find cards of a specific color:

```bash
cargo run --bin cli -- --name "Shock" --color R
```

### 3. Search with Rarity Filter

Find rare cards:

```bash
cargo run --bin cli -- --name "Black Lotus" --rarity Rare
```

### 4. Search with Set Filter

Find cards from a specific set:

```bash
cargo run --bin cli -- --name "Force of Will" --set dmr
```

### 5. Search with Multiple Filters

Combine multiple filters for precise searches:

```bash
cargo run --bin cli -- --name "Counterspell" --color U --rarity Rare
```

### 6. Using SQLite Backend

Search using a local SQLite database:

```bash
cargo run --bin cli -- --system sql --name "Dark Ritual"
```

## Available Filters

The SQLite backend requires the `AllPrintings.db` file to be present in the current directory. You can download this file from [mtgjson.com](https://www.mtgjson.com/).

## Available Filters

The CLI supports the following filters:

- `--name <string>`: Search by card name
- `--color <colors>`: Filter by color (W, U, B, R, G, C, M)
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
- `--system <system>`: Choose backend (Scryfall or Sql)

## Setup

## Local

`cargo run --bin cli -- --help` will be a good starting point for using the CLI and search for cards.

`npm start` will spin up both the webui and the server. 

## Docker Setup

To run the server using Docker, you can use the provided Dockerfile and docker-compose.yml:

### Using Docker Compose (Recommended)

0. Make sure to edit the docker-compose.yml file and point it to the right volume mount. Then download a AllPrintings.db from www.mtgjson.com and save it in there. 
I will fix this soon to auto-download on first start if the file does not exist yet.

1. Build and start the server:
   ```bash
   docker-compose up -d
   ```

2. The server will be available at `http://localhost:3000`

3. To stop the server:
   ```bash
   docker-compose down
   ```

### Database Persistence

The Docker setup uses volume mounting to persist databases.

These files will be created automatically in the `data` directory when you start the container for the first time.

### Environment Variables

The Docker container sets the following environment variables:
- `STORAGE_DB_PATH`: Path to the storage database
- `RETRIEVAL_DB_PATH`: Path to the retrieval database

The default ports are:
- 3000: Server port

### Building Manually

If you want to build the Docker image manually:
```bash
docker build -t gathers-server .
```
