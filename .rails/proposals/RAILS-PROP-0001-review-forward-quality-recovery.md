# RAILS-PROP-0001: Review-Forward Quality Recovery

ID: RAILS-PROP-0001  
Kind: proposal  
Title: Review-forward quality recovery  
Status: accepted  
Owner: EffortlessMetrics  
Created: 2026-08-12  
Target milestone: quality-runway  
Linked specs: USELESSKEY-SPEC-0005, USELESSKEY-SPEC-0023  
Linked ADRs: USELESSKEY-ADR-0003  
Linked lanes: RAILS-LANE-0002  
Support-tier impact: none  
Policy impact: CI check-policy synchronization only through issue #633

## Problem

The repository has a review-complete maintenance queue but no active committed
lane. Main-full proof is held by the capacity incident in issue #585, several
green drafts must land in a deliberate order, issue #633 requires two atomic PR
slices, and later no-panic and dependency work must not race those changes.
Keeping that sequence only in chat would make merge readiness, dependencies,
and proof boundaries stale or ambiguous for the next agent.

## Users and surfaces

Maintainers and agents need one first-read queue that distinguishes reviewed
code from merge authorization, preserves the normalized and Source of Truth
checks as separate evidence, and points to GitHub for live issue and PR state.
Reviewers need each slice to retain its own proof, non-goals, rollback, and
cleanup instead of being folded into a broad quality PR.

## Success criteria

- The newest main-full proof is healthy before ordinary merges resume.
- Reviewed drafts land one at a time in the lane order with fresh queue proof.
- CI check-policy implementation and activation remain separate PRs.
- Stale no-panic baseline entries are removed narrowly without resetting the
  baseline or absorbing unrelated churn.
- Patch dependency PRs are handled serially, and major migrations start from
  builder-ready maintainer issues rather than unreviewed bot diffs.
- Each merged slice closes only its linked issue, cleans its lane-created
  branch and worktree, and leaves remaining work accurately queued.
- The lane receives a proof-backed closeout when all selected work is landed,
  superseded, or intentionally deferred.

## Proposed shape

Use RAILS-LANE-0002 as the authoritative sequence and implementation plan.
GitHub issues and PRs remain the live execution board, while the lane records
their ordering, dependencies, bounded proof, and claim limits. No work item is
marked `ready` while main proof is held; reviewed drafts remain `blocked` or
`queued` until their dependencies are true.

## Alternatives considered

A GitHub umbrella issue would duplicate state below the repository's first-read
Rails index and would not satisfy the repo-native multi-PR goal contract. A
single broad implementation PR would mix security, checker, docs, no-panic,
and dependency collision families and weaken attribution of regressions.

## Specs to create or update

No new behavior spec is required. USELESSKEY-SPEC-0005 owns agent lane state,
USELESSKEY-SPEC-0023 owns CI source-of-truth enforcement, and linked issues own
the acceptance criteria for their bounded implementation slices.

## Architecture decisions needed

None. USELESSKEY-ADR-0003 already establishes repo-owned active goals as the
agent control plane.

## Implementation campaign shape

Restore the shared merge runway first, land the security advisory floor, finish
the two-part CI policy seam, land the AWS-LC cfg repair and mutation-doc
reconciliation, remove stale no-panic baseline rows, then process patch and
major dependency queues serially. The implementation plan carries the exact
order and promotion conditions.

## Evidence plan

Every merge requires the work item's local proof, a current normalized
`Uselesskey Rust Small Result`, inspection of `Source of Truth Advisory`, and a
passing advisory merge-queue decision before the next merge. Evidence proves
only the changed seam on the checked revision.

## Claim boundary

This proposal establishes durable queue ownership, ordering, and evidence
boundaries. It does not make a blocked draft mergeable, prove runner capacity,
promote advisory checks, establish release readiness, or prove cryptographic
correctness or compatibility for unbuilt dependency migrations.

## Risks

- Live GitHub state can outpace committed lane state; refresh status after each
  merge or material disposition.
- Dependency bot branches can become stale after a lockfile merge; update and
  re-prove them one at a time.
- Broad baseline regeneration can hide unrelated no-panic churn; inspect a
  generated proposal before editing the committed ledger.

## Non-goals

- No release, publish, tag, signing, deployment, crates.io, or source sync.
- No direct branch-protection changes or CI role promotion.
- No broad no-panic baseline reset or mutation-tooling restoration.
- No combining behavior keepers with dependency updates in one PR.
- No major dependency adoption directly from an unqualified bot branch.
- No claim that green PR checks equal main-full or release proof.

## Rollback / exit

If the lane becomes inaccurate, stop selecting work from it, correct or close
the lane in a source-truth-only PR, and leave live implementation PRs untouched.
Do not recover by marking blocked work ready or by weakening its proof.

## Exit criteria

Close the lane only after selected items are merged, superseded, or explicitly
deferred; linked issues are correctly disposed; main is synchronized; lane
worktrees and branches are cleaned; and a Rails closeout records proof and
remaining work.
