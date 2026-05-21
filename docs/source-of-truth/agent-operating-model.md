# Agent Operating Model

Agents should be able to enter the repository cold and find the next safe
change from repo state.

## Startup

Use this order:

1. Read `.uselesskey/goals/active.toml`.
2. Choose the next `ready` work item unless the user explicitly names a
   different item.
3. Read the linked implementation plan.
4. Read the linked spec for behavior and proof.
5. Read linked ADRs only when the change depends on a durable decision.
6. Read the linked proposal only for why and context.
7. Inspect the current branch, status, open PRs, and relevant diffs.

If no active goal exists, use the newest explicit user instruction plus the
accepted source-of-truth docs, then create or update the repo-native goal in a
dedicated PR before relying on it for later work.

## PR Loop

Each PR should do one semantic job:

```text
read active goal
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
