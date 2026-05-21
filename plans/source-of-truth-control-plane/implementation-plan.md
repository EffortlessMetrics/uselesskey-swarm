+++
id = "USELESSKEY-PLAN-0027"
kind = "plan"
title = "Source-of-truth control plane"
status = "accepted"
owner = "EffortlessMetrics"
created = "2026-05-21"
milestone = "control-plane"
linked_proposal = "USELESSKEY-PROP-0002"
linked_specs = [
  "USELESSKEY-SPEC-0002",
  "USELESSKEY-SPEC-0016",
  "USELESSKEY-SPEC-0017",
  "USELESSKEY-SPEC-0023",
]
linked_adrs = [
  "USELESSKEY-ADR-0002",
  "USELESSKEY-ADR-0003",
]
+++

# Source-of-Truth Control Plane Implementation Plan

Plan id: `USELESSKEY-PLAN-0027`

## Goal

Make the repo-native control plane complete enough that humans and agents can
answer the current work, why it exists, which behavior must hold, which proof
validates it, which public claim it supports, which policy ledger changed, and
what should happen next from repository artifacts instead of chat history.

## Non-goals

- Do not move release, publish, signing, crates.io, GitHub-release, or
  source-sync authority into `uselesskey-swarm`.
- Do not productize the generic proof-stack platform in `tokmd` from this repo.
- Do not make new source-of-truth checks blocking before advisory burn-in.
- Do not add public claims without proof commands and explicit boundaries.
- Do not duplicate support-tier truth inside specs.

## Artifact Map

| Artifact | Role |
| --- | --- |
| `docs/proposals/USELESSKEY-PROP-0002-source-of-truth-control-plane.md` | Why this lane exists. |
| `docs/specs/USELESSKEY-SPEC-0023-source-of-truth-enforcement.md` | Behavior and validation contract. |
| `docs/source-of-truth/` | Human-facing doctrine and agent operating model. |
| `docs/templates/` | Copyable artifact templates. |
| `policy/doc-artifacts.toml` | Proposal/spec/ADR/plan/status/policy ledger. |
| `policy/claim-ledger.toml` | Public claim to proof-command and boundary ledger. |
| `docs/status/SUPPORT_TIERS.md` | Claim and workflow support posture. |
| `.uselesskey/goals/active.toml` | Current agent-operable lane state. |
| `docs/handoffs/` | Closeouts and handoff packets. |

## PR Sequence

| Item | Status | Scope |
| --- | --- | --- |
| Badge queue hygiene | Done | Merge or close generated badge endpoint refresh. |
| Fixture contract payload | Done | Negative fixture ledger, schema docs, material/task-first specs, surface matrix. |
| Source-of-truth doctrine | Done | `docs/source-of-truth/` overview, taxonomy, linking, agent model. |
| Control-plane templates | Done | Proposal, spec, ADR, plan, active goal, closeout, PR body templates. |
| Control-plane proposal | Done | `USELESSKEY-PROP-0002`. |
| Enforcement spec | Done | `USELESSKEY-SPEC-0023`. |
| Doc artifact ledger | Done | `policy/doc-artifacts.toml`. |
| Check doc artifacts | Done | `cargo xtask check-doc-artifacts`. |
| Claim ledger payload | Done | Missing payload claims and public-claims rows. |
| Support-tier map | Done | `docs/status/SUPPORT_TIERS.md`. |
| Check support tiers | Done | `cargo xtask check-support-tiers`. |
| Control-plane implementation plan | Done | This plan and its doc-artifact ledger entry. |
| Active goal manifest | Done | Replace archived lane with source-of-truth control-plane active goal. |
| Check active goals | Done | Add `cargo xtask check-goals`. |
| Agent operating contract | Done | Root `AGENTS.md` and Codex operating contract doc updates. |
| PR and issue templates | Done | GitHub PR and issue templates. |
| Advisory CI | Done | Source-of-truth checks beside the normalized routed result. |
| Report and generators | Active | Repo contract report done; PR body generator and closeout generator remain. |
| Product surface application | Ready | OIDC/JWKS, JWT, webhook, TLS, first-five-minutes docs, CI recipes, release handoff. |

## Per-PR Proof Commands

Use the narrow proof listed by the active goal work item first. Common control
plane proof commands are:

```bash
cargo xtask check-doc-artifacts
cargo xtask check-support-tiers
cargo xtask spec-check --strict
cargo xtask docs-sync --check
cargo xtask typos
git diff --check
```

Code-changing `xtask` slices also run:

```bash
cargo fmt --check -p xtask
cargo test -p xtask <module_name>
cargo xtask pr
```

## Rollback

Each PR must remain revertable as a single semantic slice. If a checker creates
false positives, revert the checker or mark it advisory; do not remove the
underlying source-of-truth docs or ledgers unless their content is wrong.

## Promotion Rules

- Advisory checks may run beside the routed CI lane first.
- Promote `check-doc-artifacts` and `check-goals` only after several clean PRs.
- Keep support-tier, negative-fixture, and bundle-schema checks advisory until
  their failure modes are understood on real changes.
- Preserve `Uselesskey Rust Small Result` as the current required merge signal.

## Closeout Shape

The lane closeout records:

- merged control-plane work;
- proof commands and hosted check evidence;
- public claim and support-tier deltas;
- policy ledgers changed;
- remaining ready work items;
- release risks and source-sync needs;
- the explicit boundary that `EffortlessMetrics/uselesskey` still owns release,
  publish, signing, tags, GitHub releases, and public source sync.
