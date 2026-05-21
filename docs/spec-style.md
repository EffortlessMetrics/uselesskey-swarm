# Spec style and durable ownership

The full source-of-truth chain remains required:

`roadmap -> proposal -> spec -> ADR (as needed) -> lane tracker -> implementation plan -> PRs -> proof -> support/policy -> closeout`.

The durable home for that chain is `.uselesskey-spec/`, not agent/tool state
folders.

## Namespace model

- `.uselesskey-spec/` = durable repo-owned rails
- `docs/` = contributor explanation and human-facing guidance
- `policy/` = live ledgers referenced by specs/lane work
- `plans/` = only when part of repo-native planning surfaces

Awareness-only (external state): `.codex/`, `.spec/`, `.claude/`, `.jules/`.
