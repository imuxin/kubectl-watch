fmt:
	cargo fmt
debug:
	cargo build
release:
	cargo build --release
example:
	cargo run --example $(example) --release
