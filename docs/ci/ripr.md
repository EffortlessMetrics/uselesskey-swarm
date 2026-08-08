# ripr static weak-oracle exposure lane

`ripr` is static weak-oracle exposure analysis. It flags changed behavior whose
tests or assertions look weak, cheaply and at PR time.

It does not execute the code under analysis, prove correctness, or measure
coverage. It is an advisory signal for where focused tests are missing.

## Role split

| Tool | Question |
| --- | --- |
| `ripr` | Did the changed code expose behavior whose tests or oracles look weak? |
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

A clean `ripr` receipt means no selected static weak-oracle findings were
reported for the analyzed scope. It does not prove behavioral correctness or
coverage adequacy.
