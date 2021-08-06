FROM --platform=$BUILDPLATFORM rust:alpine AS vendor

WORKDIR /code-tmp
RUN cargo init
RUN mkdir /code
RUN cp -r /code-tmp /code/lib
RUN cp -r /code-tmp /code/proxy

COPY Cargo.toml /code/
COPY Cargo.lock /code/
COPY lib/Cargo.toml /code/lib/
COPY proxy/Cargo.toml /code/proxy/

WORKDIR /code
RUN mkdir -p /code/.cargo && cargo vendor > /code/.cargo/config

FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

COPY --from=vendor /code /code
COPY ./lib /code/lib
COPY ./proxy /code/proxy
WORKDIR /code
RUN cargo build --package pcloud-proxy --offline --release

FROM alpine:latest

ENV HOST=0.0.0.0
ENV PORT=3000
ENV RUST_LOG=info

COPY --from=builder /code/target/release/pcloud-proxy /usr/bin/pcloud-proxy

EXPOSE 3000

ENTRYPOINT [ "/usr/bin/pcloud-proxy" ]
