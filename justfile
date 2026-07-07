# list recipes
help:
    just --list

# format the code
fmt:
    cargo fmt --all

# alias for fmt
format: fmt

# check formatting without writing changes
fmt-check:
    cargo fmt --all -- --check

# lint the code
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# run the dice-roll example
dev:
    cargo run --example dice

# build the crate
build:
    cargo build --release

# run tests
test:
    cargo test

# Lint and then test targets (like CI does)
check: fmt-check lint test

# Run checks and build
ci: check build
