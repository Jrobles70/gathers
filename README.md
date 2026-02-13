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

# Setup

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

