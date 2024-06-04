DEFAULT: reload

.PHONY: lint ## run linters
lint: codespell
	cargo clippy
	biome lint osint-graph-frontend

.PHONY: frontend
frontend:
	cd osint-graph-frontend && vite build --emptyOutDir

.PHONY: backend
backend:
	cargo run --bin osint-graph-backend

.PHONY: serve
serve: frontend backend


.PHONY: reload
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
coverage: coverage/test coverage/grcov
	echo "Coverage report is in ./target/coverage/html/index.html"