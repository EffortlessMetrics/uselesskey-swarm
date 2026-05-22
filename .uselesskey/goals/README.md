# Active Goals

This directory stores machine-readable lane state for agents working in
`uselesskey`.

The active goal manifest is the current execution source only when
`.uselesskey/goals/active.toml` has `status = "active"`. Handoffs, learnings,
and archived manifests record history; they should not be treated as active
instructions.

## Files

- `active.toml` - Current agent lane when `status = "active"`; historical
  state when `status = "archived"`.
- `archive/` - Completed or superseded active-goal manifests.
- `templates/active.toml` - Template for new active goals.

Do not create `active.toml` until the lane has an accepted proposal or spec to
link to.

When `active.toml` is archived, there is no current uselesskey goal. Use
`.rails/index.toml`, `.rails/migration-status.md`, and the newest explicit user
instruction to choose one narrow aligned improvement. Repo-local generators such
as `cargo xtask pr-body` and `cargo xtask closeout` require an active manifest
before they write new work artifacts.

When resuming agent work, read
[`docs/handoffs/agent-bootstrap.md`](../../docs/handoffs/agent-bootstrap.md)
before acting on older chat context.
