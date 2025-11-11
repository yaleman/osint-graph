set positional-arguments

platform := shell("uname -m")

[private]
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



# run coverage checks
coverage:
    just coverage_inner --out=Html
    @echo "Coverage report should be at file://$(pwd)/tarpaulin-report.html"

# run coverage and submit to coveralls.io
coveralls:
    just coverage_inner --out=Html --coveralls $COVERALLS_REPO_TOKEN
    @echo "Coverage report should be at https://coveralls.io/github/yaleman/osint-graph?branch=$(git branch --show-current)"

# only used for coverage commands
[private]
@coverage_inner *args='':
    cargo tarpaulin  --workspace --exclude-files=src/main.rs $@

# Run in reload mode
reload:
    cargo watch -s 'just run' --why

# Check spelling
codespell:
    uvx codespell -c

# Build the local container
docker_build:
    docker buildx build --platform linux/{{platform}} -t ghcr.io/yaleman/osint-graph:latest --load .

# Runs the OpenAPI spec check script
openapi_spec:
    ./check_openapi_spec.sh