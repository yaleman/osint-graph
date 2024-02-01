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
