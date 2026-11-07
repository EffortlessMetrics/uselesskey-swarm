# Hosted light CI repair

## Plan and acceptance

Restore the workflow-validation and docs-policy-light jobs to `ubuntu-latest`,
matching docs/ci/routed-rust-workflow.md. Run 41565868359 selected workflow
validation successfully, but job 125687188752 waited 24 hours for the
`em-ci-tiny` self-hosted labels and was canceled without executing a step.
The normalized result consequently failed.

Update the two job-specific policy tests to require hosted execution and reject
self-hosted configuration. Preserve route classification, fork guards, proof
commands, permissions, and the normalized required check. No release work or
changes to heavy Rust/performance runners are included.

Proof: `cargo test -p xtask routed_`, `cargo test -p xtask workflow_hygiene_guard`,
`bash ci/check-bare-self-hosted.sh .github/workflows`, and `git diff --check`.
Hosted completion must be observed separately; local checks only establish the
workflow contract. Revert this patch to roll back. Heavy runner availability is
a separate infrastructure follow-up.

## Validation

Pass: three routed contract tests, two workflow hygiene tests, shell runner
guard, YAML parsing, and whitespace checks. Hosted execution and full Rust
workspace tests were not established by these local checks.
