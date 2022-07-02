EXTRA_ARGS?=
RUST_VERSION?=1.62.0
VERSION?=local

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

build-cli: build-cli-bin build-cli-deb build-cli-image

build-cli-bin:
	docker buildx build \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target cli-bin \
		--output type=local,dest=$(shell pwd)/target \
		.

build-cli-deb:
	docker buildx build \
		--platform linux/amd64,linux/arm64,linux/arm/v7 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target cli-deb \
		--output type=local,dest=$(shell pwd)/target \
		.

build-cli-image:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target cli-image \
		--tag jdrouet/pcloud-cli:${VERSION} \
		.
		
build-http: build-http-image

build-http-image:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/amd64,linux/arm64 \
		--build-arg RUST_VERSION=${RUST_VERSION} \
		--target http-image \
		--tag jdrouet/pcloud-cli:${VERSION} \
		.
