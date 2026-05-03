set dotenv-load := true

check:
    cargo check --all-features

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-features

nextest:
    cargo nextest run --all-features

doc:
    cargo doc --all-features --no-deps

ci: fmt-check clippy test doc
