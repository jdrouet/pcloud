FROM rust:alpine AS base

RUN apk add --no-cache libc-dev

WORKDIR /code/cli
RUN cargo init

WORKDIR /code/fuse
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

COPY cli/src /code/cli/src
COPY lib/src /code/lib/src

RUN cargo build --offline --release --package pcloud-cli

FROM alpine

COPY --from=base /code/target/release/pcloud-cli /pcloud-cli

ENTRYPOINT ["/pcloud-cli"]
