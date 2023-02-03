fmt:
	cargo fmt
build:
	cargo build
release:
	cargo build --release
example:
	cargo run --example $(example) --release
