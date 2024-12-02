FROM rust:1.82.0 as builder
WORKDIR /app

RUN rustup default nightly

COPY . .

RUN cargo install --path .

FROM ubuntu:latest

RUN apt-get update

COPY --from=builder /usr/local/cargo/bin/backend /usr/local/bin/backend
COPY Rocket.toml /

CMD ["backend"]