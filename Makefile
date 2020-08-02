VERSION := $(shell git describe --exact-match --tags 2> /dev/null || git rev-parse --short HEAD)

all:
	cargo build

release:
	cargo build --release

armv7:
	cross build --target=armv7-unknown-linux-musleabihf --release
	docker buildx build --platform linux/arm/v7 -t hue-notify:$(VERSION) .
