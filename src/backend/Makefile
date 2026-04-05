.PHONY: dev build fix

dev:
	cargo run

build:
	cargo build --release

fix:
	cargo fmt --all
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test
