# Run cargo clippy on all targets
clippy:
    cargo clippy --all-targets

# Run cargo tests
test:
    cargo test

# Run cargo fmt
fmt:
    cargo fmt

# Run frontend linting/checks
frontend-lint:
    cd osint-graph-frontend && npm run lint

# Run all checks (clippy, test, fmt, frontend-lint)
check: clippy test fmt frontend-lint