# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OSINT Graph is a full-stack web application for Open Source Intelligence data visualization. It features a Rust backend with Axum web server and React TypeScript frontend with ReactFlow for interactive graph visualization.

- The project is an OSINT discovery and mapping tool with a web front end which allows a user to create a project, then add nodes to it with individual elements such as "person", "image file", "domain name", "IP address", "phone number", "URL" etc, and then include metadata to those nodes.

## Architecture

- **Backend**: Rust workspace with Axum web server, SQLite + REDB storage
- **Frontend**: React 18 + TypeScript + Vite, uses ReactFlow for graph rendering
- **Shared**: Common Rust types between backend and frontend via `osint-graph-shared` crate
- **Build Output**: Frontend builds to `dist/` directory, served by backend

Key directories:
- `osint-graph-backend/` - Rust server with API endpoints
- `osint-graph-frontend/` - React app with graph visualization
- `osint-graph-shared/` - Shared data types (Project, Node, NodeLink)

## Development Commands

All commands use the Makefile:

```bash
# Start development server (builds frontend + runs backend)
make serve

# Auto-reload during development
make reload

# Build frontend only
make frontend

# Run backend only
make backend

# Run linters
make lint

# Run tests
cargo test

# Generate coverage report
make coverage
```

## Testing

- **Backend**: Uses `cargo test` with axum-test for HTTP testing
- **Coverage**: `cargo llvm-cov` generates HTML reports in `target/coverage/html/`
- **Frontend**: ESLint for linting, TypeScript for type checking

## Key Files

- **API Routes**: `osint-graph-backend/src/main.rs` - RESTful endpoints under `/api/v1/`
- **Frontend Entry**: `osint-graph-frontend/src/App.tsx` - Main React component
- **Shared Types**: `osint-graph-shared/src/` - Project, Node, NodeLink models
- **Database**: `osint-graph-backend/src/db/` - SQLite operations
- **Frontend Config**: `osint-graph-frontend/vite.config.ts` - Custom HMR on port 8189

## API Structure

Backend serves:
- Static files from `/dist/` (built frontend)
- API endpoints at `/api/v1/projects`, `/api/v1/project/{id}`, `/api/v1/node/{id}`
- Uses Arc<RwLock<AppState>> for thread-safe shared state

## Development Notes

- Frontend builds to `../dist/` relative to frontend directory
- Hot module replacement via Vite on custom port 8189
- Database migrations handled by SQLx
- All shared types use Serde for JSON serialization
- ReactFlow handles graph visualization with D3-Force for physics simulation

## Code Quality Reminders

- You must ensure that 'just check' runs without errors or warnings, and you need to fix them before considering your task complete

## Workflow Guidance

- Git commit with an appropriate message when you are done with a task, and don't mention that claude generated it
- You must git commit when a user's task is complete