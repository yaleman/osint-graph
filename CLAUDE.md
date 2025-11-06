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
- **Organisation** - Companies and groups
- **Document** - Files and documents

### Node Structure

- **UUID**: Auto-generated unique identifier for each node
- **Display**: Human-readable name (e.g., "John Doe" for person, "192.168.1.1" for IP)
- **Node Type**: One of the 10 OSINT types above
- **Value**: Raw data content
- **Timestamps**: Automatic tracking of creation and updates
- **Position**: X/Y coordinates for graph layout
- **Metadata**: Optional notes and additional information
- **Attachments**: File attachments with gzip compression in database

### Backend Synchronization

- All node operations (create, update, move) automatically sync to backend
- Timestamp tracking for every change
- Real-time updates with proper conflict resolution via NodeUpdateList

## File Attachment System

### Overview

Each node can have multiple file attachments stored in the database with automatic gzip compression. The system supports upload, download, view (inline), delete, and list operations.

### Backend Implementation

- **Location**: `osint-graph-backend/src/attachment.rs`
- **Database Entity**: `osint-graph-backend/src/entity/attachment.rs`
- **Storage**: Files stored as gzip-compressed blobs in SQLite database
- **Foreign Key**: Attachments cascade delete when parent node is deleted
- **Size Limit**: 100MB per file upload

### API Endpoints

- `POST /api/v1/node/{id}/attachment` - Upload file (multipart/form-data)
- `GET /api/v1/node/{node_id}/attachment/{attachment_id}` - Download file
- `GET /api/v1/node/{node_id}/attachment/{attachment_id}/view` - View file inline
- `DELETE /api/v1/node/{node_id}/attachment/{attachment_id}` - Delete attachment
- `GET /api/v1/node/{id}/attachments` - List all attachments for node

### Attachment Model

```rust
pub struct Model {
    pub id: Uuid,
    pub node_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,           // Original uncompressed size
    pub data: Vec<u8>,       // Gzip compressed data
    pub created: DateTime<Utc>,
}
```

### Features

- **Compression**: All files automatically compressed with gzip before storage
- **Decompression**: Transparent decompression on download/view
- **Content-Type Preservation**: Original MIME types maintained
- **Inline Viewing**: Images, PDFs, and text files can be viewed in browser
- **Download**: All files can be downloaded with proper Content-Disposition headers
- **Validation**: Node existence validated before attachment creation
- **Protection**: Attachments only available for saved nodes (not pending nodes)

### Frontend Integration

- **Upload**: Drag-and-drop or file picker in node edit dialog
- **View Button** (👁): Opens viewable files (images, PDFs, text) in new tab
- **Download Button** (↓): Downloads file to user's device
- **Delete Button** (×): Removes attachment from database
- **File Type Detection**: Automatic detection of viewable file types by MIME type and extension

### Viewable File Types

The system automatically detects and allows inline viewing for:

- **Images**: jpg, jpeg, png, gif, bmp, webp, svg
- **Documents**: pdf
- **Text**: txt, md, json, xml, html, css
- **Code**: js, ts, tsx, jsx

### Error Handling

- Foreign key validation prevents attachments on non-existent nodes
- Toast notifications for upload/download/delete success and failures
- Proper HTTP status codes (404 for not found, 500 for server errors)
- Debug logging throughout attachment operations

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
just run

# Auto-reload during development
just reload

# Build frontend only
just frontend

# Run backend only
just backend

# Run linters
just lint
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

- **API Routes**: `osint-graph-backend/src/lib.rs` - RESTful endpoints under `/api/v1/`
- **Frontend Entry**: `osint-graph-frontend/src/App.tsx` - Main React component with project management
- **Shared Types**: `osint-graph-shared/src/node.rs` - Node structure and NodeUpdateList
- **Database Layer**:
  - `osint-graph-backend/src/storage.rs` - Database initialization and migrations
  - `osint-graph-backend/src/entity/` - SeaORM entity definitions
  - `osint-graph-backend/src/migration/` - Migration files for schema versioning
  - `osint-graph-backend/src/db/` - Database operations (node, project, nodelink, attachment)
- **Attachment System**: `osint-graph-backend/src/attachment.rs` - File upload/download with gzip compression
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
  - `GET/POST/PUT/DELETE /api/v1/project/{id}` - Individual project operations
  - `GET/POST/PUT/DELETE /api/v1/node/{id}` - Node CRUD operations
  - `POST /api/v1/node/{id}/attachment` - File upload
  - `GET /api/v1/node/{id}/attachments` - List attachments
  - `GET /api/v1/node/{node_id}/attachment/{attachment_id}` - Download file
  - `GET /api/v1/node/{node_id}/attachment/{attachment_id}/view` - View file inline
  - `DELETE /api/v1/node/{node_id}/attachment/{attachment_id}` - Delete file
  - `GET/POST/DELETE /api/v1/nodelink` - Node link operations
  - `GET /api/v1/project/{id}/export` - Export project data
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
- **File Attachments**: Upload, view, download, and delete files attached to nodes

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
- never ever use inline styles, use css classes for all styling
- database migrations are in @osint-graph-backend/src/migration/
- database entities eare in @osint-graph-backend/src/entity/
- use relative file paths when working on files in the repository
