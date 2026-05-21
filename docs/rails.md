# Rails Framework

This repository uses `.rails/` as the durable Rails control-plane footprint.

Start from:

```text
.rails/index.toml
```

The index points to the active lane, portable templates, Rails-owned
directories, and existing source-of-truth artifacts that remain authoritative
while migration is staged.

For the current authority and migration state of existing uselesskey artifacts,
read:

```text
.rails/migration-status.md
```

## Ownership Model

- `.rails/proposals/` owns why a lane exists.
- `.rails/specs/` owns behavior and proof contracts.
- `.rails/adr/` owns durable decisions.
- `.rails/lanes/` owns PR-sized sequencing and active work.
- `.rails/templates/` owns portable scaffolds.
- `.rails/closeouts/` owns proof-backed handoff.

Existing `docs/specs/`, `policy/*.toml`, `docs/status/`, `plans/`, and
`.uselesskey/goals/` artifacts continue to work and remain indexed from
`.rails/index.toml`.

## Boundaries

Rails does not own external tool or agent state directories:

- `.codex/` is Codex execution state and awareness-only for Rails.
- `.spec/` is Spec Kit / speckit state and awareness-only for Rails.
- `.claude/` and `.jules/` are external agent/session state and awareness-only
  for Rails.

No Rails-owned artifact path should live under those directories.
