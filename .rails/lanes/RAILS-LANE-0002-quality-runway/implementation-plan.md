# Quality Runway and Maintenance Queue Implementation Plan

ID: RAILS-PLAN-0001  
Kind: implementation-plan  
Title: Quality runway and maintenance queue  
Status: active  
Owner: EffortlessMetrics  
Created: 2026-08-12  
Linked proposal: RAILS-PROP-0001  
Linked specs: USELESSKEY-SPEC-0005, USELESSKEY-SPEC-0023  
Linked ADRs: USELESSKEY-ADR-0003  
Linked lane: RAILS-LANE-0002  
Support-tier impact: none  
Policy impact: issue #633 activation slice only

## Objective

Turn the current reviewed and queued quality work into a serial, evidence-led
campaign without treating PR checks as main proof or widening any bounded seam.

## Non-goals

- Do not merge while `cargo xtask check-merge-queue` reports `hold`,
  `investigate`, or `unknown` for ordinary work.
- Do not mix workflow activation with Rust checker implementation.
- Do not reset no-panic baselines or restore retired mutation tooling.
- Do not adopt major dependencies without a maintainer migration issue and
  public-type, encoding, determinism, and negative-path review where relevant.
- Do not publish, tag, sign, release, deploy, or move source-sync authority.

## Artifact map

- Rails lane state: `.rails/lanes/RAILS-LANE-0002-quality-runway/lane.toml`
- Main-proof incident: issue #585
- Security repair: issue #636 and PR #637
- CI policy seam: issue #633 and PR #639, followed by PR B
- AWS-LC cfg repair: issue #638 and PR #640
- Mutation-doc reconciliation: PR #631
- Retired mutation source: merged PR #618
- Dependency execution and migration links: recorded on the lane work items

## PR-sized sequence

1. Hold ordinary merges until issue #585's newest main-full proof is healthy and
   `cargo xtask check-merge-queue` reports `pass`.
2. Squash PR #637 to restore the `quinn-proto` advisory floor; close issue #636
   only after the merge is verified.
3. Squash PR #639 as issue #633 PR A. Keep issue #633 open.
4. Build issue #633 PR B from fresh post-#639 main and atomically add the
   checker to the Source of Truth workflow and policy-owned command list.
5. Squash PR #640, then close issue #638 after verifying the merge.
6. Freshness-review and squash PR #631 after the runway work is true on main.
7. Remove only stale no-panic baseline entries whose selectors disappeared in
   PR #618; inspect generated proposal churn before committing ledger changes.
8. Handle patch dependency PRs serially: #620, #623, #625, #627, then #630.
   Keep #630 after PR #640 because they share the AWS-LC collision family.
9. Qualify major migrations through maintainer issues: #632 for closed PR #621,
   #622, #624, #634 for closed PR #629, #635 for closed PR #626, and #628
   after a builder-ready migration issue exists.
10. Write a Rails closeout, clear `active_lane`, preserve real follow-ups, and
    clean lane-created branches and worktrees.

After every merge, synchronize main, verify the newest main proof, update the
lane status when appropriate, and remove only artifacts created by that lane.
Remote branches are repository-configured to delete after merge; local
worktrees and branches still require validated cleanup.

## Proof commands

Control-plane establishment:

- `cargo xtask docs-sync --check`
- `cargo xtask typos`
- `cargo xtask spec-check --strict`
- `cargo xtask check-goals`
- TOML parse of `.rails/index.toml` and the lane manifest
- `git diff --check`

Queue promotion and merge:

- `cargo xtask check-merge-queue`
- Work-item commands recorded in `lane.toml`
- Current `Uselesskey Rust Small Result`
- Inspected `Source of Truth Advisory`

## Claim boundary

The plan proves that the selected queue has a durable order and that each slice
has a proportional proof path. It does not prove a queued change before that
proof runs, convert hosted PR success into main or release proof, or expand any
linked issue's behavior claim.

## Rollback

Revert or close the control-plane artifacts together if the campaign is
abandoned. Revert individual implementation PRs through their own rollback
paths; do not rewrite unrelated history, reopen superseded bot PRs, or weaken
checks to preserve the sequence.
