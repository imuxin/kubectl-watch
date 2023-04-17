fmt:
	cargo fmt
build:
	cargo build
build-release:
	cargo build --release
example:
	cargo run --example $(example) --release
fmt-check:
	cargo fmt --all -- --check
