ARG RUST_VERSION=1.58.1
FROM rust:${RUST_VERSION}-bullseye AS base

RUN --mount=type=cache,sharing=locked,id=apt-list,target=/var/lib/apt \
  --mount=type=cache,sharing=locked,id=apt-cache,target=/var/cache/apt \
  apt-get update \
  && apt-get install -y libfuse3-dev \
    gcc-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
  && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add armv7-unknown-linux-gnueabihf

FROM base AS bin-builder

WORKDIR /code/cli
RUN cargo init

WORKDIR /code/fuse
RUN cargo init

WORKDIR /code/http-server
RUN cargo init

WORKDIR /code/lib
RUN cargo init --lib

WORKDIR /code
COPY .cargo/config.toml /code/.cargo/config.toml
COPY Cargo.lock Cargo.toml /code/
COPY cli/Cargo.toml /code/cli/Cargo.toml
COPY http-server/Cargo.toml /code/http-server/Cargo.toml
COPY lib/Cargo.toml /code/lib/Cargo.toml

RUN --mount=type=cache,sharing=locked,id=cargo-git,target=/usr/local/cargo/git \
  --mount=type=cache,sharing=locked,id=cargo-registry,target=/usr/local/cargo/registry \
  cargo vendor >> /code/.cargo/config.toml
  
COPY cli/src /code/cli/src
COPY cli/readme.md /code/cli/readme.md
COPY lib/src /code/lib/src
COPY http-server/src /code/http-server/src

FROM base AS deb-builder

RUN --mount=type=cache,sharing=locked,id=apt-list,target=/var/lib/apt \
  --mount=type=cache,sharing=locked,id=apt-cache,target=/var/cache/apt \
  apt-get update \
  && apt-get install -y \
    dpkg dpkg-dev liblzma-dev \
  && rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,sharing=locked,id=cargo-git,target=/usr/local/cargo/git \
  --mount=type=cache,sharing=locked,id=cargo-registry,target=/usr/local/cargo/registry \
  cargo install cargo-deb

COPY --from=bin-builder /code /code

WORKDIR /code

FROM bin-builder AS amd64-bin

RUN cargo build --release --package pcloud-cli --target x86_64-unknown-linux-gnu

FROM deb-builder AS amd64-deb

RUN cargo deb --package pcloud-cli --target x86_64-unknown-linux-gnu

FROM bin-builder AS arm64-bin

RUN cargo build --release --package pcloud-cli --target aarch64-unknown-linux-gnu

FROM deb-builder AS arm64-deb

RUN cargo deb --package pcloud-cli --target aarch64-unknown-linux-gnu

FROM bin-builder AS armv7-bin

RUN cargo build --release --package pcloud-cli --target armv7-unknown-linux-gnueabihf

FROM deb-builder AS armv7-deb

RUN cargo deb --package pcloud-cli --target armv7-unknown-linux-gnueabihf

FROM scratch AS amd64-deb-package

COPY --from=amd64-deb /code/target/x86_64-unknown-linux-gnu/debian/*.deb /

FROM scratch AS amd64-bin-package

COPY --from=amd64-bin /code/target/x86_64-unknown-linux-gnu/release/pcloud-cli /pcloud-cli_amd64

FROM scratch AS arm64-deb-package

COPY --from=arm64-deb /code/target/aarch64-unknown-linux-gnu/debian/*.deb /

FROM scratch AS arm64-bin-package

COPY --from=arm64-bin /code/target/aarch64-unknown-linux-gnu/release/pcloud-cli /pcloud-cli_arm64

FROM scratch AS all-packages

COPY --from=amd64-bin /code/target/x86_64-unknown-linux-gnu/release/pcloud-cli /pcloud-cli_amd64
COPY --from=amd64-deb /code/target/x86_64-unknown-linux-gnu/debian/*.deb /
COPY --from=arm64-bin /code/target/aarch64-unknown-linux-gnu/release/pcloud-cli /pcloud-cli_arm64
COPY --from=arm64-deb /code/target/aarch64-unknown-linux-gnu/debian/*.deb /
COPY --from=armv7-bin /code/target/armv7-unknown-linux-gnueabihf/release/pcloud-cli /pcloud-cli_armv7
COPY --from=armv7-deb /code/target/armv7-unknown-linux-gnueabihf/debian/*.deb /
