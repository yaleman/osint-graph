# OSINT Graph TODO (Current Priorities)

This list tracks remaining work that is not already shipped.

## P1 - UX and Accessibility Cleanup

- [ ] Make project settings tabs keyboard-accessible (`role="tab"` semantics and key navigation).
- [ ] Remove no-op `onKeyDown={() => {}}` handlers in `ProjectManagementDialog`.
- [ ] Remove inline styles in frontend dialogs and move styling to CSS classes.
- [ ] Ensure long-running actions (export/import/delete) disable duplicate submissions.

## P1 - Attachment Performance and Safety

- [ ] Refactor attachment download/view to stream data instead of fully buffering large files.
- [ ] Add tests for large attachment handling and decompression failure paths.
- [ ] Confirm attachment size limits and error messages are consistent across API and UI.

## P2 - Testing and Documentation Hygiene

- [ ] Add missing parser tests noted in shared data modules.
- [ ] Keep endpoint docs aligned with implemented routes (for example attachment endpoints).
- [ ] Run `just check` clean before closing each milestone.
