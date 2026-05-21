+++
id = "USELESSKEY-PLAN-0029"
kind = "plan"
title = "v0.10.0 release readiness"
status = "accepted"
owner = "EffortlessMetrics"
created = "2026-05-21"
milestone = "v0.10.0"
linked_proposal = "USELESSKEY-PROP-0001"
linked_specs = [
  "USELESSKEY-SPEC-0002",
  "USELESSKEY-SPEC-0006",
  "USELESSKEY-SPEC-0013",
  "USELESSKEY-SPEC-0014",
  "USELESSKEY-SPEC-0017",
  "USELESSKEY-SPEC-0020",
  "USELESSKEY-SPEC-0024",
]
linked_adrs = [
  "USELESSKEY-ADR-0002",
  "USELESSKEY-ADR-0003",
  "USELESSKEY-ADR-0004",
]
+++

# v0.10.0 Release Readiness Implementation Plan

Plan id: `USELESSKEY-PLAN-0029`

## Goal

Prove that the v0.10.0 release candidate matches the now-documented user path
without publishing, tagging, signing, pushing to crates.io, creating a GitHub
release, or moving source-sync authority.

## Non-goals

- No publish.
- No tag.
- No crates.io push.
- No GitHub release.
- No signing.
- No source sync.
- No production security claim.
- No provider compatibility claim.
- No downstream verifier correctness claim.

## Artifact Map

| Artifact | Role |
| --- | --- |
| `docs/specs/USELESSKEY-SPEC-0024-v0.10.0-release-readiness.md` | Behavior and proof contract for release readiness. |
| `plans/v0.10.0-release-readiness/implementation-plan.md` | PR sequence and proof map. |
| `.uselesskey/goals/active.toml` | Current release-readiness work queue. |
| `docs/handoffs/2026-05-21-source-of-truth-control-plane-closeout.md` | Prior lane closeout. |
| `plans/release-handoff/2026-05-21-source-of-truth-control-plane.md` | Input packet from control-plane lane. |
| `policy/claim-ledger.toml` | Public claim proof and boundary map. |
| `docs/status/SUPPORT_TIERS.md` | Support posture for release-facing claims. |

## PR Sequence

| Item | Status | Scope |
| --- | --- | --- |
| Open release-readiness goal | Done | Add SPEC-0024, this plan, active goal, and doc-artifact ledger entries. |
| Release surface inventory | Done | Matrix of surface, user command/snippet, proof, release risk, and owner. |
| Version/snippet reconciliation | Done | Decide every `0.9.1` snippet as current stable, v0.10.0 candidate, or post-publish update. |
| Installed CLI release smoke | Done | Prove `doctor`, `bundle`, `verify-bundle`, `inspect-bundle`, and strict CI audit from checkout. |
| Facade release smoke | Ready | Prove external library examples against the release-candidate checkout. |
| Package dry-run | Planned | Validate package contents, README render inputs, docs/schemas inclusion, and absence of target receipts. |
| Release readiness record | Planned | Record proof, hosted CI, known non-blockers, publish order, rollback, and claim boundaries. |

## Per-PR Proof Commands

Use the active goal work item commands first. Common lane-opening proof:

```bash
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask docs-sync --check
cargo xtask typos
cargo xtask pr
git diff --check
```

Release-candidate proof slices use:

```bash
cargo xtask external-adoption-smoke --path .
cargo xtask external-adoption-smoke --path . --library-examples
cargo test -p uselesskey-cli --all-features bundle verify_bundle audit_bundle
cargo xtask publish-preflight
cargo xtask publish-check
cargo xtask no-blob
```

## Rollback

Revert the PR that opened the lane if the release-readiness goal is premature.
For later slices, revert the affected PR and move the work item back to `ready`
or `blocked` with an explicit reason.

## Promotion Rules

- A readiness record can say "ready for release review" only after every active
  work item is done or explicitly deferred.
- A release note can claim only what `policy/claim-ledger.toml` and
  `docs/status/SUPPORT_TIERS.md` support.
- Publication remains a separate explicit instruction and must run in the
  release/public-source boundary repo.

## Closeout Shape

The lane closeout records:

- release-facing surfaces checked;
- version snippets changed or deferred;
- installed CLI and facade smoke evidence;
- package dry-run evidence;
- hosted CI evidence;
- known non-blockers;
- publish order and rollback notes;
- explicit non-claims and source-boundary reminders.
