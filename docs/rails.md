# Rails framework

This repository uses `.rails/` as the durable Rails knowledge base.

- `.rails/` stores proposals, specs, ADRs, lanes, templates, support maps, policy references, receipts, schemas, and closeouts.
- `docs/` provides human-facing explanation and adoption guidance.

## Scope boundaries

Rails does not own external tool or agent state directories:

- `.codex/` is Codex execution state and awareness-only for Rails.
- `.spec/` is Spec Kit / speckit state and awareness-only for Rails.
- `.claude/` and `.jules/` are external agent/session state and awareness-only for Rails.

No Rails-owned artifact path should live under those directories.
