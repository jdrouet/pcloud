VERSION?=local

clean:
	rm -rf target/release target/i386 target/amd64 target/arm32v7 target/arm64v8

build: prebuild build-i386 build-amd64 build-arm32v7 build-arm64v8

prebuild: clean
	mkdir -p target/release

build-i386:
	docker build \
		--platform linux/i386 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/i386/ \
		.
	mv target/i386/pcloud-cli target/release/pcloud-cli_${VERSION}_i386-gnu
	mv target/i386/pcloud-fuse target/release/pcloud-fuse_${VERSION}_i386-gnu
	mv target/i386/pcloud-http-server target/release/pcloud-http-server_${VERSION}_i386-gnu

build-amd64:
	docker build \
		--platform linux/amd64 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/amd64/ \
		.
	mv target/amd64/pcloud-cli target/release/pcloud-cli_${VERSION}_amd64-gnu
	mv target/amd64/pcloud-fuse target/release/pcloud-fuse_${VERSION}_amd64-gnu
	mv target/amd64/pcloud-http-server target/release/pcloud-http-server_${VERSION}_amd64-gnu

build-arm32v7:
	docker build \
		--platform linux/arm/v7 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/arm32v7/ \
		.
	mv target/arm32v7/pcloud-cli target/release/pcloud-cli_${VERSION}_arm32v7-gnu
	mv target/arm32v7/pcloud-fuse target/release/pcloud-fuse_${VERSION}_arm32v7-gnu
	mv target/arm32v7/pcloud-http-server target/release/pcloud-http-server_${VERSION}_arm32v7-gnu

build-arm64v8:
	docker build \
		--platform linux/arm64/v8 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/arm64v8/ \
		.
	mv target/arm64v8/pcloud-cli target/release/pcloud-cli_${VERSION}_arm64v8-gnu
	mv target/arm64v8/pcloud-fuse target/release/pcloud-fuse_${VERSION}_arm64v8-gnu
	mv target/arm64v8/pcloud-http-server target/release/pcloud-http-server_${VERSION}_arm64v8-gnu
