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
| Workflow changes | `workflow` | Runs hosted workflow validation and the no-bare-self-hosted guard; it does not run Rust CI. |
| Rust or mixed implementation changes | self-hosted Rust runner when available | Uses org-level runner discovery and the CX43/CPX42/CX53 capacity contract. |
| Fork PRs | `github` | Hosted fallback is allowed for fork safety. |
| Push to `main`, or `workflow_dispatch` with `run_full_gate=true` | `main-full` | Runs the hosted full gate and makes the normalized result follow that job. |

The workflow keeps `cancel-in-progress: false` so a heavy/core run that is
already executing is not canceled by a newer push. GitHub Actions may still keep
only one pending replacement run for the same concurrency group.

Pushes to `main` do not use PR runner discovery. They run
`Uselesskey Main Full Gate`, and the normalized `Uselesskey Rust Small Result`
waits for that full gate so the branch state is not marked red merely because no
self-hosted PR runner is idle.

## Main Branch Queue Behavior

Because main pushes share one routed Rust concurrency group, a newer main push
can stay `pending` while an older `Uselesskey Main Full Gate` is still running.
That is expected while the older run is within the workflow timeout. Treat the
pending newer run as queued, not failed.

Normal response:

- inspect the older run and confirm it is still progressing or still within its
  timeout;
- inspect the latest main `Source of Truth` run separately;
- do not rerun, cancel, or label a main run merely because a newer replacement
  run is pending.

Escalate only when the active full gate fails, exceeds its timeout, or is
clearly stuck in the same step beyond the timeout policy.

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
| `self_hosted_not_marked_ready` | Repo variable says self-hosted runners are not ready. | Add the label only for a PR that can safely use hosted fallback. |
| `no_idle_runner` | No matching idle self-hosted runner was found. | Add the label when waiting for a self-hosted runner is not necessary. |
| `runner_token_missing`, `runner_token_unauthorized`, `runner_token_forbidden`, `runner_api_failed`, `parse_failed` | Runner discovery failed. | Treat as CI infrastructure triage; do not paper over repeated discovery failures without recording the reason. |

After adding the label, create a fresh `synchronize` event if the failed run was
already queued before the label existed. Rerunning the old workflow can reuse the
old event payload and miss the new label. A `workflow_dispatch` run with
`force_target=github` is useful as ad hoc branch proof, but it does not update
the PR's `pull_request` check rollup used by branch protection.

## Local Proof

When changing the routed workflow or its policy test, run:

```bash
cargo test -p xtask routed_rust_workflow_uses_org_runner_discovery_and_capacity_contract
ci/check-bare-self-hosted.sh
git diff --check
```

Add broader `xtask` policy tests when the change affects more than one routed
contract.
