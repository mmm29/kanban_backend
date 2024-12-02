FROM mcr.microsoft.com/devcontainers/rust:latest

# psql
RUN apt-get update && apt-get install -y postgresql-client

# nightly rust
RUN rustup default nightly