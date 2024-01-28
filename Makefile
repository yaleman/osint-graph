.PHONY: build
build:
	cd osint-graph && trunk build --dist ../dist/

.PHONY: serve
serve: build
	cd osint-graph && trunk serve

