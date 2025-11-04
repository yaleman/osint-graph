# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OSINT Graph is a full-stack web application for Open Source Intelligence data visualization. It features a Rust backend with Axum web server and React TypeScript frontend with ReactFlow for interactive graph visualization.

The project is an OSINT discovery and mapping tool with a web frontend that allows users to create projects and add nodes with individual OSINT elements such as "person", "image file", "domain name", "IP address", "phone number", "URL", etc., along with metadata for those nodes.

## Architecture

- **Backend**: Rust workspace with Axum web server, SQLite database with SeaORM
- **Frontend**: React 18 + TypeScript + Vite, uses ReactFlow for graph rendering
- **Shared**: Common Rust types between backend and frontend via `osint-graph-shared` crate
- **Build Output**: Frontend builds to `dist/` directory, served by backend
- **Database**: SeaORM with migration system for schema management

Key directories:

- `osint-graph-backend/` - Rust server with API endpoints
  - `src/entity/` - SeaORM entity definitions
  - `src/migration/` - Database migration files
  - `src/db/` - Database operation implementations
- `osint-graph-frontend/` - React app with graph visualization
- `osint-graph-shared/` - Shared data types (Node, NodeLink)

### Database Layer

The backend uses **SeaORM** as the ORM layer with SQLite:

- **Migrations**: Schema changes managed via SeaORM migrations in `src/migration/`
- **Entities**: Type-safe database models in `src/entity/` (project, node, nodelink, attachment)
- **Operations**: Database operations use `ConnectionTrait` for query execution
- **Foreign Keys**: Automatic cascade delete/update for referential integrity
- **Connection**: `DatabaseConnection` type replaces direct sqlx usage

## Node System

### OSINT Node Types (10 types supported)

- **Person** - Individual people with names and contact info
- **Domain** - Internet domain names
- **IP Address** - Network addresses
- **Phone** - Telephone numbers
- **Email** - Email addresses  
- **URL** - Web links and resources
- **Image** - Image files and photos
- **Location** - Physical addresses and places
- **Organization** - Companies and groups
- **Document** - Files and documents

### Node Structure

- **UUID**: Auto-generated unique identifier for each node
- **Display**: Human-readable name (e.g., "John Doe" for person, "192.168.1.1" for IP)
- **Node Type**: One of the 10 OSINT types above
- **Value**: Raw data content
- **Timestamps**: Automatic tracking of creation and updates
- **Position**: X/Y coordinates for graph layout
- **Metadata**: Optional notes and additional information

### Backend Synchronization

- All node operations (create, update, move) automatically sync to backend
- Timestamp tracking for every change
- Real-time updates with proper conflict resolution via NodeUpdateList

## Development Commands

### Just Tasks

```bash
# Build frontend and run the application
just run

# Run all quality checks (clippy, tests, fmt, frontend linting)
just check

# Individual tasks
just clippy          # Rust linting
just test           # Run tests
just fmt            # Format code
just frontend-lint  # Frontend ESLint
```

### Make Tasks

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
```

### Testing & Coverage

```bash
# Run tests
cargo test

# Coverage analysis with tarpaulin
cargo tarpaulin --out Html --output-dir target/coverage

# Coverage for specific package
cargo tarpaulin --packages osint-graph-shared
```

## Testing

- **Backend**: Uses `cargo test` with axum-test for HTTP testing
- **Coverage**: `cargo tarpaulin` generates HTML reports (currently 86.45% coverage)
- **Frontend**: ESLint for linting, TypeScript for type checking
- **Comprehensive test suite**: 16+ unit tests for NodeUpdateList synchronization logic

## Key Files

- **API Routes**: `osint-graph-backend/src/main.rs` - RESTful endpoints under `/api/v1/`
- **Frontend Entry**: `osint-graph-frontend/src/App.tsx` - Main React component with project management
- **Shared Types**: `osint-graph-shared/src/node.rs` - Node structure and NodeUpdateList
- **Database Layer**:
  - `osint-graph-backend/src/storage.rs` - Database initialization and migrations
  - `osint-graph-backend/src/entity/` - SeaORM entity definitions
  - `osint-graph-backend/src/migration/` - Migration files for schema versioning
  - `osint-graph-backend/src/db/` - Database operations (node, project, nodelink, attachment)
- **API Integration**: `osint-graph-frontend/src/api.tsx` - Backend communication with validation
- **Node Types**: `osint-graph-frontend/src/types.tsx` - TypeScript definitions
- **Project Components**:
  - `osint-graph-frontend/src/components/ProjectSelector.tsx` - Project switching UI
  - `osint-graph-frontend/src/components/ProjectMismatchDialog.tsx` - Validation error handling

## API Structure

Backend serves:

- Static files from `/dist/` (built frontend)
- API endpoints:
  - `GET/POST /api/v1/projects` - Project management
  - `GET/POST /api/v1/project/{id}` - Individual project operations
  - `GET/POST /api/v1/node/{id}` - Node CRUD operations
- Uses `Arc<RwLock<AppState>>` for thread-safe shared state
- AppState contains `DatabaseConnection` for SeaORM access

## User Interface Features

### Interactive Graph

- **Background Click**: Click anywhere on canvas to open node creation menu
- **Node Creation**: Select from 10 OSINT node types with color coding
- **Auto-Edit**: New nodes automatically open edit dialog for immediate naming
- **Node Editing**: Double-click any node to edit its display name
- **Drag & Drop**: Move nodes around, positions auto-save to backend
- **Connections**: Drag between nodes to create relationships

### Real-time Sync

- All changes immediately persist to backend with timestamps
- Position updates on drag
- Display name changes on edit
- Automatic conflict resolution

### Project Management

- **Project Validation**: On startup, frontend validates that localStorage project ID exists in backend
- **Mismatch Handling**: If project doesn't exist, user is prompted with options:
  - Create new project
  - Select existing project from list
- **Project Selector**: Dropdown UI in top-left to:
  - View current project name
  - Switch between projects
  - Create new projects
- **Toast Notifications**: User-friendly error and success messages via react-hot-toast
- **Data Integrity**: SQLite foreign key constraints prevent orphaned nodes
- **Backend Validation**: Node operations validate project exists before saving

## Development Notes

- Frontend builds to `../dist/` relative to frontend directory
- Hot module replacement via Vite on custom port 8189
- Database managed through SeaORM migrations
- All database operations use `ConnectionTrait` for execution
- All shared types use Serde for JSON serialization
- ReactFlow handles graph visualization
- UUID generation via `uuid` crate
- Color-coded nodes for visual type identification
- Database migrations run automatically on startup

## Code Quality Requirements

- You must ensure that `just check` runs without errors or warnings before considering any task complete
- Maintain test coverage above 85% (currently at 86.45%)
- All new Node functionality must include comprehensive tests
- Frontend linting must pass with zero warnings

## Workflow Guidance

### CRITICAL REQUIREMENTS

- **Git commits are MANDATORY when tasks are completed** - Every completed task MUST be committed
- **You MUST git commit when a user's task is complete** - This is non-negotiable
- **Update this CLAUDE.md file any time system features/design are changed**
- Use TodoWrite tool for complex multi-step tasks to track progress

### Commit Guidelines

- Write clear, descriptive commit messages
- Include what was implemented, not just "update code"
- Mention key features, improvements, or fixes
- Do not mention that Claude generated the code
- Always commit when a user's task is marked as complete

## Development Best Practices

- When making changes, always validate that a change to the front end is reflected in the backend too, and vice versa