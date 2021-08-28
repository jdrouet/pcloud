FROM rust:alpine AS base

RUN apk add --no-cache libc-dev

WORKDIR /code/cli
RUN cargo init

WORKDIR /code/http-server
RUN cargo init

WORKDIR /code/lib
RUN cargo init --lib

WORKDIR /code
COPY Cargo.lock Cargo.toml /code/
COPY cli/Cargo.toml /code/cli/Cargo.toml
COPY http-server/Cargo.toml /code/http-server/Cargo.toml
COPY lib/Cargo.toml /code/lib/Cargo.toml

RUN cargo fetch

COPY lib/src /code/lib/src
COPY http-server/src /code/http-server/src

RUN cargo build --offline --release --package pcloud-http-server

FROM alpine

COPY --from=base /code/target/release/pcloud-http-server /pcloud-http-server

ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["/pcloud-http-server"]
