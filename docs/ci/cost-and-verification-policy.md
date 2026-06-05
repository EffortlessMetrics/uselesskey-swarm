# Cost and verification policy

CI exists to buy proof. The repo goal is proof per Linux-equivalent minute
(LEM), not fewer checks.

We reduce wasted CI so that expensive validation can run where it matters:
small deterministic PR gates by default, deeper runtime evidence when risk,
labels, main, nightly, or release lanes justify the spend.

## Principles

- Default PR lanes should be cheap, deterministic, and high-signal.
- Optional lanes are routed by changed surface, label, branch, schedule, or
  release intent.
- A skipped optional lane is not a pass; it is a policy decision.
- Branch protection should prefer a normalized aggregate result over many leaf
  jobs whose routing details can change.
- Receipts should explain which lanes ran, which lanes were skipped by policy,
  and which claim boundary the run supports.

## Lane posture

| Lane type | Default PR? | Purpose |
| --- | --- | --- |
| Fast Rust gate | Yes | Formatting, compile/lint, and focused tests for ordinary Rust changes. |
| Source-of-truth advisory | Yes when relevant | Detect policy, docs, goals, claim, and receipt drift. |
| Static mutation exposure (`ripr`) | Advisory / risk-routed | Surface weak-oracle risk earlier than runtime mutation. |
| Unsafe review | Advisory / unsafe-routed | Check changed unsafe seams for reviewable contracts and witness routes. |
| Coverage | No | Measure execution surface on main, manual, release, or labelled runs. |
| Runtime mutation | No | Calibrate and backstop test adequacy on risk, nightly, or release lanes. |
| Miri / fuzz | No | Run concrete UB or robustness witnesses where the changed surface warrants it. |

## Aggregate gate doctrine

The durable merge signal should report the policy outcome rather than requiring
every leaf workflow directly. Leaf jobs may be required by the aggregate when
policy selects them, but optional jobs should be reported as one of:

- `passed`;
- `failed`;
- `skipped-by-policy`;
- `advisory-failed`.

For this swarm repository, `policy/ci-checks.toml` maps CI checks to their
roles, including the normalized `Uselesskey Rust Small Result` merge signal and
advisory source-of-truth checks.
