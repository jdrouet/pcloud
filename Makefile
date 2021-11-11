EXTRA_ARGS?=
EXTRA_TAGS?=
VERSION?=local

publish:
	cd lib && cargo publish ${EXTRA_ARGS}
	cd cli && cargo publish ${EXTRA_ARGS}
	cd fuse && cargo publish ${EXTRA_ARGS}
	cd http-server && cargo publish ${EXTRA_ARGS}

clean:
	rm -rf target/release target/i386 target/amd64 target/arm32v7 target/arm64v8

build: prebuild build-i386 build-amd64 build-arm32v7 build-arm64v8

prebuild: clean
	mkdir -p target/release

build-i386: build-i386-gnu

build-i386-gnu:
	docker build \
		--platform linux/i386 \
		--file=scripts/debian.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/i386/ \
		.
	mv target/i386/pcloud-cli target/release/pcloud-cli_${VERSION}_i386-gnu
	mv target/i386/pcloud-fuse target/release/pcloud-fuse_${VERSION}_i386-gnu
	mv target/i386/pcloud-http-server target/release/pcloud-http-server_${VERSION}_i386-gnu

build-amd64: build-amd64-gnu build-amd64-musl

build-amd64-gnu:
	docker build \
		--platform linux/amd64 \
		--file=scripts/debian.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/amd64/ \
		.
	mv target/amd64/pcloud-cli target/release/pcloud-cli_${VERSION}_amd64-gnu
	mv target/amd64/pcloud-fuse target/release/pcloud-fuse_${VERSION}_amd64-gnu
	mv target/amd64/pcloud-http-server target/release/pcloud-http-server_${VERSION}_amd64-gnu

build-amd64-musl:
	docker build \
		--platform linux/amd64 \
		--file=scripts/alpine.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/amd64/ \
		.
	mv target/amd64/pcloud-cli target/release/pcloud-cli_${VERSION}_amd64-musl
	mv target/amd64/pcloud-fuse target/release/pcloud-fuse_${VERSION}_amd64-musl
	mv target/amd64/pcloud-http-server target/release/pcloud-http-server_${VERSION}_amd64-musl

build-arm32v7: build-arm32v7-gnu

build-arm32v7-gnu:
	docker build \
		--platform linux/arm/v7 \
		--file=scripts/debian.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/arm32v7/ \
		.
	mv target/arm32v7/pcloud-cli target/release/pcloud-cli_${VERSION}_arm32v7-gnu
	mv target/arm32v7/pcloud-fuse target/release/pcloud-fuse_${VERSION}_arm32v7-gnu
	mv target/arm32v7/pcloud-http-server target/release/pcloud-http-server_${VERSION}_arm32v7-gnu

build-arm64v8: build-arm64v8-gnu build-arm64v8-musl

build-arm64v8-gnu:
	docker build \
		--platform linux/arm64/v8 \
		--file=scripts/debian.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/arm64v8/ \
		.
	mv target/arm64v8/pcloud-cli target/release/pcloud-cli_${VERSION}_arm64v8-gnu
	mv target/arm64v8/pcloud-fuse target/release/pcloud-fuse_${VERSION}_arm64v8-gnu
	mv target/arm64v8/pcloud-http-server target/release/pcloud-http-server_${VERSION}_arm64v8-gnu

build-arm64v8-musl:
	docker build \
		--platform linux/arm64/v8 \
		--file=scripts/alpine.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/arm64v8/ \
		.
	mv target/arm64v8/pcloud-cli target/release/pcloud-cli_${VERSION}_arm64v8-musl
	mv target/arm64v8/pcloud-fuse target/release/pcloud-fuse_${VERSION}_arm64v8-musl
	mv target/arm64v8/pcloud-http-server target/release/pcloud-http-server_${VERSION}_arm64v8-musl

build-images: build-images-cli build-images-fuse build-images-http-server

build-images-cli: build-images-cli-alpine build-images-cli-debian

build-images-cli-alpine:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/amd64,linux/arm64/v8 \
		--tag jdrouet/pcloud-cli:${VERSION}-alpine \
		--target cli \
		--file scripts/alpine.Dockerfile \
		.

build-images-cli-debian:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/i386,linux/amd64,linux/arm/v7,linux/arm64/v8 \
		--tag jdrouet/pcloud-cli:${VERSION} \
		--tag jdrouet/pcloud-cli:${VERSION}-debian \
		--target cli \
		--file scripts/debian.Dockerfile \
		.

build-images-fuse: build-images-fuse-alpine build-images-fuse-debian

build-images-fuse-alpine:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/amd64,linux/arm64/v8 \
		--tag jdrouet/pcloud-fuse:${VERSION}-alpine \
		--target fuse \
		--file scripts/alpine.Dockerfile \
		.

build-images-fuse-debian:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/i386,linux/amd64,linux/arm/v7,linux/arm64/v8 \
		--tag jdrouet/pcloud-fuse:${VERSION} \
		--tag jdrouet/pcloud-fuse:${VERSION}-debian \
		--target fuse \
		--file scripts/debian.Dockerfile \
		.

build-images-http-server: build-images-http-server-alpine build-images-http-server-debian

build-images-http-server-alpine:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/amd64,linux/arm64/v8 \
		--tag jdrouet/pcloud-http-server:${VERSION}-alpine \
		--target http-server \
		--file scripts/alpine.Dockerfile \
		.

build-images-http-server-debian:
	docker buildx build ${EXTRA_ARGS} \
		--platform linux/i386,linux/amd64,linux/arm/v7,linux/arm64/v8 \
		--tag jdrouet/pcloud-http-server:${VERSION} \
		--tag jdrouet/pcloud-http-server:${VERSION}-debian \
		--target http-server \
		--file scripts/debian.Dockerfile \
		.
