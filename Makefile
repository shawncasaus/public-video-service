.PHONY: run fmt lint test
run:
	cargo run -p api-gateway
fmt:
	cargo fmt --all
lint:
	cargo clippy --all-targets -- -D warnings
test:
	cargo test --all

