# Source Release Handoff

Use this note when work has landed in `EffortlessMetrics/uselesskey-swarm` and a
future release operator needs to decide how to carry it back to the source repo.

Current rule:

```text
uselesskey-swarm is the normal development repo.
EffortlessMetrics/uselesskey remains the release, publish, signing, tag, and
public-source boundary until a separate release-authority lane moves it.
```

This note is not release prep. It is a handoff checklist for later.

## What Stays in Swarm

Normal development PRs should continue to land in `uselesskey-swarm`:

- docs and spec work;
- fixture confidence coverage;
- installed CLI workflow polish;
- `xtask` proof improvements;
- dependency hygiene that does not publish or tag;
- examples and clean-project adoption proof.

Required PR check:

```text
Uselesskey Rust Small Result
```

Do not require conditional implementation jobs as branch-protection checks.

## What Stays in Source

Keep these in `EffortlessMetrics/uselesskey` until explicitly assigned:

- crates.io publish;
- GitHub release creation;
- release tags;
- signing;
- publish retry;
- release workflow authority;
- docs.rs and post-release audit authority.

Do not move `release.yml`, `publish-retry.yml`, signing secrets, or crates.io
release authority as part of a routine swarm PR.

## Handoff Packet

Before a source release lane starts, prepare a packet from swarm `main`:

```bash
git log --oneline <last-source-sync>..origin/main
gh pr list --state merged --base main --limit 50
cargo xtask claim-report --check-public-claims
cargo xtask external-adoption-smoke --path . --format json
cargo xtask adoption-regression --external
cargo xtask pr
git diff --check
```

Record:

- swarm commit range;
- merged PR list;
- public-claim docs status;
- external adoption receipt paths;
- adoption regression receipt paths;
- known local validation gaps;
- explicit statement that no publish/tag/signing step ran in swarm.

## Source Sync Rules

When a release operator is ready to sync source:

1. Start from a clean source repo checkout.
2. Fetch `EffortlessMetrics/uselesskey-swarm`.
3. Review the swarm commit range and merged PR list.
4. Merge or cherry-pick only the intended development commits.
5. Keep source release workflows authoritative unless the lane explicitly moves
   them.
6. Run source release-prep gates in source, not only swarm.
7. Do not tag, publish, or create a GitHub Release without explicit release
   execution approval.

## What This Does Not Prove

- It does not prove a release candidate is ready.
- It does not prove crates.io state.
- It does not prove docs.rs state.
- It does not authorize a version bump, tag, publish, or GitHub release.
- It does not move release authority from source to swarm.

## Reviewer Boundary

Swarm receipts are development evidence. Source release evidence remains the
public release boundary. A later release-prep lane should cite swarm PRs and
receipts, then regenerate release evidence in the source release checkout.
