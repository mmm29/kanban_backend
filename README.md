## About
Backend for kanban-todo-app

## Installation

### Manual

* Install nightly Rust 1.86 or newer.
* Install PostgreSQL 16 or newer.
* Clone the repo
    ```bash
    git clone https://github.com/mmm29/kanban_backend
    cd kanban_backend
    ```
* Build the API server
    ```bash
    cargo build --release
    ```
    The binary will be at `target/release/bakend`.
* Create the database and initialize it with an SQL script at `postgresql/init-schema.sql`.
* Set `DATABASE` environment variable
    ```bash
    export DATABASE=postgres://user:password@host:port/db
    ```
    where
    - `user` - the user of the database
    - `password` - the password of the user
    - `host` - the host of the database, may be `localhost`
    - `port` - the port of the database, by default `5432`
    - `db` - the name of the database

* Run the API server
    ```bash
    ./target/release/bakend
    ```

The API server will be listening on port `80`.

### Dev container

Run the devcontainer from `.devcontainer/devcontainer.json`.

## Run
Run with in-memory storage:
```bash
cargo run
```

To enable debug logging set `RUST_LOG=debug` environment variable:
```bash
RUST_LOG=debug cargo run
```

To enable PostgreSQL persistent storage set `DATABASE` environment variable:
```bash
DATABASE=postgres://user:password@host:port/name cargo run
```

## How to write documentation
Follow the guidelines described in [the official Rust documentation](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html).