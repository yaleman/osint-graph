.PHONY: build
build:
	cd osint-graph &&  trunk build --dist ../dist/ --filehash false

.PHONY: backend
backend:
	cargo run --bin osint-graph-backend


.PHONY: serve
serve: build backend
