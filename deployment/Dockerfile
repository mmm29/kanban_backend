FROM rust:1.86-bullseye AS builder

WORKDIR /usr/src/backend
COPY . .

RUN rustup default nightly

RUN cargo install --path .

FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/backend /usr/local/bin/backend
COPY ../Rocket.toml .

CMD ["backend"]
