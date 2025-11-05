default:
    just --list

# Run cargo clippy on all targets
clippy:
    cargo clippy --all-targets --all-features --workspace

# Run cargo tests
test:
    cargo test

# Run cargo fmt
fmt:
    cargo fmt

# Run the backend
backend:
	cargo run --bin osint-graph

# Build the frontend
frontend:
    cd osint-graph-frontend && vite build --emptyOutDir

# lint all the things
lint: clippy fmt frontend-lint

# Run frontend linting/checks
frontend-lint:
    biome ci osint-graph-frontend

# Build frontend and run the application
run:
    killall osint-graph-backend || true
    cd osint-graph-frontend && vite build --emptyOutDir
    cargo run

# Run all checks (clippy, test, fmt, frontend-lint)
check: clippy test fmt frontend-lint


set positional-arguments

@coverage_inner *args='':
    cargo tarpaulin --workspace --exclude-files=src/main.rs $@

# run coverage checks
coverage:
    just coverage_inner --out=Html
    @echo "Coverage report should be at file://$(pwd)/tarpaulin-report.html"

coveralls:
    just coverage_inner --out=Html --coveralls $COVERALLS_REPO_TOKEN
    @echo "Coverage report should be at https://coveralls.io/github/yaleman/osint-graph?branch=$(git branch --show-current)"

reload:
    cargo watch -s 'just run' --why