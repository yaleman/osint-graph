#!/bin/bash

docker run --rm -it \
  -p 9000:9000 \
  --name osint-graph \
  --mount "type=bind,src=$HOME/.cache/osint-graph.sqlite3,target=/data/osint-graph.sqlite3" \
  --mount "type=bind,src=${OSINT_GRAPH_TLS_CERT},target=/certs/fullchain.pem,readonly" \
  --mount "type=bind,src=${OSINT_GRAPH_TLS_KEY},target=/certs/privkey.pem,readonly" \
  --env-file .env \
  ghcr.io/yaleman/osint-graph:latest