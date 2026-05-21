# Contributing with Rails

When adding or updating source-of-truth artifacts:

1. Put durable artifacts under `.rails/`.
2. Index every owned artifact through `.rails/index.toml`.
3. Keep sequencing in focused lane trackers under `.rails/lanes/`.
4. Keep existing `docs/specs`, `policy/*.toml`, `.uselesskey/goals`, and
   release-readiness artifacts working while migration is staged.
5. Keep external namespaces awareness-only: do not create Rails-owned artifacts
   in `.codex/`, `.spec/`, `.claude/`, or `.jules/`.

## Artifact ownership model

- Proposals belong in `.rails/proposals/`.
- Specs belong in `.rails/specs/`.
- ADRs belong in `.rails/adr/`.
- Lane trackers belong in `.rails/lanes/`.
- Closeouts belong in `.rails/closeouts/`.
- Support tier and public-claim state remains in `docs/status/` and
  `policy/claim-ledger.toml` unless a migration lane explicitly moves or
  mirrors it.
- Policy ledgers remain in `policy/*.toml` unless a migration lane explicitly
  moves or mirrors them.

Use Rails-scoped IDs such as `RAILS-PROP-*`, `RAILS-SPEC-*`,
`RAILS-ADR-*`, and `RAILS-LANE-*` for portable framework artifacts. Existing
`USELESSKEY-*` IDs remain valid for the current uselesskey specs, ADRs, and
plans.
