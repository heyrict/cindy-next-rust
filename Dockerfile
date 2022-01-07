FROM rust:slim-buster as build_env
ENV HOME /home/cindy
# Use cargo mirrors
RUN mkdir -p $HOME/.cargo/ $HOME/cindy-next-rust
#RUN echo "[source.crates-io]\nreplace-with = 'tuna'\n[source.tuna]\nregistry = 'https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git'" > $HOME/.cargo/config
# Update dependent dynamic libraries
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libpq-dev libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies
WORKDIR /home/cindy/cindy-next-rust
COPY Cargo.toml .
RUN mkdir $HOME/cindy-next-rust/src && echo 'fn main{}' > src/main.rs && cargo fetch && rm src/main.rs

# Build packages
FROM build_env as builder
ENV HOME /home/cindy
WORKDIR /home/cindy/cindy-next-rust
COPY . .
RUN cargo build --release && strip target/release/cindy-next-rust

# Build utility binaries
FROM build_env as util_builder
ENV HOME /home/cindy
RUN cargo install diesel_cli --no-default-features --features "postgres"
RUN cargo install just

# Run the server
FROM debian:buster-slim
COPY --from=builder /home/cindy/cindy-next-rust/target/release/cindy-next-rust /usr/bin
COPY --from=util_builder /usr/local/cargo/bin/diesel /usr/bin
COPY --from=util_builder /usr/local/cargo/bin/just /usr/bin
CMD ["/usr/bin/cindy-next-rust"]
