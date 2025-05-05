FROM --platform=$BUILDPLATFORM rust:alpine AS fetcher

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

RUN mkdir -p /code/.cargo \
    && cargo vendor > /code/.cargo/config

FROM rust:alpine AS builder

RUN apk add --no-cache libc-dev musl-dev pkgconfig

ENV USER=root

COPY --from=fetcher /code /code

WORKDIR /code

COPY cli/src /code/cli/src
COPY http-server/src /code/http-server/src
COPY lib/src /code/lib/src
COPY lib/readme.md /code/lib/

RUN cargo build --offline --release

RUN cp /code/target/release/pcloud-cli /code/target/release/pcloud-cli-$(/code/target/release/pcloud-cli --version | cut -d ' ' -f 2)-$(uname -m)-musl
RUN cp /code/target/release/pcloud-http-server /code/target/release/pcloud-http-server-$(/code/target/release/pcloud-http-server --version | cut -d ' ' -f 2)-$(uname -m)-musl

FROM --platform=$BUILDPLATFORM scratch AS artifact

COPY --from=builder /code/target/release/pcloud-cli-* /
COPY --from=builder /code/target/release/pcloud-http-server-* /

FROM alpine AS cli-image

COPY --from=builder /code/target/release/pcloud-cli /usr/bin/pcloud-cli

ENTRYPOINT ["/usr/bin/pcloud-cli"]

FROM alpine AS http-server-image

COPY --from=builder /code/target/release/pcloud-http-server /usr/bin/pcloud-http-server

ENTRYPOINT ["/usr/bin/pcloud-http-server"]
