+++
id = "USELESSKEY-SPEC-0023"
kind = "spec"
title = "Source-of-truth enforcement"
status = "accepted"
owner = "EffortlessMetrics"
created = "2026-05-21"
milestone = "control-plane"
linked_proposal = "USELESSKEY-PROP-0002"
linked_adrs = [
  "USELESSKEY-ADR-0002",
  "USELESSKEY-ADR-0003",
]
support_tier_impact = [
  "docs/status/SUPPORT_TIERS.md",
]
policy_impact = [
  "policy/doc-artifacts.toml",
  "policy/claim-ledger.toml",
  "policy/negative-fixtures.toml",
]
+++

# USELESSKEY-SPEC-0023: Source-of-Truth Enforcement

## Problem

`USELESSKEY-SPEC-0001` defines the source-of-truth model, but the model needs an
enforcement contract before later PRs add ledgers, checks, PR-body generation,
closeout generation, and advisory CI.

The repo should be able to prove that proposals, specs, ADRs, plans, active
goals, support tiers, policy ledgers, proof commands, PR bodies, and closeouts
are linked enough for humans and agents to work from repo state instead of chat
history.

Without a validation contract, source-of-truth files can still drift:

- accepted specs can link missing proposals, ADRs, or plans;
- active goal work items can point at stale or nonexistent artifacts;
- stable claims can appear without proof commands or boundaries;
- policy ledgers can define IDs that status matrices do not mirror;
- closeouts can omit what changed, what was proven, and what remains.

## Behavior

The source-of-truth control plane must maintain a traversable graph across these
artifact families:

| Artifact family | Required check |
| --- | --- |
| Proposal/spec/ADR/plan documents | Parse front matter, validate IDs, status, kind, and links. |
| Active goal manifests | Parse TOML, validate work-item IDs, statuses, links, and commands. |
| Public claims and support tiers | Validate claim IDs, tiers, proof commands, docs, specs, and boundaries. |
| Policy ledgers | Validate IDs, statuses, required fields, mirrored docs, and checker coverage. |
| Receipts and generated reports | Validate schema version, command provenance, metadata-only boundaries, and output paths. |
| PR and closeout packets | Validate links, proof commands, claim boundaries, rollback, and remaining work. |

Validation should be incremental. Each checker owns one part of the graph and
can be promoted only after it proves stable on real PRs.

Initial command targets are:

```bash
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask check-support-tiers
cargo xtask check-claim-proof-policy
cargo xtask claim-report --check-public-claims
cargo xtask check-negative-fixtures
cargo xtask check-bundle-schemas
```

Generated report and packet targets are:

```bash
cargo xtask repo-contract-report
cargo xtask pr-body --work-item <id>
cargo xtask closeout --goal <goal-id>
```

Advisory CI may run all source-of-truth checks, but branch protection should
promote only the stable core after burn-in.

## Non-goals

This spec does not add fixture behavior.

This spec does not move release, publish, signing, crates.io, GitHub release,
tag, or source-sync authority into `uselesskey-swarm`.

This spec does not make advisory checks blocking by itself.

This spec does not productize a generic `tokmd` proof-stack system.

This spec does not duplicate support-tier truth inside specs or CI lane truth
inside specs. It requires links to the owning artifacts instead.

This spec does not require installed CLI commands to execute repo-local xtask
proof commands.

## Required Evidence

Docs-only changes to this spec should run:

```bash
cargo xtask spec-check --strict
cargo xtask docs-sync --check
git diff --check
```

When individual checkers are added, each checker PR must include:

```bash
cargo test -p xtask <checker_test_filter>
cargo xtask <checker-command>
git diff --check
```

When advisory CI is added, it must run without replacing the existing normalized
`Uselesskey Rust Small Result` gate.

## Claim Boundary

This spec proves the control-plane graph can be made checkable and reportable.

It does not prove downstream verifier correctness, provider compatibility,
production token security, production PKI behavior, or release readiness.

It does not claim a checker exists until the corresponding xtask command is
implemented and wired into evidence.

## Acceptance

This spec is accepted when:

- it defines the artifact families that belong to the control-plane graph;
- it defines the initial checker commands and generated packet commands;
- it separates advisory burn-in from blocking promotion;
- it preserves release and source-sync boundaries;
- it defines proof expectations for docs-only and checker PRs.

