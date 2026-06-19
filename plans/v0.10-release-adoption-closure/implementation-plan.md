+++
id = "USELESSKEY-PLAN-0030"
kind = "plan"
title = "v0.10 release adoption closure"
status = "accepted"
owner = "EffortlessMetrics"
created = "2026-06-13"
milestone = "v0.10.0"
linked_proposal = "USELESSKEY-PROP-0001"
linked_specs = [
  "USELESSKEY-SPEC-0002",
  "USELESSKEY-SPEC-0006",
  "USELESSKEY-SPEC-0013",
  "USELESSKEY-SPEC-0014",
  "USELESSKEY-SPEC-0017",
  "USELESSKEY-SPEC-0020",
  "USELESSKEY-SPEC-0022",
  "USELESSKEY-SPEC-0024",
]
linked_adrs = [
  "USELESSKEY-ADR-0002",
  "USELESSKEY-ADR-0003",
  "USELESSKEY-ADR-0004",
]
+++

# v0.10 Release Adoption Closure Implementation Plan

Plan id: `USELESSKEY-PLAN-0030`

## Goal

Make v0.10.0 the first release where downstream adoption feels like a product
path instead of repo archaeology.

A cold user should be able to install the CLI, choose a job profile, generate
fixtures, verify and inspect the bundle, audit metadata-only receipts, upload
only the safe audit outputs, and branch on stable `failure_class` values. A
Rust user should be able to add the facade crate with the documented feature
set and run tests without studying leaf crates.

## Product Boundary

The release-facing product is a trust-boundary evidence packet:

```text
generate deterministic fixtures
exercise valid and invalid paths
audit the generated bundle
upload metadata-only receipts
branch on stable failure classes
show reviewers the proof boundary
```

The lane keeps that boundary tight. It does not add new fixture families,
contract packs, provider compatibility claims, production security claims,
scanner approval claims, or CI runner migration work. It does not publish,
tag, sign, push to crates.io, create a GitHub release, or move release
authority from `EffortlessMetrics/uselesskey`.

## Release-Facing User Path

The CLI path to make release-quality is:

```bash
cargo install uselesskey-cli --version 0.10.0 --locked
uselesskey doctor --format json
uselesskey bundle --profile webhook --out target/uselesskey-webhook
uselesskey verify-bundle target/uselesskey-webhook
uselesskey inspect-bundle target/uselesskey-webhook
uselesskey audit-bundle target/uselesskey-webhook --out target/uselesskey-webhook-audit --ci --expect-profile webhook --policy strict
```

The safe upload set is:

```text
target/uselesskey-webhook-audit/bundle-audit.json
target/uselesskey-webhook-audit/bundle-audit.md
```

The Rust facade path to make release-quality is:

```toml
[dev-dependencies]
uselesskey = { version = "0.10.0", default-features = false, features = ["rsa", "jwk", "token"] }
```

then:

```bash
cargo test
```

Before v0.10.0 is published, these `0.10.0` snippets are release-candidate
targets only. Public install and dependency snippets remain on the current
published version until the source repo release bump or post-publish docs step.

## Artifact Map

| Artifact | Role |
| --- | --- |
| `.uselesskey/goals/active.toml` | Active work queue for the release adoption closure lane. |
| `plans/v0.10-release-adoption-closure/README.md` | Short plan-area entry point. |
| `plans/v0.10-release-adoption-closure/implementation-plan.md` | PR sequence and proof map. |
| `docs/release/v0.10-adoption-proof-inventory.md` | Planned release reviewer map of surfaces, jobs, docs, examples, proof, and blockers. |
| `docs/release/v0.10-command-ledger.md` | Planned release-facing command/snippet ledger. |
| `docs/release/v0.10-source-handoff.md` | Planned swarm-to-source handoff for the release candidate. |
| `docs/specs/USELESSKEY-SPEC-0024-v0.10.0-release-readiness.md` | Existing release-readiness behavior and proof contract. |
| `policy/doc-artifacts.toml` | Source-of-truth inventory for this plan. |

## PR Sequence

