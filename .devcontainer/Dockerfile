FROM mcr.microsoft.com/devcontainers/rust:bullseye

# psql
RUN apt-get update && apt-get install -y postgresql-client

# nightly rust
RUN rustup default nightly && rustup component add clippy rustfmt