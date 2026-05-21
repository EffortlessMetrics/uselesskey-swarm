+++
id = "USELESSKEY-PLAN-0028"
kind = "plan"
title = "Source-of-truth control plane release handoff"
status = "implemented"
owner = "EffortlessMetrics"
created = "2026-05-21"
milestone = "control-plane"
linked_proposal = "USELESSKEY-PROP-0002"
linked_specs = [
  "USELESSKEY-SPEC-0002",
  "USELESSKEY-SPEC-0006",
  "USELESSKEY-SPEC-0016",
  "USELESSKEY-SPEC-0017",
  "USELESSKEY-SPEC-0020",
  "USELESSKEY-SPEC-0021",
  "USELESSKEY-SPEC-0022",
  "USELESSKEY-SPEC-0023",
]
linked_adrs = [
  "USELESSKEY-ADR-0002",
  "USELESSKEY-ADR-0003",
]
+++

# Source-of-Truth Control Plane Release Handoff

Date: 2026-05-21

This packet hands off the completed `uselesskey-swarm` control-plane lane for a
future source/release decision. It does not move release authority.

## Scope Landed In Swarm

- Closed badge queue hygiene with generated badge endpoint refresh PR #21.
- Added the fixture contract payload: negative fixture ledger, negative fixture
  matrix, bundle schemas, material classification, task-first docs spec, and
  public-surface matrix.
- Added the source-of-truth doctrine, artifact templates, PROP-0002, SPEC-0023,
  doc-artifact ledger, and `check-doc-artifacts`.
- Added public claim and support-tier maps plus `check-support-tiers`.
- Added `.uselesskey/goals/active.toml`, `check-goals`, the Codex operating
  contract, PR/issue templates, and advisory source-of-truth CI.
- Added repo-contract report, PR-body generator, and closeout generator.
- Bound OIDC/JWKS, JWT/token, webhook, and TLS/X.509 contract-pack docs to the
  negative fixture ledger, claim ledger, and support tiers.
- Simplified the first-five-minutes path and added downstream CI recipes.

## Public Claims

The current public claim source remains
[`policy/claim-ledger.toml`](../../policy/claim-ledger.toml), with reader
summaries in [`docs/status/PUBLIC_CLAIMS.md`](../../docs/status/PUBLIC_CLAIMS.md)
and [`docs/status/SUPPORT_TIERS.md`](../../docs/status/SUPPORT_TIERS.md).

Claims now explicitly include:

- scanner-safe fixtures;
- generated badge endpoints;
- OIDC/JWKS contract pack;
- JWT/token negative fixtures;
- webhook contract pack;
- TLS contract pack;
- metadata-only audit packets;
- bundle manifest schema;
- negative coverage receipt;
- external crates.io install smoke;
- public crate surface cleanup;
- `ripr` evidence surfaces.

No release headline should claim more than those ledgers and their proof
commands support.

## Support-Tier State

- Stable: scanner-safe fixtures, generated badge endpoints, public crate
  surface cleanup, OIDC/JWKS contract pack, webhook contract pack, TLS contract
  pack, and `ripr+` evidence endpoint.
- Stabilizing: JWT/token negative fixtures, metadata-only audit packets, bundle
  manifest schema, and negative coverage receipt.
- Advisory: external crates.io install smoke and `ripr` PR review evidence.
- Not supported: production key management, provider compatibility
  certification, downstream verifier correctness, and release/publish/signing
  authority in swarm.

## Proof Commands Used In The Lane

Representative local proof commands that protected the landed slices:

```bash
cargo xtask spec-check --strict
cargo xtask docs-sync --check
cargo xtask typos
cargo xtask check-doc-artifacts
cargo xtask check-support-tiers
cargo xtask check-goals
cargo xtask check-negative-fixtures
cargo xtask check-bundle-schemas
cargo xtask external-adoption-smoke --path .
cargo xtask external-adoption-smoke --path . --library-examples
cargo xtask pr
git diff --check
```

Hosted merge proof remained the normalized `Uselesskey Rust Small Result`.
Source-of-truth checks ran beside it in advisory mode.

## Policy Ledgers Changed

- [`policy/doc-artifacts.toml`](../../policy/doc-artifacts.toml): records
  proposal/spec/ADR/plan/status/policy artifacts and link expectations.
- [`policy/claim-ledger.toml`](../../policy/claim-ledger.toml): maps public
  claims to support tiers, proof commands, docs, and boundaries.
- [`policy/negative-fixtures.toml`](../../policy/negative-fixtures.toml): maps
  stable negative fixture IDs to ownership, public surfaces, bundle exposure,
  scanner-safety posture, runtime-material posture, and boundaries.

## Deferred Work

- Keep source-of-truth CI advisory until it has enough clean real-PR history.
- Promote only normalized source-of-truth results if branch protection needs
  another required check.
- Continue hardening `check-bundle-schemas`, metadata-only audit tests, and
  support-tier enforcement as real product changes expose false positives.
- Productize the generic proof-stack pattern later in `tokmd`; do not build the
  generic platform inside this repo.

## Source Sync Needs

Before any source sync from swarm to `EffortlessMetrics/uselesskey`, compare:

- docs and policy ledgers;
- `.uselesskey/goals/` state;
- GitHub templates and advisory workflow;
- generated target-only receipts that should not be synced;
- release and publish workflow boundaries.

The sync should preserve the public-source boundary and should not infer a
release from this handoff.

## Release Risks

- New docs and ledgers increase public-claim precision; release notes must not
  overstate production security, provider compatibility, or downstream verifier
  correctness.
- Stabilizing claims should remain clearly labeled until their proof commands
  are treated as release-quality evidence.
- Generated runtime material must remain out of docs, receipts, schemas, and
  review artifacts.
- Advisory source-of-truth checks should not become required until burn-in
  shows low false-positive risk.

## Boundary

`EffortlessMetrics/uselesskey` still owns release, publish, signing, tags,
GitHub releases, crates.io authority, and public source synchronization.
This swarm handoff is release input, not release execution.
