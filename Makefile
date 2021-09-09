build: build-i386 build-amd64 build-arm32v7 build-arm64v8

build-i386:
	docker build \
		--platform linux/i386 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/i386/ \
		.

build-amd64:
	docker build \
		--platform linux/amd64 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/amd64/ \
		.

build-arm32v7:
	docker build \
		--platform linux/arm/v7 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/arm32v7/ \
		.

build-arm64v8:
	docker build \
		--platform linux/arm64/v8 \
		--file=scripts/builder.Dockerfile \
		--tag=pcloud:builder \
		--target=artifact \
		--output type=local,dest=$(shell pwd)/target/arm64v8/ \
		.
