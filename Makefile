.PHONY: all build test clean
all: build
build:
	cargo build --release
test:
	cargo test --locked
clean:
	cargo clean
