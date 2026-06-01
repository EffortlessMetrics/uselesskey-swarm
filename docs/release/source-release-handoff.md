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

## Current Release-review Packet

For the v0.10.0 source handoff, start with the current swarm packet:

- [`v0.10.0-readiness-record.md`](v0.10.0-readiness-record.md) - release
  review status, proof commands, hosted CI, known non-blockers, publish order,
  rollback, and non-actions;
- [`v0.10.0-public-surface-inventory.md`](v0.10.0-public-surface-inventory.md)
  - release-facing surfaces, commands, proof, risks, and owners;
- [`v0.10.0-version-snippet-reconciliation.md`](v0.10.0-version-snippet-reconciliation.md)
  - current-stable versus release-candidate snippet decisions;
- [`v0.10.0-installed-cli-smoke.md`](v0.10.0-installed-cli-smoke.md) -
  installed CLI doctor, bundle, verify, inspect, and strict audit proof;
- [`v0.10.0-facade-release-smoke.md`](v0.10.0-facade-release-smoke.md) -
  downstream-style facade example proof;
- [`v0.10.0-package-dry-run.md`](v0.10.0-package-dry-run.md) - package
  dry-run and package-content boundary evidence;
- [`../handoffs/2026-05-21-v0-10-0-release-readiness-closeout.md`](../handoffs/2026-05-21-v0-10-0-release-readiness-closeout.md)
  - lane closeout, required evidence, and next safe action.

These files are release-review input from `uselesskey-swarm`. They do not prove
that the synced source checkout is ready until the release operator reruns the
release-prep gates in `EffortlessMetrics/uselesskey`.

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

Before a source release lane starts, refresh a packet from swarm `main`:

```bash
git log --oneline <last-source-sync>..origin/main
gh pr list --state merged --base main --limit 50
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask check-support-tiers
cargo xtask docs-sync --check
cargo xtask typos
cargo xtask claim-report --check-public-claims
cargo xtask contract-packs --check
cargo xtask external-adoption-smoke --path . --format json
cargo xtask external-adoption-smoke --path . --library-examples
cargo xtask external-adoption-smoke --path . --ci-recipes --format json
cargo xtask publish-preflight
cargo xtask publish-check
cargo xtask no-blob
cargo xtask pr
git diff --check
```

Run the `external-adoption-smoke` variants sequentially. Each variant rewrites
`target/external-adoption-smoke/report.md` and
`target/external-adoption-smoke/report.json`; archive or upload the default
path-mode receipt before running the library-example or CI-recipe variant when
the handoff needs multiple records.

Record:

- swarm commit range;
- merged PR list;
- public-claim docs status;
- support-tier and claim-ledger status;
- contract-pack status;
- default path, library-example, and CI-recipe external adoption receipt paths;
- package dry-run status;
- known local validation gaps;
- hosted `Uselesskey Rust Small Result` and `Source of Truth Advisory` status
  for the relevant swarm PRs;
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
8. Keep public install snippets on the current published version until the
   release version exists on crates.io.

## What This Does Not Prove

- It does not prove the source-repo release candidate is ready after sync.
- It does not prove crates.io state.
- It does not prove docs.rs state.
- It does not authorize a version bump, tag, publish, or GitHub release.
- It does not move release authority from source to swarm.

## Reviewer Boundary

Swarm receipts are development evidence. Source release evidence remains the
public release boundary. A later release-prep lane should cite swarm PRs and
receipts, then regenerate release evidence in the source release checkout.
