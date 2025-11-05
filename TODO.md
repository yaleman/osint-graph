# Project Management UI Implementation TODO

## Phase 1: Backend Schema & Database

### 1.3 Backend API Endpoints - Export/Import

- [ ] Add `POST /api/v1/project/import` endpoint with query param `mode` (new/overwrite/merge)
- [ ] Implement `import_project` handler:
  - mode=new: Generate new UUIDs for everything, preserve relationships
  - mode=overwrite: Delete existing project and replace with import data
  - mode=merge: Add nodes/links to existing project with new UUIDs
- [ ] Write test for import mode=new
- [ ] Write test for import mode=overwrite
- [ ] Write test for import mode=merge

### 1.4 Backend API Endpoints - File Attachments

- [ ] Add `POST /api/v1/node/{id}/attachment` endpoint (multipart/form-data)
- [ ] Implement `upload_attachment` handler with zstd compression
- [ ] Write test for file upload with various file types
- [ ] Write test for file upload size limits
- [ ] Add `GET /api/v1/node/{id}/attachment/{attachment_id}` endpoint
- [ ] Implement `download_attachment` handler with decompression
- [ ] Write test for file download
- [ ] Add `DELETE /api/v1/node/{id}/attachment/{attachment_id}` endpoint
- [ ] Implement `delete_attachment` handler
- [ ] Write test for attachment deletion
- [ ] Add `GET /api/v1/node/{id}/attachments` endpoint (list all attachments for a node)
- [ ] Implement `list_attachments` handler
- [ ] Write test for listing attachments
- [ ] update project export endpoint to include zstd-compressed base64-encoded attachments when an additional query parameter of "include_attachments" is set to true

## Phase 2: Frontend Types & API

### 2.1 Update Frontend Types

- [ ] Create `Attachment` interface in `types.tsx`:
  - id: string
  - filename: string
  - content_type: string
  - size: number
  - created: string
- [ ] Update `OSINTNode` interface to add `attachments?: string[]` (array of attachment IDs)
- [ ] Create `ImportMode` type: 'new' | 'overwrite' | 'merge'

### 2.2 Frontend API Client

- [ ] Add `importProject(data: ProjectExport, mode: ImportMode): Promise<Project>` in `api.tsx`
- [ ] Add `uploadAttachment(nodeId: string, file: File): Promise<Attachment>` in `api.tsx`
- [ ] Add `downloadAttachment(nodeId: string, attachmentId: string): Promise<Blob>` in `api.tsx`
- [ ] Add `deleteAttachment(nodeId: string, attachmentId: string): Promise<void>` in `api.tsx`
- [ ] Add `listAttachments(nodeId: string): Promise<Attachment[]>` in `api.tsx`

## Phase 3: UI Components

### 3.2 Import Tab Implementation

- [ ] Replace "Coming Soon" placeholder with functional import UI
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

- [ ] Test import with valid JSON file succeeds
- [ ] Test import with invalid JSON shows error
- [ ] Test file upload/download in node editor

### 4.3 Documentation

- [ ] Document export JSON structure in `CLAUDE.md`
- [ ] Document import modes (new/overwrite/merge) in `CLAUDE.md`
- [ ] Document file attachment system in `CLAUDE.md`
- [ ] Add example export JSON to documentation

## Phase 5: Cleanup & Polish

- [ ] Run `just check` final validation
- [ ] Update CLAUDE.md with latest features
- [ ] Clean up TODO.md (remove completed sections)

## Notes

### Dependencies Needed for Remaining Features

**Backend:**

- `multipart` or `axum-multipart` for file uploads (attachments feature)

**Frontend:**

- Consider `react-dropzone` for file upload UI (import tab and attachments)
