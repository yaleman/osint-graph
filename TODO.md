# Project Management UI Implementation TODO

## Phase 1: Backend Schema & Database

### 1.2 File Attachment Schema
- [ ] Create `Attachment` struct in new file `osint-graph-shared/src/attachment.rs`:
  - id: Uuid
  - node_id: Uuid
  - filename: String
  - content_type: String
  - size: i64
  - data: Vec<u8> (will be zstd compressed)
  - created: DateTime<Utc>
- [ ] Add `attachments: Vec<Uuid>` field to Node struct (just IDs, not full data)
- [ ] Create attachments table in `osint-graph-backend/src/db/attachment.rs`:
  - Table schema with FOREIGN KEY to node(id)
  - Implement DBEntity trait
  - Add zstd compression on save
  - Add zstd decompression on load
- [ ] Update node database schema to add attachments column (TEXT NULL, JSON array of UUIDs)
- [ ] Write tests for Attachment CRUD with compression/decompression
- [ ] Write tests for cascade deletion (deleting node deletes attachments)

### 1.3 Backend API Endpoints - Export/Import
- [ ] Add `GET /api/v1/project/:id/export` endpoint
- [ ] Implement `export_project` handler that returns JSON with structure:
  ```json
  {
    "project": { Project },
    "nodes": [ OSINTNode array ],
    "links": [ NodeLink array ],
    "attachments": { node_id: [ { id, filename, content_type, data_base64 } ] }
  }
  ```
- [ ] Write test for export endpoint with full project data
- [ ] Add `POST /api/v1/project/import` endpoint with query param `mode` (new/overwrite/merge)
- [ ] Implement `import_project` handler:
  - mode=new: Generate new UUIDs for everything, preserve relationships
  - mode=overwrite: Delete existing project and replace with import data
  - mode=merge: Add nodes/links to existing project with new UUIDs
- [ ] Write test for import mode=new
- [ ] Write test for import mode=overwrite
- [ ] Write test for import mode=merge

### 1.4 Backend API Endpoints - File Attachments
- [ ] Add `POST /api/v1/node/:id/attachment` endpoint (multipart/form-data)
- [ ] Implement `upload_attachment` handler with zstd compression
- [ ] Write test for file upload with various file types
- [ ] Write test for file upload size limits
- [ ] Add `GET /api/v1/node/:id/attachment/:attachment_id` endpoint
- [ ] Implement `download_attachment` handler with decompression
- [ ] Write test for file download
- [ ] Add `DELETE /api/v1/node/:id/attachment/:attachment_id` endpoint
- [ ] Implement `delete_attachment` handler
- [ ] Write test for attachment deletion
- [ ] Add `GET /api/v1/node/:id/attachments` endpoint (list all attachments for a node)
- [ ] Implement `list_attachments` handler
- [ ] Write test for listing attachments

## Phase 2: Frontend Types & API

### 2.1 Update Frontend Types
- [ ] Update `Project` interface in `osint-graph-frontend/src/types.tsx`:
  - Add `description?: string`
  - Add `tags?: string[]`
  - Add `last_updated?: Date`
- [ ] Create `Attachment` interface in `types.tsx`:
  - id: string
  - filename: string
  - content_type: string
  - size: number
  - created: string
- [ ] Update `OSINTNode` interface to add `attachments?: string[]` (array of attachment IDs)
- [ ] Create `ProjectExport` interface:
  ```typescript
  {
    project: Project,
    nodes: OSINTNode[],
    links: NodeLink[],
    attachments: Record<string, Attachment[]>
  }
  ```
- [ ] Create `ImportMode` type: 'new' | 'overwrite' | 'merge'

### 2.2 Frontend API Client
- [ ] Add `updateProject(id: string, data: Partial<Project>): Promise<void>` in `api.tsx`
- [ ] Add `deleteProject(id: string): Promise<void>` in `api.tsx`
- [ ] Add `exportProject(id: string): Promise<ProjectExport>` in `api.tsx`
- [ ] Add `importProject(data: ProjectExport, mode: ImportMode): Promise<Project>` in `api.tsx`
- [ ] Add `uploadAttachment(nodeId: string, file: File): Promise<Attachment>` in `api.tsx`
- [ ] Add `downloadAttachment(nodeId: string, attachmentId: string): Promise<Blob>` in `api.tsx`
- [ ] Add `deleteAttachment(nodeId: string, attachmentId: string): Promise<void>` in `api.tsx`
- [ ] Add `listAttachments(nodeId: string): Promise<Attachment[]>` in `api.tsx`

## Phase 3: UI Components

### 3.1 Project Selector Updates
- [ ] Add gear/settings icon button in `ProjectSelector.tsx` next to "+ New" button
- [ ] Add state for settings dialog open/closed
- [ ] Add click handler to open ProjectManagementDialog
- [ ] Style gear button to match existing design

### 3.2 Project Management Dialog Component
- [ ] Create new file `osint-graph-frontend/src/components/ProjectManagementDialog.tsx`
- [ ] Create dialog component with close button and backdrop
- [ ] Add tabbed interface with 4 tabs: General, Export, Import, Delete
- [ ] Add dialog state management (current tab, loading states)

#### Tab 1: General Settings
- [ ] Create form with fields:
  - Project name (text input, required)
  - Description (textarea, optional)
  - Tags (chip input with add/remove, optional)
  - User (text input or dropdown if we add user management later)
- [ ] Add form validation (name required, non-empty)
- [ ] Add "Save Changes" button
- [ ] Implement save handler calling `updateProject` API
- [ ] Show success/error toast messages
- [ ] Update parent component state on successful save

