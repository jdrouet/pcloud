VERSION?=0.1.0

define build
	cross build --release --target $(1) --bin $(2)
	mv target/$(1)/release/$(2) target/release/$(3)
endef

build: build-amd64 build-arm32v7 build-arm64

build-amd64:
	docker build -t jdrouet/pcloud-builder:amd64 -f scripts/cross-amd64.Dockerfile .
	rm -rf target/x86_64-unknown-linux-gnu target/x86_64-unknown-linux-musl
	$(call build,x86_64-unknown-linux-gnu,pcloud-cli,pcloud-cli_${VERSION}_amd64-gnu)
	$(call build,x86_64-unknown-linux-musl,pcloud-cli,pcloud-cli_${VERSION}_amd64-musl)
	$(call build,x86_64-unknown-linux-gnu,pcloud-http-server,pcloud-http-server_${VERSION}_amd64-gnu)
	$(call build,x86_64-unknown-linux-musl,pcloud-http-server,pcloud-http-server_${VERSION}_amd64-musl)

build-arm64:
	rm -rf target/aarch64-unknown-linux-gnu target/aarch64-unknown-linux-musl
	$(call build,aarch64-unknown-linux-gnu,pcloud-cli,pcloud-cli_${VERSION}_arm64-gnu)
	$(call build,aarch64-unknown-linux-musl,pcloud-cli,pcloud-cli_${VERSION}_arm64-musl)
	$(call build,aarch64-unknown-linux-gnu,pcloud-http-server,pcloud-http-server_${VERSION}_arm64-gnu)
	$(call build,aarch64-unknown-linux-musl,pcloud-http-server,pcloud-http-server_${VERSION}_arm64-musl)

build-arm32v7:
	rm -rf target/armv7-unknown-linux-gnueabihf target/armv7-unknown-linux-musleabihf
	$(call build,armv7-unknown-linux-gnueabihf,pcloud-cli,pcloud-cli_${VERSION}_arm32v7-gnu)
	$(call build,armv7-unknown-linux-musleabihf,pcloud-cli,pcloud-cli_${VERSION}_arm32v7-musl)
	$(call build,armv7-unknown-linux-gnueabihf,pcloud-http-server,pcloud-http-server_${VERSION}_arm32v7-gnu)
	$(call build,armv7-unknown-linux-musleabihf,pcloud-http-server,pcloud-http-server_${VERSION}_arm32v7-musl)
