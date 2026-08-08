# Test Evidence Lanes

`uselesskey` is a fixture platform for secret-shaped test systems. CI evidence
should be organized around that promise: deterministic fixtures, scanner-safe
bundle defaults, realistic negative cases, and adapter contracts should be
backed by the right proof at the right time.

The goal is not simply faster CI. The goal is to make each lane answer a clear
question without making every pull request pay release-candidate cost.

## Signal Boundaries

Different tools answer different questions:

| Signal | Question answered | Boundary |
| --- | --- | --- |
| Coverage | Did tests execute this surface? | Does not prove assertions would detect wrong behavior. |
| `ripr` | Does changed behavior appear to have a meaningful test oracle? | Static exposure signal only. |
| Release evidence | Can the shipped public promises be proven together? | Release-candidate proof, not day-to-day PR feedback. |

`ripr` is advisory. Use it to route focused test-writing toward behavior that
looks weakly exposed; it does not prove that behavior is correct.

## Lane 1: PR Fast Evidence

Runs on every pull request.

Purpose:

- catch build, lint, docs, support metadata, scanner-risk, and public-surface
  drift quickly;
- show whether changed behavior appears to reach a meaningful test oracle;
- keep ordinary review feedback fast enough to act on.

Typical signals:

- formatting and Clippy;
- impact-scoped tests;
- `cargo xtask docs-sync --check`;
- `cargo xtask public-surface` when topology or support metadata changes;
- `cargo xtask publish-preflight --allow-dirty` when package metadata changes;
- `cargo xtask examples-smoke` when examples or user recipes change;
- `cargo xtask no-blob` when fixture, docs, bundle, or example outputs change;
- `cargo xtask ripr-pr` for advisory `ripr` PR exposure evidence;
- `cargo xtask ripr-review-comments` for advisory line-placeable review
  guidance;
- `git diff --check`.

In the swarm development repository, routed PR CI exposes one required contract
check: `Uselesskey Rust Small Result`. The route, hosted fallback, and concrete
runner jobs are implementation details; branch protection should require the
normalized result check only. See
[`routed-rust-workflow.md`](routed-rust-workflow.md) for the hosted fallback
label and workflow-change routing rules.

Blocking posture:

- normal build, lint, docs, public-surface, publish, no-blob, and example
  failures block when their affected surface is in scope;
- `ripr` should start advisory and become blocking only for severe exposure
  gaps after the baseline is stable.

## Performance Evidence

Performance evidence runs in the PR/main CI perf job and in the dedicated
scheduled/manual workflow at `.github/workflows/performance.yml`.

Run the local performance evidence command with:

```bash
cargo xtask perf --compare
```

`cargo xtask perf` writes the machine-readable benchmark report to
`target/xtask/perf/latest.json`. The scheduled/manual workflow also writes a
human-readable `target/xtask/perf/latest.md` summary, uploads
`target/xtask/perf/` as the `performance-evidence` artifact, and compares the
latest report against `docs/metadata/perf-baselines.json` by default.

This lane is evidence for fixture-generation cost trends. It is runner-sensitive
and does not prove cryptographic correctness, scanner safety, or deterministic
identity by itself.

## Lane 2: Release Evidence

Runs for release branches, release candidates, or tag candidates.

Purpose:

- prove the shipped public fixture platform, not just the changed diff;
- give users and maintainers a linkable evidence package for deterministic
  fixtures, negative cases, scanner-safe bundles, adapters, and package proof.

Signals:

- repository-level `ripr` exposure summary;
- `cargo xtask public-surface`;
- package proof and publish dry-runs;
- `cargo xtask bundle-proof --profile scanner-safe` for bundle generation,
  `verify-bundle`, `inspect-bundle`, export handoff, and no-blob proof;
- `cargo xtask bundle-proof --profile oidc` for OIDC contract-pack contents,
  owner-crate JWK/token tests, verifier proof, inspection, and no-blob proof;
