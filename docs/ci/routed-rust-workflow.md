# Routed Rust Workflow

The swarm repository routes Rust proof through
`.github/workflows/em-ci-routed-rust.yml`. Branch protection should depend on
the normalized `Uselesskey Rust Small Result` check, not the conditional runner
jobs.

## Runner Selection

The router first classifies the change:

| Change class | Target | Notes |
| --- | --- | --- |
| Docs, policy, plans, Rails, goal, and selected metadata paths | `docs` | Runs the hosted docs/policy light path. |
| Workflow changes | `route-fail` unless allowlisted | Requires explicit hosted fallback approval. |
| Rust or mixed implementation changes | self-hosted Rust runner when available | Uses org-level runner discovery and the CPX42/CX43/CX53 contract. |
| Fork PRs | `github` | Hosted fallback is allowed for fork safety. |

The workflow keeps `cancel-in-progress: false` so a heavy/core run that is
already executing is not canceled by a newer push. GitHub Actions may still keep
only one pending replacement run for the same concurrency group.

## Hosted Fallback

Use the `allow-github-hosted` PR label only when hosted fallback is acceptable
for the specific PR. The label allows fallback for workflow changes, unavailable
self-hosted readiness, or no idle self-hosted runner. It does not move the
release/source boundary and does not authorize release, publish, signing, tag,
GitHub release, crates.io, or source-sync work.

If the router fails with one of these reasons, inspect the PR scope before
adding the label:

| Reason | Meaning | Normal response |
| --- | --- | --- |
| `workflow_change_requires_allow_github_hosted` | The PR changes `.github/workflows/*`. | Review the workflow diff, then add `allow-github-hosted` if the hosted fallback proof is acceptable. |
| `self_hosted_not_marked_ready` | Repo variable says self-hosted runners are not ready. | Add the label only for a PR that can safely use hosted fallback. |
| `no_idle_runner` | No matching idle self-hosted runner was found. | Add the label when waiting for a self-hosted runner is not necessary. |
| `runner_token_missing`, `runner_token_unauthorized`, `runner_token_forbidden`, `runner_api_failed`, `parse_failed` | Runner discovery failed. | Treat as CI infrastructure triage; do not paper over repeated discovery failures without recording the reason. |

After adding the label, create a fresh `synchronize` event if the failed run was
already queued before the label existed. Rerunning the old workflow can reuse the
old event payload and miss the new label.

## Local Proof

When changing the routed workflow or its policy test, run:

```bash
cargo test -p xtask routed_rust_workflow_uses_org_runner_discovery_and_cpx42_contract
git diff --check
```

Add broader `xtask` policy tests when the change affects more than one routed
contract.
