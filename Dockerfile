# Common

ARG RUST_VERSION
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-alpine AS fetcher

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

RUN --mount=type=cache,sharing=locked,id=cargo-git,target=/usr/local/cargo/git \
  --mount=type=cache,sharing=locked,id=cargo-registry,target=/usr/local/cargo/registry \
  mkdir -p /code/.cargo \
  && cargo vendor > /code/.cargo/config.toml
  
COPY cli/src /code/cli/src
COPY lib/src /code/lib/src

# Cli related stages

ARG RUST_VERSION
FROM rust:${RUST_VERSION}-alpine AS cli-bin-builder

RUN apk add --no-cache musl-dev

COPY --from=fetcher /code /code

WORKDIR /code

RUN cargo build --offline --release --package pcloud-cli
RUN cp /code/target/release/pcloud-cli /pcloud-cli_$(uname -m)

FROM alpine AS cli-bin

COPY --from=cli-bin-builder /pcloud-cli_* /

FROM alpine AS cli-image

COPY --from=cli-bin-builder /code/target/release/pcloud-cli /pcloud-cli

ENTRYPOINT ["/pcloud-cli"]

FROM rust:${RUST_VERSION}-bullseye AS deb-builder

RUN --mount=type=cache,sharing=locked,id=apt-list,target=/var/lib/apt \
  --mount=type=cache,sharing=locked,id=apt-cache,target=/var/cache/apt \
  apt-get update \
  && apt-get install -y dpkg dpkg-dev liblzma-dev libfuse3-dev \
  && rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,sharing=locked,id=cargo-git,target=/usr/local/cargo/git \
  --mount=type=cache,sharing=locked,id=cargo-registry,target=/usr/local/cargo/registry \
  cargo install cargo-deb

COPY --from=fetcher /code /code
COPY cli/readme.md /code/cli/readme.md

WORKDIR /code

FROM deb-builder AS cli-deb-builder

RUN --mount=type=cache,sharing=locked,id=cargo-git,target=/usr/local/cargo/git \
  --mount=type=cache,sharing=locked,id=cargo-registry,target=/usr/local/cargo/registry \
  cargo deb --package pcloud-cli
  
FROM scratch AS cli-deb-package

COPY --from=cli-deb-builder /code/target/debian/*.deb /

# Http Server related stages

ARG RUST_VERSION
FROM rust:${RUST_VERSION}-alpine AS http-bin-builder

COPY --from=fetcher /code /code

RUN cargo build --offline --release --package pcloud-http-server

FROM alpine AS http-image

COPY --from=base /code/target/release/pcloud-http-server /pcloud-http-server

ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["/pcloud-http-server"]