- scanner-safe `no-blob` proof;
- docs/examples smoke;
- adapter matrix checks;
- receipt drift checks.
- scheduled/manual performance evidence.

Release evidence should produce durable Markdown and JSON artifacts that can be
linked from release notes. Plan or run the release evidence lane with:

```bash
cargo xtask release-evidence --version 0.7.0 --out target/release-evidence --summary
```

The v0.7.0 release-candidate checklist starts in
[`docs/release/evidence-matrix-v0.7.0.md`](../release/evidence-matrix-v0.7.0.md).

## Change-Type Routing

| Change type | Fast PR evidence | Release role |
| --- | --- | --- |
| Docs-only policy or roadmap | Docs sync, typos, link/diff checks | Release docs unaffected. |
| User examples or bundle recipes | Examples smoke, no-blob, docs sync | Release examples proof. |
| Public-surface metadata or crate topology | Public-surface, publish-preflight, docs sync | Release matrix proof. |
| Core derivation/hash/identity | Fast gates plus deterministic regression tests | Release-blocking evidence. |
| Negative fixture semantics | Owner tests plus `ripr` exposure | Failure-atlas proof. |
| Bundle manifest/verifier/receipts | CLI tests, bundle verify, no-blob | Bundle release receipts. |
| Adapter conversion behavior | Adapter tests and examples | Adapter contract proof. |
| Compatibility shim only | Import, no-default, package, metadata proof | Release shim policy. |

## Author Workflow

For an ordinary PR:

1. Run the fast local gates for the touched surface.
2. Run `cargo xtask ripr-pr` and read `target/ripr/pr/summary.md`.
3. Add focused tests when changed behavior is weakly exposed.

For a high-risk PR:

1. Run the fast local gates.
2. Run `cargo xtask ripr-pr` to find missing or weak test oracles.
3. Run `cargo xtask impacted-evidence --base origin/main` to identify the
   evidence owner crates.
4. Add focused tests for severe exposure gaps.
5. Record the focused tests added in the PR body.

`cargo xtask ripr-pr` treats `ripr` as an external tool and writes advisory
artifacts under `target/ripr/pr/`:

- `repo-exposure.json`;
- `repo-exposure.md`;
- `summary.md`;
- `review.md`.

`cargo xtask ripr-pr-summary` regenerates `summary.md` from the available
machine-readable PR artifacts after `ripr` review guidance and impacted
evidence have been produced. Missing data is represented explicitly rather than
filled in from prose.

Pull request CI uploads that directory as the `ripr-pr` artifact.

`cargo xtask ripr-review-comments` runs `ripr review-comments` with explicit
root, base, head, and output paths. It writes advisory review guidance under
`target/ripr/review/`:

- `comments.json`;
- `comments.md`.

Pull request CI uploads that directory as the `ripr-review` artifact, appends
`comments.md` to the step summary, and emits non-blocking warning annotations
from `comments[]` only. `summary_only[]` stays in the summary and artifact;
inline PR comments are disabled by default.

If `ripr` is not installed, the command writes skipped artifacts with a clear
reason and exits successfully. Other `ripr` runtime failures should fail the
command so the tool integration does not silently mask bad evidence.

`cargo xtask impacted-evidence --base origin/main` writes
`target/xtask/impacted-evidence/latest.json` and prints the same JSON summary to
stdout. The summary maps changed paths to public owner crates and explains why a
path is high risk. When `target/ripr/pr/repo-exposure.json` exists, the same
report includes a `ripr` routing section that classifies severe exposure gaps,
maps them to public owner crate surfaces where possible, and lists the
focused-test actions expected from the PR author.

Pull request CI runs `cargo xtask impacted-evidence` after `ripr-pr` and uploads
`target/xtask/impacted-evidence/` as the `impacted-evidence` artifact.

The PR workflow also writes a GitHub step summary after the evidence-producing
steps. The summary lists fast-gate statuses, RIPR counts and severe-gap routing,
impacted owner crates, suggested actions, and the uploaded artifact names. It is a review aid; the underlying gates and artifacts
remain the source of truth.
