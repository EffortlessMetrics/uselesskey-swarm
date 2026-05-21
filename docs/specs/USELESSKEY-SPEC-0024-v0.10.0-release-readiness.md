+++
id = "USELESSKEY-SPEC-0024"
kind = "spec"
title = "v0.10.0 release readiness"
status = "accepted"
owner = "EffortlessMetrics"
created = "2026-05-21"
milestone = "v0.10.0"
linked_proposal = "USELESSKEY-PROP-0001"
linked_adrs = [
  "USELESSKEY-ADR-0002",
  "USELESSKEY-ADR-0003",
  "USELESSKEY-ADR-0004",
]
linked_plan = "plans/v0.10.0-release-readiness/implementation-plan.md"
linked_specs = [
  "USELESSKEY-SPEC-0002",
  "USELESSKEY-SPEC-0006",
  "USELESSKEY-SPEC-0013",
  "USELESSKEY-SPEC-0014",
  "USELESSKEY-SPEC-0017",
  "USELESSKEY-SPEC-0020",
  "USELESSKEY-SPEC-0022",
  "USELESSKEY-SPEC-0023",
]
support_tier_impact = []
policy_impact = []
+++

# USELESSKEY-SPEC-0024: v0.10.0 Release Readiness

## Problem

The control-plane lane made the user path and proof surfaces explicit, but a
release candidate still needs a separate readiness lane. That lane must prove
the documented path against the candidate state without quietly publishing or
turning swarm into the release authority.

The repo needs to answer:

```text
Which v0.10.0 surfaces are release-facing?
Which commands prove them?
Which snippets are current stable versus release-candidate guidance?
Which checks must run before publication?
Which claims remain out of scope?
```

## Behavior

The v0.10.0 release-readiness lane prepares release evidence only. It must:

- inventory public user surfaces, commands, proof, release risk, and owner;
- reconcile `0.9.1`, current-stable, and release-candidate snippets without
  pretending that unpublished versions are installable;
- prove the installed CLI user path from a release-candidate checkout:
  `doctor`, `bundle`, `verify-bundle`, `inspect-bundle`, and
  `audit-bundle --ci --expect-profile --policy strict`;
- prove facade examples against the release-candidate checkout;
- validate package contents through dry-run packaging and blob checks;
- write a readiness record with proof commands, hosted CI, known non-blockers,
  publish order, rollback, and what not to claim.

This lane may produce release readiness records and source-sync input. It must
not publish crates, create tags, sign artifacts, push to crates.io, create a
GitHub release, or move source-sync authority.

## Non-goals

This spec does not:

- publish v0.10.0;
- create a release tag;
- sign artifacts;
- push to crates.io;
- create or edit a GitHub release;
- sync swarm to `EffortlessMetrics/uselesskey`;
- claim production security, provider compatibility, scanner-policy bypass
  approval, or downstream verifier correctness;
- add unrelated infrastructure unless it directly improves release/user-path
  proof.

## Required Evidence

Opening the lane must run:

```bash
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask docs-sync --check
cargo xtask typos
cargo xtask pr
git diff --check
```

Release-readiness slices use narrower work-item evidence first. The full lane
evidence set includes:

```bash
cargo xtask external-adoption-smoke --path .
cargo xtask external-adoption-smoke --path . --library-examples
cargo test -p uselesskey-cli --all-features bundle verify_bundle audit_bundle
cargo xtask check-support-tiers
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask no-blob
cargo xtask publish-preflight
cargo xtask publish-check
git diff --check
```

## Claim Boundary

This lane proves release readiness for documented test-fixture workflows. It
does not prove production security, provider compatibility, downstream verifier
correctness, scanner-policy bypass approval, publish success, or future registry
state.

## Acceptance

This spec is accepted when the repo has an active release-readiness goal, a
linked implementation plan, and a PR sequence that keeps readiness separate from
publication.

This spec is implemented when:

- release-facing surfaces are inventoried;
- version snippets are reconciled;
- installed CLI release-candidate smoke passes;
- facade release-candidate smoke passes;
- package contents are dry-run validated;
- readiness record states proof, non-blockers, publish order, rollback, and
  claim boundaries;
- no release authority moved into swarm.

## Acceptance Examples

Acceptable release-candidate CLI proof:

```bash
uselesskey doctor --format json
uselesskey bundle --profile oidc --out target/uselesskey-oidc
uselesskey verify-bundle target/uselesskey-oidc
uselesskey inspect-bundle target/uselesskey-oidc
uselesskey audit-bundle \
  --path target/uselesskey-oidc \
  --ci \
  --expect-profile oidc \
  --policy strict
```

Acceptable snippet reconciliation:

```text
0.9.1 remains current stable until v0.10.0 is published.
0.10.0 snippets are marked release-candidate or staged for release notes.
```

Not acceptable:

```text
cargo publish
git tag v0.10.0
gh release create v0.10.0
```

## Test Mapping

| Requirement | Evidence |
| --- | --- |
| Active release-readiness goal parses | `cargo xtask check-goals` |
| Spec and plan are linked | `cargo xtask check-doc-artifacts`; `cargo xtask docs-sync --check` |
| Installed CLI path works from checkout | `cargo xtask external-adoption-smoke --path .`; CLI tests |
| Facade examples work from checkout | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Package contents avoid generated payloads | `cargo xtask publish-preflight`; `cargo xtask publish-check`; `cargo xtask no-blob` |
| Claims remain bounded | `cargo xtask check-support-tiers`; readiness record review |

## Implementation Mapping

| Surface | Owner |
| --- | --- |
| Release-readiness spec | `docs/specs/USELESSKEY-SPEC-0024-v0.10.0-release-readiness.md` |
| Release-readiness plan | `plans/v0.10.0-release-readiness/implementation-plan.md` |
| Active goal | `.uselesskey/goals/active.toml` |
| Release surface inventory | `docs/release/` or `plans/v0.10.0-release-readiness/` |
| Version snippet reconciliation | README, how-to docs, examples, and release docs touched by the inventory |
| Installed CLI smoke | `crates/uselesskey-cli` tests and `cargo xtask external-adoption-smoke --path .` |
| Facade smoke | `examples/external/` and library example smoke |
| Package dry-run | `cargo xtask publish-preflight`; `cargo xtask publish-check` |
| Readiness record | `docs/handoffs/` or `plans/v0.10.0-release-readiness/` |

## Rollback

Revert the release-readiness PR and restore the prior active goal manifest if
the lane is opened prematurely. If a later readiness slice is wrong, revert that
slice and return its work item to `ready` or `blocked` with a concrete reason.

## CI Proof

Lane-opening PR:

```bash
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask docs-sync --check
cargo xtask typos
cargo xtask pr
git diff --check
```

Release-readiness closeout PR:

```bash
cargo xtask external-adoption-smoke --path .
cargo xtask external-adoption-smoke --path . --library-examples
cargo xtask check-support-tiers
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask publish-preflight
cargo xtask publish-check
cargo xtask no-blob
git diff --check
```

## Metrics / Promotion Rule

The lane is ready for release handoff when every work item in
`.uselesskey/goals/active.toml` is `done`, the readiness record names any
deferred or non-blocking gaps, and the publication repo can use the packet
without reading chat history.

