# cindy-next-rust

Graphql backend for [cindy-next](https://github.com/heyrict/cindy-next).

It is based on `async-graphql`, `actix-web`, `diesel`, powered by Rust.

## Features

- [x] Dynamic query building with `diesel` (with support for complex filtering)
- [x] Graphql interface
- [x] Realtime Subscriptions
- [x] Authorization
- [x] Access control
- [ ] Relay-like pagination (not included)

## Dev-Dependencies

- **Rust**, first of all. Follow the instructions on rustup.rs if you don't have one. The minimum supported version is v1.46.

    ```sh
    # Install rustup
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    # Install latest rust and cargo
    rustup install nightly
    ```

- **Postgresql ≥ 12**, and its dev library.

    ```sh
    # Debian-based OS
    sudo apt-get install postgresql libpq-dev
    # Arch linux
    sudo pacman -S postgresql postgresql-libs
    ```

- **Diesel client**. You need to compile this yourself

    ```sh
    cargo install diesel_cli --no-default-features --features "postgres"
    ```

- **just** (optional), to run recipes defined in `justfile`.

    ```sh
    cargo install just
    ```

## Quickstart

Basically only two binaries (`cindy-next-rust` and `diesel`) are required in the server. Currently we do not provide compiled binaries. It is recommended to compile them yourself and push the binaries to the server.

- Clone the repo with `git clone https://github.com/heyrict/cindy-next-rust`.
- Create an empty database for *Cindy* with `sudo -u postgres psql`

    ```postgresql
    CREATE ROLE cindy LOGIN PASSWORD 'cindy-password';
    ALTER ROLE cindy SET client_encoding TO 'utf8';
    ALTER ROLE cindy SET timezone TO 'UTC';
    CREATE DATABASE cindy-db;
    GRANT ALL ON DATABASE cindy-db TO cindy;
    \c cindy-db;
    CREATE EXTENSION pgcrypto;
    CREATE EXTENSION "uuid-ossp";
    ```

- Copy `.env.example` to `.env` and edit it based on your flavor.

  Make sure `DATABASE_URL` in the config file points to your postgres instance.
  If you followed the steps above, it is `postgres://cindy:cindy-password@127.0.0.1:5432/cindy-db`.

- Setup the database with `diesel database setup && diesel migration run`.

- Run `cargo run --release` or `./path/to/cindy-next-rust` if you have a compiled binary to start the server. For the former command, once compiled, it can be found in `./target/release/cindy-next-rust`.

- Create an admin account with `just signup`.

- Load initial data to the database with `psql cindy < setup/jp/initdb.up.sql`.
