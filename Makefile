all:
	cargo build

release:
	cargo build --release

armv7:
	cross build --target=armv7-unknown-linux-gnueabihf --release
