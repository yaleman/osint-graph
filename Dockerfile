# Frontend build stage
FROM node:22-slim AS frontend-builder

WORKDIR /build
COPY osint-graph-frontend/package.json osint-graph-frontend/pnpm-lock.yaml ./

# Install pnpm using corepack (built into Node.js)
RUN corepack enable && corepack prepare pnpm@latest --activate
RUN pnpm install --frozen-lockfile

COPY osint-graph-frontend/ ./
RUN pnpm run build

# Backend build stage
FROM rust:1.90.0-slim-trixie AS builder

ARG GITHUB_SHA="$(git rev-parse HEAD)"
LABEL com.osint-graph.git-commit="${GITHUB_SHA}"

# fixing the issue with getting OOMKilled in BuildKit
RUN mkdir /osint-graph
COPY . /osint-graph/

# Copy the built frontend from the frontend-builder stage
COPY --from=frontend-builder /build/../dist /osint-graph/dist

WORKDIR /osint-graph
# install the dependencies
RUN apt-get update && apt-get -q install -y \
    git \
    clang \
    pkg-config \
    mold
ENV CC="/usr/bin/clang"
RUN cargo build --quiet --release --bin osint-graph
RUN chmod +x /osint-graph/target/release/osint-graph

# FROM gcr.io/distroless/cc-debian12 AS osint-graph
FROM rust:1.90.0-slim-trixie AS secondary

RUN apt-get -y remove --allow-remove-essential \
    binutils cpp cpp-14 gcc gcc grep gzip ncurses-bin ncurses-base sed && apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/* && rm -rf /usr/local/cargo /usr/local/rustup

# # ======================
# https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile
COPY --from=builder /osint-graph/target/release/osint-graph /
COPY --from=builder /osint-graph/dist /dist
WORKDIR /
RUN useradd -m nonroot

FROM scratch AS final
ARG DESCRIPTION="OSINT Shenanigans"
ARG GITHUB_SHA="unknown"
LABEL DESCRIPTION="${DESCRIPTION}"
LABEL com.osint-graph.git-commit="${GITHUB_SHA}"

COPY --from=secondary / /

ENV OSINT_GRAPH_DB_PATH="/data/osint-graph.sqlite3"

USER nonroot
ENTRYPOINT ["./osint-graph"]


