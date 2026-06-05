# Repo style

This repo is operated as an evidence machine.

Rust and `xtask` are the default construction material. Non-Rust files,
unsafe, panic paths, lint suppressions, generated files, workflow behavior,
process/network access, expensive CI lanes, and release claims must be owned
and receipted.

Static evidence runs first:

- source-exception policy for visible retained exceptions;
- `ripr` for static mutation-exposure analysis when available;
- `unsafe-review` for unsafe-contract review when unsafe seams exist;
- rustc and Clippy for code-shape policy.

Runtime evidence runs where it pays:

- focused tests on PRs;
- targeted mutation for risk PRs;
- broader mutation, Miri, fuzzing, and coverage on nightly, main, release, or
  explicitly labelled lanes.

CI is designed for proof per Linux-equivalent minute. Default PRs are cheap,
deterministic, and high-signal. Deep validation is preserved, but routed by
risk pack, label, main, nightly, or release.

Agents work one review-fast PR at a time. Review-fast does not mean tiny; it
means coherent seam, nearby proof, efficient verification, and honest claim
boundary. Do not broaden scope to satisfy CI. Do not add invisible exceptions.

## Tool roles

`xtask` is the repo control plane. It should wrap upstream tools, aggregate
receipts, enforce repo-local glue, and provide stable commands for humans and
agents. It should not reimplement upstream tools when a standard Rust tool can
own the signal.

| Tool or lane | Role |
| --- | --- |
| Source-exception policy | Durable ledger for visible source exceptions: panic-family calls, lint suppressions, generated files, scripts, workflow surfaces, non-Rust files, and other retained policy exceptions. |
| `ripr` | Static mutation-exposure analysis: cheap PR-time weak-oracle signal before runtime mutation. |
| `unsafe-review` | Unsafe-contract reviewability: safety contract, guard, test reach, and witness route for unsafe seams. |
| `cargo-mutants` | Runtime mutation backstop where static exposure or risk routing says it pays. |
| Miri | Concrete undefined-behavior execution witness for relevant unsafe or layout-sensitive surfaces. |
| Codecov / coverage | Execution-surface telemetry, not a correctness proof. |
| `xtask` | Orchestration, receipts, reports, CI planning, release readiness, and repo-local policy glue. |

## Exception posture

There should be no invisible source exceptions. Retained exceptions need an
owner, reason, classification, and evidence route. In this repository, current
source-policy ledgers include `policy/no-panic-allowlist.toml` for panic-family
exceptions and `policy/non-rust-allowlist.toml` for tracked non-Rust surfaces;
future consolidation may move more source-exception ownership to a single
`policy/allow.toml`-style ledger when the checker stack is ready.

## Review-fast PRs

A review-fast PR has:

- one behavior, seam, or policy slice;
- no unrelated cleanup;
- local proof commands that match the changed surface;
- updated receipts or ledgers when the change creates policy-visible drift;
- a claim boundary that states what the PR proves and what remains out of
  scope.

The good path should be easiest:

```text
change code
run one repo command
see exception diffs
see weak-oracle gaps
see unsafe review cards
add focused proof
keep receipts
merge when green
```
