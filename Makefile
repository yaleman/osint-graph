
.DEFAULT: help
.PHONY: help
help:
	@grep -E -h '\s##\s' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: lint
lint: ## run linters
lint: codespell
	cargo clippy
	biome lint osint-graph-frontend

.PHONY: frontend
frontend: ## Build the frontend
frontend:
	cd osint-graph-frontend && vite build --emptyOutDir

.PHONY: backend
backend: ## Run the backend
backend:
	cargo run --bin osint-graph-backend

.PHONY: serve
serve: ## Run the frontend and backend in serve mode
serve: frontend backend

.PHONY: reload
reload: ## Rebuild and reload on changes
reload:
	cargo watch -s 'make serve' --why

.PHONY: codespell
codespell: ## Spell-check the code
codespell:
	codespell -c \
	--ignore-words .codespell_ignore \
	--skip='./target' --skip './osint-graph-frontend/node_modules' --skip './dist'


.PHONY: rust/coverage
coverage/test: ## Run coverage tests
coverage/test:
	rm -rf "$(PWD)/target/profile"
	LLVM_PROFILE_FILE="$(PWD)/target/profile/coverage-%p-%m.profraw" RUSTFLAGS="-C instrument-coverage" cargo test $(TESTS)

.PHONY: coverage/grcov
coverage/grcov: ## Run grcov
coverage/grcov:
	rm -rf ./target/coverage/html
	grcov . --binary-path ./target/debug/deps/ \
		-s . \
		-t html \
		--branch \
		--ignore-not-existing \
		--ignore '../*' \
		--ignore "/*" \
		--ignore "target/*" \
		-o target/coverage/html

.PHONY: coverage
coverage: ## Run all the coverage tests
coverage:
	cargo llvm-cov clean --workspace && \
	cargo llvm-cov --html  --ignore-filename-regex main