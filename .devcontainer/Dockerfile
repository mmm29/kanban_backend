FROM mcr.microsoft.com/vscode/devcontainers/rust:bullseye

# psql
RUN apt-get update && apt-get install -y postgresql-client

# nightly rust
RUN rustup default nightly
RUN rustup update
RUN rustup component add rust-src clippy rustfmt