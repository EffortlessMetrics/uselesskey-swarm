# Rails Migration Status

This map records which source-of-truth artifacts Rails owns now, which existing
uselesskey artifacts remain authoritative, and what must be true before anything
moves or mirrors into `.rails/`.

## Status Terms

- `rails-owned`: committed Rails artifact and portable across repos.
- `indexed-kept`: existing uselesskey artifact remains authoritative and is
  indexed from `.rails/index.toml`.
- `generated-only`: produced under `target/` or another generated location; do
  not treat as source.
- `awareness-only`: external tool or agent namespace; never place Rails-owned
  truth there.

## Current Map

| Surface | Current authority | Rails state | Migration rule | Proof/check |
| --- | --- | --- | --- | --- |
| Rails index | `.rails/index.toml` | `rails-owned` | Keep as the first read for Rails-aware agents. | `git diff --check` |
| Rails templates | `.rails/templates/` | `rails-owned` | Add portable templates here; do not duplicate tool scratch state. | `cargo xtask docs-sync --check` |
| Rails lanes | `.rails/lanes/` | `rails-owned` | Track Rails-specific lane sequence here; keep `.uselesskey/goals/` working until a later migration explicitly changes it. | `cargo xtask docs-sync --check` |
| Rails closeouts | `.rails/closeouts/` | `rails-owned` | Use for Rails framework closeouts; existing uselesskey handoffs stay in place. | `cargo xtask docs-sync --check` |
| Proposals | `docs/proposals/` | `indexed-kept` | Do not move or mirror until `policy/doc-artifacts.toml` and affected links are updated in the same PR. | `cargo xtask check-doc-artifacts` |
| Specs | `docs/specs/` | `indexed-kept` | Do not move while `cargo xtask spec-check --strict` expects current paths. | `cargo xtask spec-check --strict` |
| ADRs | `docs/adr/` | `indexed-kept` | Preserve current ADR paths until doc-artifact and docs links are migrated together. | `cargo xtask check-doc-artifacts` |
| Implementation plans | `plans/` | `indexed-kept` | Keep current plan paths stable for active goals and PR-body generation. | `cargo xtask check-goals` |
| Active goals | `.uselesskey/goals/` | `indexed-kept` | Keep as the active uselesskey agent state until a dedicated goal migration exists. | `cargo xtask check-goals` |
| Claim ledger | `policy/claim-ledger.toml` | `indexed-kept` | Keep public claim truth in policy until support-tier checks understand any new location. | `cargo xtask check-support-tiers` |
| Policy ledgers | `policy/*.toml` | `indexed-kept` | Keep governed fixture, artifact, and claim state in policy unless a migration updates all checkers. | `cargo xtask check-doc-artifacts` |
| Support status | `docs/status/` | `indexed-kept` | Keep user-facing support tier and claim docs stable for current checkers. | `cargo xtask check-support-tiers` |
| Handoffs | `docs/handoffs/` and `plans/release-handoff/` | `indexed-kept` | Keep existing release and lane handoffs where current docs link to them. | `cargo xtask docs-sync --check` |
| Receipts and reports | `target/source-of-truth/` | `generated-only` | Regenerate as evidence; do not commit generated receipts unless a specific policy says so. | command-specific receipt proof |
| Agent/runtime dirs | `.codex/`, `.spec/`, `.claude/`, `.jules/` | `awareness-only` | Never place Rails-owned durable artifacts here. | review `.rails/index.toml` |

## Migration Guardrails

Any later migration or mirroring PR must:

1. keep existing uselesskey checks green while the move lands;
2. update `.rails/index.toml` and this map in the same PR;
3. update `policy/doc-artifacts.toml` if an indexed artifact path changes;
4. update affected docs links and generated PR/closeout inputs;
5. avoid release, publish, signing, tag, GitHub release, crates.io, and
   source-sync changes.