This spec is implemented when:

- `policy/doc-artifacts.toml` inventories source-of-truth artifacts;
- `cargo xtask check-doc-artifacts` validates document links and statuses;
- `cargo xtask check-goals` validates active and archived goal manifests;
- `cargo xtask check-support-tiers` validates support-tier and claim mapping;
- `cargo xtask check-negative-fixtures` validates negative fixture policy;
- `cargo xtask check-bundle-schemas` validates generated bundle outputs against
  published schemas;
- advisory source-of-truth CI has run cleanly on real PRs;
- repo contract reports, PR bodies, and closeouts can be generated from
  committed artifacts.

## Acceptance Examples

Valid accepted spec link:

```toml
id = "USELESSKEY-SPEC-0023"
kind = "spec"
status = "accepted"
linked_proposal = "USELESSKEY-PROP-0002"
linked_adrs = ["USELESSKEY-ADR-0003"]
```

Invalid active goal work item:

```toml
[[work_item]]
id = "missing-spec"
status = "ready"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-9999"
commands = ["cargo xtask check-goals"]
```

`check-goals` must reject the missing spec link.

Valid promotion rule:

```text
check-doc-artifacts and check-goals may become blocking after clean advisory
burn-in; check-support-tiers, check-negative-fixtures, and check-bundle-schemas
remain advisory until their false-positive risk is low.
```

## Test Mapping

Checker tests should cover at least:

- duplicate artifact IDs;
- missing artifact files;
- unknown statuses;
- missing proposal/spec/ADR links;
- accepted specs without proposal or standalone reason;
- active goals with unsupported fields;
- done work items without commands or receipts;
- blocked work items without a blocker;
- stable claims without proof commands;
- implemented negative fixture entries without docs, tests, owner crate, or
  public surface;
- bundle schema examples that fail validation.

Docs-only validation for this spec is:

```bash
cargo xtask spec-check --strict
cargo xtask docs-sync --check
git diff --check
```

## Implementation Mapping

Source-of-truth enforcement is owned by:

- `docs/source-of-truth/` for doctrine;
- `docs/templates/` for artifact starting shapes;
- `docs/proposals/USELESSKEY-PROP-0002-source-of-truth-control-plane.md` for
  lane rationale;
- `docs/specs/USELESSKEY-SPEC-0023-source-of-truth-enforcement.md` for this
  contract;
- `policy/doc-artifacts.toml` for document inventory;
- `policy/claim-ledger.toml` and `docs/status/SUPPORT_TIERS.md` for public
  claims and tier mapping;
- `policy/negative-fixtures.toml` and
  `docs/status/negative-fixture-matrix.md` for negative fixture contracts;
- `.uselesskey/goals/active.toml` and `.uselesskey/goals/archive/` for current
  and historical agent lane state;
- `xtask` for checkers, reports, PR-body generation, and closeout generation;
- `.github/workflows/` for advisory source-of-truth CI after local commands
  exist.

## Rollback

Rollback should be per checker or per artifact family:

- revert a checker PR if it produces false positives;
- keep advisory CI non-blocking during burn-in;
- mark artifacts `superseded` instead of deleting stable IDs already referenced
  by docs, ledgers, receipts, or PRs;
- revert generated PR-body or closeout tooling without changing release
  authority.

## CI Proof

Before checker implementation:

```bash
cargo xtask spec-check --strict
cargo xtask docs-sync --check
git diff --check
```

After checker implementation:

```bash
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask check-support-tiers
cargo xtask check-claim-proof-policy
cargo xtask claim-report --check-public-claims
cargo xtask check-negative-fixtures
cargo xtask check-bundle-schemas
```

Advisory hosted CI should report one source-of-truth result if normalized, not
separate required implementation jobs.

## Metrics / Promotion Rule

The control plane starts advisory.

Promote `check-doc-artifacts` and `check-goals` first after clean burn-in on
real PRs. Keep support-tier, negative-fixture, and bundle-schema checks
advisory until they prove low-noise on product changes.

This spec can move from `accepted` to `implemented` only after the checkers,
advisory CI, repo contract report, PR-body generator, and closeout generator
exist and have at least one closeout that records their proof.
