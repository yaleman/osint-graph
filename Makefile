DEFAULT: reload

.PHONY: build
build:
	cd osint-graph &&  trunk build

.PHONY: backend
backend:
	cargo run --bin osint-graph-backend

.PHONY: serve
serve: build backend


.PHONY: reload
reload:
	cargo watch -s 'make serve'

.PHONY: codespell
codespell: ## Spell-check the code
codespell:
	codespell -c \
		--ignore-words .codespell_ignore \
		--skip='./target'