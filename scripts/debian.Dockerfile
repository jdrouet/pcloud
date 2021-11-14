FROM --platform=$BUILDPLATFORM rust:slim-bullseye AS fetcher

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
COPY fuse/Cargo.toml /code/fuse/Cargo.toml
COPY http-server/Cargo.toml /code/http-server/Cargo.toml
COPY lib/Cargo.toml /code/lib/Cargo.toml

RUN mkdir -p /code/.cargo \
  && cargo vendor > /code/.cargo/config

FROM rust:slim-bullseye AS builder

RUN apt-get update && apt-get install -y gcc libfuse-dev pkg-config

ENV USER=root

COPY --from=fetcher /code /code

WORKDIR /code

RUN cargo fetch

COPY cli/src /code/cli/src
COPY fuse/src /code/fuse/src
COPY http-server/src /code/http-server/src
COPY lib/src /code/lib/src

FROM builder as builder-all

RUN cargo build --offline --release

FROM builder as builder-cli

RUN cargo build --offline --release --package pcloud-cli

FROM --platform=$BUILDPLATFORM scratch AS artifact

COPY --from=builder-all /code/target/release/pcloud-cli /pcloud-cli
COPY --from=builder-all /code/target/release/pcloud-fuse /pcloud-fuse
COPY --from=builder-all /code/target/release/pcloud-http-server /pcloud-http-server

FROM debian:bullseye-slim AS cli

COPY --from=builder-cli /code/target/release/pcloud-cli /usr/bin/pcloud-cli

ENTRYPOINT ["/usr/bin/pcloud-cli"]

FROM builder as builder-fuse

RUN cargo build --offline --release --package pcloud-fuse

FROM debian:bullseye-slim AS fuse

RUN apt-get update \
  && apt-get install -y fuse \
  && rm -rf /var/lib/apt/lists/*

RUN mkdir /data

COPY --from=builder-fuse /code/target/release/pcloud-fuse /usr/bin/pcloud-fuse

ENTRYPOINT ["/usr/bin/pcloud-fuse"]
CMD ["/data"]

FROM builder as builder-http-server

RUN cargo build --offline --release --package pcloud-http-server

FROM debian:bullseye-slim AS http-server

COPY --from=builder-http-server /code/target/release/pcloud-http-server /usr/bin/pcloud-http-server

ENTRYPOINT ["/usr/bin/pcloud-http-server"]
