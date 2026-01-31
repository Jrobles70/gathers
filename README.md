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

Compatible with the `hometg` React frontend.
For better or worse (the API is not the cleanest).

## cli

Toy application to leverage the `retrieval` crates.

# Codeberg vs Github

This repo is both on Codeberg and on Github. 

I will read issues from Github, but Github is only a mirror.
Main development happens on Codeberg.
Support small tech!
