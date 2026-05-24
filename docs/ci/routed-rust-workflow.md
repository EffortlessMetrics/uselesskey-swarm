# Routed Rust Workflow

The swarm repository routes Rust proof through
`.github/workflows/em-ci-routed-rust.yml`. Branch protection should depend on
the normalized `Uselesskey Rust Small Result` check, not the conditional runner
jobs.

All jobs run on self-hosted runners. There is no GitHub-hosted fallback; the
router emits `route-fail` rather than silently falling back to a hosted runner.

## Runner Selection

The router first classifies the change:

| Change class | Target | Notes |
| --- | --- | --- |
| Docs, policy, plans, Rails, goal, and selected metadata paths | `docs` | Runs the self-hosted docs/policy light path. |
| Rust, workflow, or mixed implementation changes | self-hosted Rust runner when available | Uses org-level runner discovery and the CPX42/CX43/CX53 contract. |
| Fork PRs | `route-fail` | Fork PRs are not supported on self-hosted runners; convert to an in-repo branch. |

The workflow keeps `cancel-in-progress: false` so a heavy/core run that is
already executing is not canceled by a newer push. GitHub Actions may still keep
only one pending replacement run for the same concurrency group.

## Route Failure Reasons

When the router emits `route-fail`, inspect the reason before retrying:

| Reason | Meaning | Normal response |
| --- | --- | --- |
| `fork_pr_not_supported` | The PR is from a fork. | Move the change to an in-repo branch; fork PRs cannot run on self-hosted runners. |
| `self_hosted_not_marked_ready` | Repo variable says self-hosted runners are not ready. | Resolve the readiness flag (`USELESSKEY_SELF_HOSTED_READY`) before retrying. |
| `no_idle_runner` | No matching idle self-hosted runner was found. | Wait for an idle runner or provision more capacity. |
| `runner_token_missing`, `runner_token_unauthorized`, `runner_token_forbidden`, `runner_api_failed`, `parse_failed` | Runner discovery failed. | Treat as CI infrastructure triage; do not paper over repeated discovery failures without recording the reason. |
| `invalid_force_target`, `unknown_change_class` | Router input was invalid. | Inspect the workflow_dispatch inputs or the change classifier; fix the upstream cause. |

## Local Proof

When changing the routed workflow or its policy test, run:

```bash
cargo test -p xtask routed_rust_workflow_uses_org_runner_discovery_and_cpx42_contract
git diff --check
```

Add broader `xtask` policy tests when the change affects more than one routed
contract.
