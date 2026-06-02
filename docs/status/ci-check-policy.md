# CI Check Policy

Source-of-truth artifact id: USELESSKEY-STATUS-ci-check-policy

`policy/ci-checks.toml` owns the machine-readable check role map. This page is
the human index for maintainers and agents deciding whether a hosted signal is a
merge gate, a source-of-truth triage signal, an advisory lane, or route evidence.

## Roles

| Role | Meaning | Normal action |
| --- | --- | --- |
| `required` | Branch-protection merge signal. | Wait for success before merge. |
| `triage-required` | Repo-contract signal that should be inspected before merge. | Fix or explain source-of-truth drift. |
| `advisory-by-default` | Useful evidence that does not block ordinary PRs. | Inspect when the PR asks for that lane. |
| `main-branch-proof` | Full proof for the latest `main` commit. | Preserve the newest live run instead of casually superseding it. |
| `conditional-route` | Implementation detail selected by the router. | Read it as route evidence; use the normalized result for merge meaning. |
| `merge-blocker` | Explicit route failure. | Fix routing or use an allowed fallback policy. |

## Current Policy

| Check | Role | Workflow | Boundary |
| --- | --- | --- | --- |
| `Uselesskey Rust Small Result` | `required` | `EM CI Routed Rust` | Normalized merge signal; conditional runner jobs are route details. |
| `Source of Truth Advisory` | `triage-required` | `Source of Truth` | Repo-contract signal; failures need triage before merge. |
| `Coverage` | `advisory-by-default` | `Coverage` | Execution signal only; not an ordinary merge gate. |
| `Uselesskey Main Full Gate` | `main-branch-proof` | `EM CI Routed Rust` | Full `main` proof; the newest live run is the branch proof. |
| `Uselesskey Rust Small on CPX42` | `conditional-route` | `EM CI Routed Rust` | Concrete self-hosted implementation route. |
| `Uselesskey Rust Small on CX43` | `conditional-route` | `EM CI Routed Rust` | Concrete self-hosted implementation route. |
| `Uselesskey Rust Small on CX53` | `conditional-route` | `EM CI Routed Rust` | Concrete self-hosted implementation route. |
| `Uselesskey Rust Small on GitHub Hosted` | `conditional-route` | `EM CI Routed Rust` | Hosted fallback route, not release authority. |
| `Uselesskey Docs/Policy Light on GitHub Hosted` | `conditional-route` | `EM CI Routed Rust` | Metadata-only route; not product behavior proof. |
| `Uselesskey Workflow Validation on GitHub Hosted` | `conditional-route` | `EM CI Routed Rust` | Workflow-shape evidence. |
| `Uselesskey Route Failed` | `merge-blocker` | `EM CI Routed Rust` | Router failure that needs policy or route repair. |

## Route Receipt

`EM CI Routed Rust` uploads a `proof-route` artifact containing
`target/source-of-truth/proof-route.json`. Use it to review the selected target,
router reason, changed files and surfaces, merge blockers, skipped-by-policy
notes, and local reproduction commands. The receipt is evidence; it does not
change branch protection or make advisory checks blocking.

The `route_reasons[]` entries explain why each changed path was classified as
workflow, docs/policy metadata, or Rust proof. Issue templates, PR templates,
Rails lanes, goal manifests, policy ledgers, and docs should name their light
metadata reason directly. Unknown or implementation paths must route to Rust
proof rather than appearing as skipped work.

## Main Full Gate Receipt

`Uselesskey Main Full Gate` uploads a `main-full-gate-receipt` artifact
containing `target/source-of-truth/main-full-gate-receipt.json`. Use it to
review hosted full-gate start and completion timestamps, elapsed seconds,
`xtask ci` result, exit code, heartbeat evidence, and the relationship to
`target/xtask/receipt.json`. The receipt is evidence for the latest `main`
commit; it does not replace the normalized `Uselesskey Rust Small Result`.

`cargo xtask check-merge-queue` is the local advisory companion to this policy.
It reads the newest main `EM CI Routed Rust` push run, writes
`target/source-of-truth/merge-queue-check.json`, and reports whether ordinary
merges should proceed, hold for an unresolved main full gate, or investigate a
failed or missing main proof. It is not a branch-protection check unless a
caller deliberately runs it with `--strict`.

## Boundary

This policy does not move release, publish, signing, tag, GitHub release,
crates.io, or source-sync authority into `uselesskey-swarm`.

This policy records check meaning. It does not make advisory checks blocking and
does not replace branch protection. If branch protection changes, update
`policy/ci-checks.toml` and this page in the same PR.
