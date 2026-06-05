# ripr static mutation-exposure lane

`ripr` is static mutation-exposure analysis. It catches the same class of
findings mutation testing catches — weak test or oracle exposure — but earlier
and cheaper because it is static and PR-time.

It does not run mutants, report killed or survived outcomes, prove correctness,
or replace runtime mutation testing. Mutation testing remains the slower runtime
backstop.

## Role split

| Tool | Question |
| --- | --- |
| `ripr` | Did the changed code expose behavior whose tests or oracles look weak? |
| `cargo-mutants` | Did concrete mutants survive the selected runtime test suite? |
| `xtask` | Which diffs should run the lane, where are receipts written, and how are findings summarized? |

## PR artifacts

A future `cargo xtask ripr-pr` wrapper should prefer stable repo-facing paths,
for example:

```text
target/ripr/pr/pr-summary.md
target/ripr/pr/repo-exposure.json
target/ripr/pr/review.md
target/ripr/pr/agent-packet.json
target/ripr/pr/first-useful-action.md
target/ripr/pr/first-useful-action.json
```

## Finding posture

- Advisory first while the baseline is learned.
- Prefer focused tests or clearer assertions over suppressions.
- Suppressions need owner, reason, selector, and expiry.
- High-confidence new gaps may become soft-gated after the repo demonstrates a
  clean baseline and review workflow.

## Claim boundary

A clean `ripr` receipt means no selected static mutation-exposure findings were
reported for the analyzed scope. It does not prove behavioral correctness,
coverage adequacy, or that runtime mutation would kill all relevant mutants.
