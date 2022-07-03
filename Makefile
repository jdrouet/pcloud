EXTRA_ARGS?=
RUST_VERSION?=1.62.0
CLI_VERSION=$(shell cat cli/Cargo.toml | docker run -i --rm ghcr.io/tomwright/dasel:latest -p toml '.package.version')
FUSE_VERSION=$(shell cat fuse/Cargo.toml | docker run -i --rm ghcr.io/tomwright/dasel:latest -p toml '.package.version')
HTTP_SERVER_VERSION=$(shell cat http-server/Cargo.toml | docker run -i --rm ghcr.io/tomwright/dasel:latest -p toml '.package.version')

publish:
	cd lib && cargo publish ${EXTRA_ARGS}
	cd cli && cargo publish ${EXTRA_ARGS}
	cd fuse && cargo publish ${EXTRA_ARGS}
	cd http-server && cargo publish ${EXTRA_ARGS}

build-artifacts:
	docker buildx build \
		-f scripts/debian.Dockerfile \
		--platform linux/amd64,linux/arm64,linux/arm/v7 \
		--target artifact \
		--output type=local,dest=$(shell pwd)/target \
		.
	docker buildx build \
		-f scripts/alpine.Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--target artifact \
		--output type=local,dest=$(shell pwd)/target \
		.

build-images: build-image-cli build-image-fuse build-image-http-server

build-image-cli: build-image-cli-gnu build-image-cli-musl
		
build-image-cli-gnu:
	docker buildx build ${EXTRA_ARGS} \
		-f scripts/debian.Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target cli-image \
		--tag jdrouet/pcloud-cli:latest \
		--tag jdrouet/pcloud-cli:gnu \
		--tag jdrouet/pcloud-cli:${CLI_VERSION} \
		--tag jdrouet/pcloud-cli:${CLI_VERSION}-gnu \
		.

build-image-cli-musl:
	docker buildx build ${EXTRA_ARGS} \
		-f scripts/alpine.Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target cli-image \
		--tag jdrouet/pcloud-cli:musl \
		--tag jdrouet/pcloud-cli:${CLI_VERSION}-musl \
		.

build-image-fuse: build-image-fuse-gnu build-image-fuse-musl
		
build-image-fuse-gnu:
	docker buildx build ${EXTRA_ARGS} \
		-f scripts/debian.Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target fuse-image \
		--tag jdrouet/pcloud-fuse:latest \
		--tag jdrouet/pcloud-fuse:gnu \
		--tag jdrouet/pcloud-fuse:${FUSE_VERSION} \
		--tag jdrouet/pcloud-fuse:${FUSE_VERSION}-gnu \
		.

build-image-fuse-musl:
	docker buildx build ${EXTRA_ARGS} \
		-f scripts/alpine.Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target fuse-image \
		--tag jdrouet/pcloud-fuse:musl \
		--tag jdrouet/pcloud-fuse:${FUSE_VERSION}-musl \
		.

build-image-http-server: build-image-http-server-gnu build-image-http-server-musl
		
build-image-http-server-gnu:
	docker buildx build ${EXTRA_ARGS} \
		-f scripts/debian.Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target http-server-image \
		--tag jdrouet/pcloud-http-server:latest \
		--tag jdrouet/pcloud-http-server:gnu \
		--tag jdrouet/pcloud-http-server:${HTTP_SERVER_VERSION} \
		--tag jdrouet/pcloud-http-server:${HTTP_SERVER_VERSION}-gnu \
		.

build-image-http-server-musl:
	docker buildx build ${EXTRA_ARGS} \
		-f scripts/alpine.Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target http-server-image \
		--tag jdrouet/pcloud-http-server:musl \
		--tag jdrouet/pcloud-http-server:${HTTP_SERVER_VERSION}-musl \
		.