#### Tab 2: Export
- [ ] Add "Export Project" button
- [ ] Implement export handler:
  - Call `exportProject` API
  - Generate filename: `{project_name}_{timestamp}.json`
  - Create download blob and trigger download
- [ ] Add loading state during export
- [ ] Show success message after download
- [ ] Display export metadata (node count, link count, attachment count, file size)

#### Tab 3: Import
- [ ] Create file upload dropzone (drag & drop + click to browse)
- [ ] Accept only `.json` files
- [ ] On file selected, parse and validate JSON structure
- [ ] Show import mode selection dialog:
  - Radio buttons: "Create New Project" / "Overwrite Current" / "Merge with Current"
  - Description for each mode
  - Warning for overwrite mode
- [ ] Add "Import" button (disabled until mode selected)
- [ ] Implement import handler calling `importProject` API
- [ ] Show progress indicator during import
- [ ] On success, reload project data and close dialog
- [ ] Show success message with import stats
- [ ] Handle errors (invalid JSON, missing fields, server errors)

#### Tab 4: Delete
- [ ] Add warning text explaining cascade deletion
- [ ] Add confirmation input: "Type project name to confirm"
- [ ] Add "Delete Project" button (red, disabled until name matches)
- [ ] Implement delete handler:
  - Call `deleteProject` API
  - Clear localStorage project ID
  - Create new project or show project selector
  - Close dialog
- [ ] Show final confirmation dialog before delete
- [ ] Show success message after deletion

### 3.3 File Attachments UI in Node Editor
- [ ] Update node edit dialog in `App.tsx` to add attachments section
- [ ] Add "Attachments" section header with paperclip icon
- [ ] Display list of current attachments:
  - Filename with icon based on type
  - File size (formatted: KB/MB)
  - Download button
  - Delete button (with confirmation)
- [ ] Add "Upload File" button
- [ ] Implement file upload handler using `uploadAttachment` API
- [ ] Show upload progress indicator
- [ ] Implement download handler using `downloadAttachment` API
- [ ] Implement delete handler using `deleteAttachment` API
- [ ] Update node data after attachment changes
- [ ] Handle errors for all operations

### 3.4 Import Mode Dialog Component
- [ ] Create `ImportModeDialog.tsx` component
- [ ] Add three radio options with clear descriptions
- [ ] Add warning icon and text for overwrite mode
- [ ] Add confirm/cancel buttons
- [ ] Export dialog component for use in ProjectManagementDialog

## Phase 4: Integration & Testing

### 4.1 Backend Integration Tests
- [ ] Test complete export/import cycle (export then import with mode=new)
- [ ] Test export includes all attachments with correct compression
- [ ] Test import with attachments creates files correctly
- [ ] Test project update updates all fields correctly
- [ ] Test project deletion removes all associated data
- [ ] Test file upload/download roundtrip preserves data
- [ ] Run `just check` and ensure all tests pass

### 4.2 Frontend Integration
- [ ] Test settings dialog opens and closes correctly
- [ ] Test project update from UI updates backend and state
- [ ] Test export downloads valid JSON file
- [ ] Test import with valid JSON file succeeds
- [ ] Test import with invalid JSON shows error
- [ ] Test delete project flow completes successfully
- [ ] Test file upload/download in node editor
- [ ] Run `npm run lint` and fix any issues

### 4.3 Documentation
- [ ] Update `CLAUDE.md` with new Project schema fields
- [ ] Document export JSON structure in `CLAUDE.md`
- [ ] Document import modes (new/overwrite/merge) in `CLAUDE.md`
- [ ] Document file attachment system in `CLAUDE.md`
- [ ] Update API endpoint list in `CLAUDE.md`
- [ ] Add example export JSON to documentation

## Phase 5: Cleanup & Commit
- [ ] Run `just check` final validation
- [ ] Commit backend changes
- [ ] Commit frontend changes
- [ ] Commit documentation updates
- [ ] Delete this TODO.md file (all items complete!)

---

## Notes

### Backend Dependencies to Add
- `zstd` crate for compression (add to `osint-graph-backend/Cargo.toml`)
- `base64` crate if not already present
- `multipart` or `axum-multipart` for file uploads

### Frontend Dependencies to Add
- Consider `react-dropzone` for file upload UI
- Consider `@mui/material` Chip component for tags input (or build custom)

### Export JSON Schema
```json
{
  "version": "1.0",
  "exported_at": "2025-11-04T00:00:00Z",
  "project": {
    "id": "uuid",
    "name": "string",
    "description": "string",
    "tags": ["string"],
    "user": "uuid",
    "creationdate": "datetime",
    "last_updated": "datetime"
  },
  "nodes": [
    {
      "id": "uuid",
      "project_id": "uuid",
      "node_type": "string",
      "display": "string",
      "value": "string",
      "notes": "string",
      "pos_x": 123,
      "pos_y": 456,
      "updated": "datetime",
      "attachments": ["uuid"]
    }
  ],
  "links": [
    {
      "id": "uuid",
      "left": "uuid",
      "right": "uuid",
      "project_id": "uuid",
      "linktype": "Omni"
    }
  ],
  "attachments": {
    "node_uuid": [
      {
        "id": "uuid",
        "filename": "example.pdf",
        "content_type": "application/pdf",
        "size": 12345,
        "data": "base64_zstd_compressed_string",
        "created": "datetime"
      }
    ]
  }
}
```
