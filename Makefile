.PHONY: all
all: fmt build test lint

.PHONY: fmt
fmt:
	@cargo fmt --all

.PHONY: build
build:
	@cargo build

.PHONY: test
test:
	@cargo test

.PHONY: itest
itest:
	@RUST_LOG=off cargo test -- --ignored

.PHONY: lint
lint:
	@cargo clippy

.PHONY: clean
clean:
	@cargo clean