| PR | Work item | Scope | Primary validation |
| --- | --- | --- | --- |
| 1 | `open-release-adoption-closure-lane` | Open the active goal and plan area for release adoption closure. | `cargo xtask check-goals`; `cargo xtask docs-sync --check`; `cargo xtask typos`; `git diff --check` |
| 2 | `adoption-proof-inventory` | Create `docs/release/v0.10-adoption-proof-inventory.md` with surface, user job, doc, example, proof command, and release blocker rows. | `cargo xtask docs-sync --check`; `cargo xtask typos`; `cargo xtask check-goals`; `git diff --check` |
| 3 | `command-ledger-freeze` | Freeze every release-facing command and snippet with its containing doc and proof owner. | `cargo xtask docs-sync --check`; `cargo xtask typos`; `cargo xtask check-goals`; `git diff --check` |
| 4 | `version-snippet-reconciliation` | Classify each `0.9.1` and `0.10.0` snippet as current published, release-candidate placeholder, version-bump update, or intentionally unchanged. | `rg "0\\.9\\.1|0\\.10\\.0" README.md docs examples crates`; `cargo xtask docs-sync --check`; `git diff --check` |
| 5 | `installed-cli-release-smoke` | Tighten installed-style CLI adoption smoke for local package path before publish and published version after publish. | `cargo xtask external-adoption-smoke --path . --ci-recipes --format json`; `cargo xtask adoption-regression --external` |
| 6 | `facade-published-version-smoke` | Prove the clean downstream Rust facade path in checkout mode and document published-version release-time proof. | `cargo xtask external-adoption-smoke --path . --library-examples` |
| 7 | `receipt-contract-freeze` | Freeze bundle audit JSON contract, failure classes, status/profile fields, schema version, and metadata-only boundaries. | `cargo xtask docs-sync --check`; `cargo xtask typos`; `cargo xtask check-goals`; `git diff --check` |
| 8 | `package-contents-proof` | Validate v0.10 package contents exclude generated runtime payloads and target receipts while including intended docs, schemas, examples, README, and license metadata. | `cargo xtask publish-preflight`; `cargo xtask publish-check`; `cargo xtask no-blob`; `cargo package --workspace --allow-dirty --exclude uselesskey-bdd --exclude uselesskey-bdd-steps --exclude uselesskey-interop-tests --exclude uselesskey-test-support --exclude uselesskey-test-grid --exclude uselesskey-feature-grid --exclude uselesskey-bench --exclude uselesskey-integration-tests --exclude materialize-shape-buildrs-example --exclude materialize-buildrs-example --exclude xtask`; `git diff --check` |
| 9 | `source-release-handoff` | Create `docs/release/v0.10-source-handoff.md` with swarm commit, proof inventory, command ledger, package proof, non-blockers, deferred PRs, non-claims, and rollback. | `cargo xtask docs-sync --check`; `cargo xtask check-goals`; `git diff --check` |
| 10 | `source-repo-release-candidate` | In `EffortlessMetrics/uselesskey`, prepare the real v0.10.0 candidate: versions, internal deps, README/docs snippets, changelog, and release notes. | source-repo build, test, external adoption, publish preflight, publish check, no-blob, docs-sync, and diff checks |
| 11 | `final-release-dry-run-record` | Record exact dry-run evidence: SHA, toolchain, package dry-run, file list, installed CLI smoke, facade smoke, receipt smoke, claim/support checks, main CI, and known non-blockers. | `cargo xtask check-doc-artifacts`; `cargo xtask check-goals`; `git diff --check` |
| 12 | `release-execution` | After explicit approval only, publish crates in order, tag v0.10.0, push the tag, and verify published install and external adoption. | publish and post-publish proof commands from the release approval |

## Command Ledger Seeds

PR 3 owns the complete ledger. It must at least cover:

```bash
cargo install uselesskey-cli --version 0.10.0 --locked
uselesskey doctor --format json
uselesskey bundle --profile webhook --out target/uselesskey-webhook
uselesskey verify-bundle target/uselesskey-webhook
uselesskey inspect-bundle target/uselesskey-webhook
uselesskey audit-bundle target/uselesskey-webhook --out target/uselesskey-webhook-audit --ci --expect-profile webhook --policy strict
cargo xtask external-adoption-smoke --path . --ci-recipes --format json
cargo xtask adoption-regression --external
cargo xtask publish-preflight
cargo xtask publish-check
```

Each row must name the doc containing the command, the smoke or release check
that proves it, and whether it is available before publish or only after the
v0.10.0 source release exists.

## Receipt Contract Rules

PR 7 owns the detailed contract, but this lane starts with these release rules:

- automation branches on `bundle-audit.json`, not Markdown or log prose;
- `checks[].failure_class`, status, profile, schema version, and top-level
  receipt fields are the release-facing contract;
- Markdown is a reviewer aid, not the automation contract;
- generated runtime fixture payloads are unsafe to upload;
- the metadata-only upload set is limited to the audit JSON and Markdown
  files unless a later release proof explicitly expands it.

## Proof Order

Use the active work item's commands first. Add broader proof only when the
slice changes shared behavior, generated artifacts, packaging, or release
state.

The lane-opening proof is:

```bash
cargo xtask check-doc-artifacts
cargo xtask check-goals
cargo xtask docs-sync --check
cargo xtask typos
git diff --check
```

The source release candidate proof runs in `EffortlessMetrics/uselesskey`, not
as an implicit swarm publish:

```bash
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-features --locked
cargo xtask external-adoption-smoke --path . --format json
cargo xtask adoption-regression --external
cargo xtask publish-preflight
cargo xtask publish-check
cargo xtask no-blob
cargo xtask docs-sync --check
git diff --check
```

Release execution is blocked until explicit approval.

## Stop Conditions

Stop and report instead of broadening the lane when:

- a needed release-facing command does not exist;
- a snippet cannot be honestly proven before publish;
- `0.10.0` appears as an installable public version before it exists on
  crates.io;
- an audit receipt would require uploading generated fixture material;
- package proof finds target receipts or runtime payloads in publish contents;
- CI capacity becomes the only blocker and the documented routed fallback has
  not been tried;
- a change would move release authority into swarm.

## Rollback

Revert the PR that introduced the incorrect artifact or work item, then return
the affected active-goal item to `ready` or `blocked` with a concrete reason.
If the lane itself is premature, restore the prior archived active-goal
manifest and remove this plan area plus its doc-artifact ledger row.

After publication, rollback is a source-repo release decision. Prefer a
forward-fix patch release or yanked package only with explicit maintainer
approval and recorded evidence.
