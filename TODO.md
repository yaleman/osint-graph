# OSINT Graph TODO (Current Priorities)

This list tracks remaining work that is not already shipped.

## P1 - UX and Accessibility Cleanup

- [ ] Make project settings tabs keyboard-accessible (`role="tab"` semantics and key navigation).
- [ ] Remove no-op `onKeyDown={() => {}}` handlers in `ProjectManagementDialog`.
- [ ] Remove inline styles in frontend dialogs and move styling to CSS classes.
- [ ] Ensure long-running actions (export/import/delete) disable duplicate submissions.

## P2 - Testing and Documentation Hygiene

- [ ] Keep endpoint docs aligned with implemented routes (for example attachment endpoints).
- [ ] Run `just check` clean before closing each milestone.
