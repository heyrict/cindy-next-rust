# cindy-next-rust

An experimental graphql backend for [cindy-next](https://github.com/heyrict/cindy-next).

It is based on `async-graphql`, `actix-web`, `diesel` now but may change in future versions.

## Features

- [x] Dynamic query building with `diesel` (with limitations on complex filtering)
- [x] Graphql interface
- [ ] Authorization
- [ ] Access control
- [ ] Relay-like pagination (it won't be included here, but you can build that with the code here!)

## Quickstart

- Make sure you have exact the same database schema as the latest `cindy-next` commit. (Migration scripts to be come)
- Run `DATABASE_URL=... cargo run` to run the server
