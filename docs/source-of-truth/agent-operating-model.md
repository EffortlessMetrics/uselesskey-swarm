# Agent Operating Model

Agents should be able to enter the repository cold and find the next safe
change from repo state.

## Startup

Use this order:

1. Read `.rails/index.toml`.
2. If `active_lane` is set, read that `.rails/lanes/*` lane and choose the next
   `ready` item unless the user explicitly names a different item.
3. If `active_lane` is empty, read `last_closed_lane` and
   `.rails/migration-status.md` before looking for other active state.
4. Read `.uselesskey/goals/active.toml` when present. Choose the next `ready`
   work item only when the manifest status is `active`.
5. Treat `status = "archived"` in `.uselesskey/goals/active.toml` as no current
   uselesskey goal.
6. Read the linked implementation plan.
7. Read the linked spec for behavior and proof.
8. Read linked ADRs only when the change depends on a durable decision.
9. Read the linked proposal only for why and context.
10. Inspect the current branch, status, open PRs, and relevant diffs.

If no active Rails lane or active uselesskey goal exists, use the newest
explicit user instruction plus the accepted source-of-truth docs to choose the
smallest aligned PR-sized improvement. Create or update a repo-native goal in a
dedicated PR only when the next lane needs more than one PR.

## PR Loop

Each PR should do one semantic job:

```text
read .rails/index.toml
  -> read active lane or archived-goal state
    -> read plan item
      -> make one scoped change
        -> update affected ledgers only
          -> run listed proof commands
            -> write PR body from repo truth
              -> merge when required checks are green
                -> update closeout or next work item when appropriate
```

The PR body should include:

- summary;
- linked proposal, spec, ADR, and plan item;
- scope and non-goals;
- support-tier impact;
- policy impact;
- proof commands and results;
- claim boundary;
- rollback.

## Validation

Run the commands listed on the work item first. For source-of-truth docs-only
changes, the default local proof is:

```bash
cargo xtask docs-sync --check
cargo xtask typos
git diff --check
```

When source-of-truth enforcement exists, add the relevant checker command rather
than inventing ad hoc validation.

## Ledger Updates

Update ledgers only when their owned truth changes:

- `policy/claim-ledger.toml` for public claim or proof mapping changes;
- `docs/status/SUPPORT_TIERS.md` for tier or boundary changes;
- `policy/negative-fixtures.toml` for negative fixture contract changes;
- `policy/doc-artifacts.toml` for source-of-truth artifact inventory changes;
- `.uselesskey/goals/active.toml` for current work state changes.

Do not update ledgers just to mention a PR.

## Stop Conditions

Stop and report clearly when:

- the active goal and user instruction conflict;
- the linked spec is missing or contradicts the requested change;
- required proof commands do not exist;
- generated runtime material would need to enter docs, schemas, receipts, or
  audit packets;
- the change would move release, publish, signing, tag, crates.io, GitHub
  release, or source-sync authority into `uselesskey-swarm`.

Prefer a small corrective PR over broad cleanup when the graph is stale.
