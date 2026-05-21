+++
id = "USELESSKEY-PLAN-0000"
kind = "plan"
title = "Short implementation plan title"
status = "proposed"
owner = "EffortlessMetrics"
created = "YYYY-MM-DD"
milestone = "v0.0.0"
linked_proposal = "USELESSKEY-PROP-0000"
linked_specs = ["USELESSKEY-SPEC-0000"]
linked_adrs = []
linked_plan = ""
support_tier_impact = []
policy_impact = []
+++

# USELESSKEY-PLAN-0000: Short Implementation Plan Title

## Goal

What this plan completes.

## Non-goals

What must not be mixed into this lane.

## Artifact Map

| Artifact | Role | Owner |
| --- | --- | --- |
| `docs/specs/USELESSKEY-SPEC-0000-example.md` | Behavior contract | repo-infra |
| `policy/example.toml` | Policy ledger, if needed | repo-infra |

## PR Sequence

| Item | PR title | Files | Proof |
| --- | --- | --- | --- |
| `first-work-item` | `docs: example` | `docs/example.md` | `cargo xtask docs-sync --check`; `git diff --check` |

## Per-PR Proof Commands

```bash
cargo xtask docs-sync --check
cargo xtask typos
git diff --check
```

## Support-Tier Impact

Which support tiers change, or `none`.

## Policy Impact

Which policy ledgers change, or `none`.

## Claim Boundary

What the plan supports and what it does not prove.

## Rollback

How to revert one PR and how to stop the lane safely.

## Promotion Rules

What moves advisory checks, claims, or surfaces to stronger status.

## Closeout Shape

What the closeout or handoff must record.
