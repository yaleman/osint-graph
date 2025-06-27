# Run cargo clippy on all targets
clippy:
    cargo clippy --all-targets

# Run cargo tests
test:
    cargo test

# Run cargo fmt
fmt:
    cargo fmt

# Run all checks (clippy, test, fmt)
check: clippy test fmt