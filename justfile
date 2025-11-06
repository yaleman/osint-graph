default:
    just --list

# Run cargo clippy on all targets
clippy:
    cargo clippy  --quiet --all-targets --all-features --workspace

# Run cargo tests
test:
    cargo test --quiet

# Run cargo fmt
fmt:
    cargo fmt --quiet

# Run the backend
backend:
	cargo run  --quiet --bin osint-graph

# Build the frontend
frontend:
    cd osint-graph-frontend && vite build --emptyOutDir

# lint all the things
lint: clippy fmt frontend-fmt frontend-lint

# fmt the frontend code
frontend-fmt:
    biome format osint-graph-frontend --fix

# Run frontend linting/checks
frontend-lint:
    biome ci osint-graph-frontend

# Build frontend and run the application
run:
    killall osint-graph-backend || true
    cd osint-graph-frontend && vite build --emptyOutDir
    cargo run -- --debug

# Run all checks (clippy, test, fmt, frontend-lint)
check: clippy test fmt frontend-lint


set positional-arguments

@coverage_inner *args='':
    cargo tarpaulin --quiet --workspace --exclude-files=src/main.rs $@

# run coverage checks
coverage:
    just coverage_inner --out=Html
    @echo "Coverage report should be at file://$(pwd)/tarpaulin-report.html"

coveralls:
    just coverage_inner --out=Html --coveralls $COVERALLS_REPO_TOKEN
    @echo "Coverage report should be at https://coveralls.io/github/yaleman/osint-graph?branch=$(git branch --show-current)"

reload:
    cargo watch -s 'just run' --why