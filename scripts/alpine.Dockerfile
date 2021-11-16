ARG VERSION=local

FROM --platform=$BUILDPLATFORM rust:alpine AS fetcher

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

FROM rust:alpine AS builder

RUN apk add --no-cache fuse-dev libc-dev pkgconfig

ENV USER=root

COPY --from=fetcher /code /code

WORKDIR /code

RUN cargo fetch

COPY cli/src /code/cli/src
COPY fuse/src /code/fuse/src
COPY http-server/src /code/http-server/src
COPY lib/src /code/lib/src

RUN cargo build --offline --release
ARG VERSION

RUN mv /code/target/release/pcloud-cli /code/target/release/pcloud-cli-${VERSION}-musl \
  && mv /code/target/release/pcloud-fuse /code/target/release/pcloud-fuse-${VERSION}-musl \
  && mv /code/target/release/pcloud-http-server /code/target/release/pcloud-http-server-${VERSION}-musl

FROM --platform=$BUILDPLATFORM scratch AS artifact

COPY --from=builder /code/target/release/pcloud-cli-* /
COPY --from=builder /code/target/release/pcloud-fuse-* /
COPY --from=builder /code/target/release/pcloud-http-server-* /

FROM alpine AS cli

COPY --from=builder /code/target/release/pcloud-cli /usr/bin/pcloud-cli

ENTRYPOINT ["/usr/bin/pcloud-cli"]

FROM alpine AS fuse

RUN apk add --no-cache fuse

RUN mkdir /data

COPY --from=builder /code/target/release/pcloud-fuse /usr/bin/pcloud-fuse

ENTRYPOINT ["/usr/bin/pcloud-fuse"]
CMD ["/data"]

FROM alpine AS http-server

COPY --from=builder /code/target/release/pcloud-http-server /usr/bin/pcloud-http-server

ENTRYPOINT ["/usr/bin/pcloud-http-server"]
