# Contributing with Rails

When adding or updating source-of-truth artifacts:

1. Put durable artifacts under `.rails/`.
2. Index every owned artifact through `.rails/index.toml`.
3. Keep sequencing in focused lane trackers under `.rails/lanes/`.
4. Keep external namespaces awareness-only: do not create Rails-owned artifacts in `.codex/`, `.spec/`, `.claude/`, or `.jules/`.

## Artifact ownership model

- Proposals belong in `.rails/proposals/`.
- Specs belong in `.rails/specs/`.
- ADRs belong in `.rails/adr/`.
- Lane trackers belong in `.rails/lanes/`.
- Closeouts belong in `.rails/closeouts/`.
- Support claim mappings belong in `.rails/support/`.
- Policy references belong in `.rails/policy/`.

Use repo-scoped IDs such as `USELESSKEY-PROP-*`, `USELESSKEY-SPEC-*`, and `USELESSKEY-ADR-*`.
