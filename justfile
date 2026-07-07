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
    cargo test --all-features

# check documentation with rustdoc warnings denied
doc-check:
    RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

# verify package contents without publishing
package:
    cargo package

# format, lint, test, document, and package like CI
check: fmt-check lint test doc-check package

# Run checks and build
ci: check build
