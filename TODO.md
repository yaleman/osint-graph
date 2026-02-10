# OSINT Graph TODO (Current Priorities)

This list tracks remaining work that is not already shipped.

## P0 - Complete Project Import End-to-End

- [ ] Backend: add `POST /api/v1/project/import` with `mode=new|overwrite|merge`.
- [ ] Validate import payloads against `ProjectExport` structure with clear error responses.
- [ ] Implement ID remapping logic for nodes, links, and attachments during import.
- [ ] Frontend: replace import tab "Coming Soon" with JSON file select/drop UI.
- [ ] Add import mode selection with explicit overwrite confirmation.
- [ ] After successful import, reload graph state and show import summary counts.

## P1 - UX and Accessibility Cleanup

- [ ] Make project settings tabs keyboard-accessible (`role="tab"` semantics and key navigation).
- [ ] Remove no-op `onKeyDown={() => {}}` handlers in `ProjectManagementDialog`.
- [ ] Remove inline styles in frontend dialogs and move styling to CSS classes.
- [ ] Ensure long-running actions (export/import/delete) disable duplicate submissions.

## P1 - Attachment Performance and Safety

- [ ] Refactor attachment download/view to stream data instead of fully buffering large files.
- [ ] Add tests for large attachment handling and decompression failure paths.
- [ ] Confirm attachment size limits and error messages are consistent across API and UI.

## P2 - Runtime and Auth Hardening

- [x] Implement OIDC discovery retry task when initial provider discovery fails.
- [x] Implement SIGHUP reload behavior or remove the placeholder path with explicit documentation.
- [x] Resolve dead-code allowances in auth middleware where possible.

## P2 - Testing and Documentation Hygiene

- [ ] Add missing parser tests noted in shared data modules.
- [ ] Add integration coverage for import modes and export/import round-trip behavior.
- [ ] Keep endpoint docs aligned with implemented routes (for example attachment endpoints).
- [ ] Run `just check` clean before closing each milestone.
