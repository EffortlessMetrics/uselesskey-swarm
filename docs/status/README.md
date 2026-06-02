# Status

Status docs map public claims to support tiers, proof commands, receipts, and
user-facing docs.

This directory is for compact indexes and ledgers. Put long rationale in a
proposal, behavior contract in a spec, and historical lessons in `docs/learnings/`.

## Expected Surfaces

Status docs cover:

- Public README claims and their proof commands
- Support tiers for contract packs and adapter surfaces
- Release evidence rows for stable claims
- Claim boundaries for scanner-safe, cryptographic, and registry-facing signals

No status page should imply enforcement before the matching spec and `xtask`
check exist.

## Index

- [PUBLIC_CLAIMS.md](PUBLIC_CLAIMS.md) - Public claim to proof-command and boundary map
- [SUPPORT_TIERS.md](SUPPORT_TIERS.md) - Claim and workflow support-tier map
- [negative-fixture-matrix.md](negative-fixture-matrix.md) - Negative fixture stable ID implementation state
- [public-surface-matrix.md](public-surface-matrix.md) - Compact public crate promise matrix
- [ci-check-policy.md](ci-check-policy.md) - Required, advisory, triage, and route-only CI check roles
- [workflow-support.md](workflow-support.md) - User workflow to claim, support tier, proof, receipt, and boundary map
