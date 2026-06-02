# Codex Operating Contract

This contract is the Codex-specific form of the source-of-truth operating
model. It turns the active goal, linked plan, proof commands, and merge signal
into the default way to work in `uselesskey-swarm`.

## Startup Contract

For repo work, start from committed source-of-truth artifacts in this order:

1. Read `.rails/index.toml`.
2. If `active_lane` is set, read that Rails lane and select the next `ready`
   work item unless the user explicitly names another item.
3. If `active_lane` is empty, read `last_closed_lane` and
   `.rails/migration-status.md`.
4. Read `.uselesskey/goals/active.toml` when present. Select a `ready` work item
   only when the manifest status is `active`; `archived` means there is no
   current uselesskey goal.
5. Read the work item's linked implementation plan.
6. Read the linked spec for behavior, proof, and non-goals.
7. Read linked ADRs only when the change depends on a durable decision.
8. Read the linked proposal only for why and context.
9. Verify the current branch, worktree status, relevant diffs, and open PR
   state before changing files.

When the active goal and the latest user instruction conflict, stop and report
the conflict. Do not silently choose chat history over committed repo state.

When no active Rails lane or active uselesskey goal exists, use the latest user
instruction plus committed source-of-truth surfaces to make one narrow aligned
improvement. Open a new repo-native goal only when the work should span multiple
PRs.

## PR Contract

Each PR should make one semantic change. The PR should be small enough that a
reviewer can connect it to one work item, one proof set, and one rollback path.

For each PR:

1. Use the active Rails lane or active goal work item as the execution source
   when one exists.
2. Make the scoped change described by the linked plan and spec.
3. Update only affected ledgers, status files, goals, templates, or docs.
4. Run the commands listed on the work item first.
5. Add the relevant source-of-truth checker when the changed artifact family has
   one.
6. Write the PR body from committed repo truth.
7. Wait for `Uselesskey Rust Small Result` before merge.
8. Inspect `Source of Truth Advisory` when it runs. A failure is a repo-contract
   triage signal to fix or explain before merge; it is not one of the
   conditional implementation jobs hidden behind the normalized Rust result.
9. After merge, advance the active goal, closeout, or next ready item when that
   is part of the slice.

Conditional implementation jobs are routing details. Do not treat them as
separate merge gates unless branch protection changes explicitly say so.
Coverage is advisory by default. Inspect it when a PR explicitly requests
coverage evidence, but do not block an otherwise green merge on ordinary
coverage context unless branch protection or the linked work item requires it.

## Command Contract

Do not invent proof commands, lints, policies, workflow names, or merge rules.
Before relying on a command or path, verify it exists in the repo or in the
linked source-of-truth artifacts.

Use the narrow listed proof first. Broaden validation only when the change
touches shared behavior, public claims, policy ledgers, or checker code.

Only parallelize commands when their output roots are independent. The
`external-adoption-smoke` modes all share `target/external-adoption-smoke/` and
must be run sequentially in one checkout because each mode owns the same lock
and receipt files.

For docs-only source-of-truth slices, the normal proof set is:

```bash
cargo xtask docs-sync --check
cargo xtask typos
cargo xtask check-goals
git diff --check
```

Use `cargo xtask spec-check --strict` when proposals, specs, ADRs, plans, or
doc-artifact links change.

## Ledger Contract

Update owned truth in only the artifact that owns it:

- `.uselesskey/goals/active.toml` owns current goal and work-item state.
- `plans/*/implementation-plan.md` owns PR sequence, rollback, promotion, and
  closeout shape.
- `policy/doc-artifacts.toml` owns proposal, spec, ADR, plan, status, and
  policy artifact inventory.
- `policy/claim-ledger.toml` owns public claim to proof mapping.
- `docs/status/SUPPORT_TIERS.md` owns tier posture and support boundaries.
- `policy/negative-fixtures.toml` owns stable negative fixture policy state.

Do not duplicate support-tier truth inside specs. Link to the owning status and
claim artifacts instead.

## Boundary Contract

Do not move release, publish, signing, crates.io, tag, GitHub-release, or
source-sync authority into `uselesskey-swarm`.

Do not make advisory source-of-truth checks blocking before the repo has burned
them in on real PRs and promoted them through a separate CI/policy change.

Do not put generated runtime material, private keys, token values, HMAC secrets,
JWK private members, webhook request bodies, or secret-shaped payloads into
docs, schemas, receipts, audit packets, or PR bodies.

Once active goals exist, do not use chat history as the source of truth for the
next action. Use chat as operator input, then reflect durable state back into
the repo through the active goal, plan, ledger, PR body, or closeout.
