# No-Actions Development

`uselesskey-swarm` is the internal implementation repository. GitHub issues,
branches, pull requests, comments, reviews, and merge history remain the work
control plane. GitHub Actions is not part of the intended ordinary development
path. The workflow and repository-settings cutover is a separate operation;
this local proof tool does not disable existing hosted triggers.

The repository must be green before a pull request is merged. Removing hosted CI
moves implementation proof and review before the merge boundary.

## Development sequence

1. Start from a controlling issue with a bounded claim and an operator-reviewed
   proof plan.
2. Implement on a branch from current `main` and commit the final candidate.
3. Record an exact existing ancestor commit as the comparison base. Review the
   changed paths, required obligations, focused commands, and justified skips.
4. Run the invocation wrapper in a clean dedicated checkout. It executes the
   focused commands followed by `cargo xtask pr`, then seals the result.
5. Retain the complete invocation envelope at an immutable evidence locator and
   record its digest and the accepted plan digest.
6. Complete substantive review and repair every accepted finding.
7. Merge only when proof and review cover the exact final head and no known
   blocker remains.

The wrapper and canonical admission share the pure validator in
`scripts/development_proof_contract.py`. Canonical vendors the exact reviewed
contract bytes; source and canonical changes to this contract must be coordinated
under the admission owner, not maintained as independent validators.

The wrapper requires Python 3.11 or newer, Git, and the tools used by the selected
proof commands. Store the reviewed plan under ignored `target/`, or outside
the checkout, so creating it does not dirty the candidate:

```bash
python3 -B scripts/seal-development-proof.py --run \
  --base <full-existing-ancestor-commit> \
  --plan target/development-proof-plan.toml
```

The command prints the exact `head`, `proof_head`, and
`proof_receipt_sha256` fields, plus the invocation archive path, invocation ID,
and proof-plan SHA-256. The receipt digest now names the complete **development
invocation envelope**, not `target/xtask/receipt.json`.

A direct `cargo xtask pr` run is still useful for development feedback. Its bare
schema-2 receipt cannot be sealed. Both the default sealing command and the
older `--receipt target/xtask/receipt.json` form reject a bare receipt and direct
the operator to `--run`.

## Review the plan before execution

A plan declares exact successful xtask obligations and focused command argv.
The accepted comparison base must cover the complete development change being
claimed, including earlier product commits. Ancestry alone does not prove that
range is complete; canonical acceptance must bind the intended base and range.
The wrapper always requires successful `detect-changes`, `public-surface`,
and `spec-check` steps in the final PR receipt.

Product changes need a required successful behavioral step, such as
`test:uselesskey-webhook`, or an explicit focused product test command.
Documentation-only changes need a focused documentation check. Changes to
control files, scripts, policy, or xtask need focused control proof, including
when product files change in the same candidate. Unclassified paths fail before
commands run; route them explicitly in the classifier through reviewed source
changes.

The `profile` is `product` when any product path changes, otherwise `control`
when a control path changes, otherwise `docs`. This is a conservative routing
check. It does not decide that selected tests adequately cover the claim.

This example is for a Python proof-tool change with accompanying policy and
documentation edits. It is not an accepted plan for product or Rust changes:

```toml
schema_version = 1
profile = "control"
required_steps = ["spec-check"]

[allow_skips]
"fmt" = "This candidate changes no Rust source or Cargo manifest."
"clippy" = "This candidate changes no Rust source or Cargo manifest."
"tests" = "Focused Python regressions cover the changed control behavior."
"feature-matrix" = "No facade, feature, or Cargo surface changes."
"dep-guard" = "No dependency or Cargo surface changes."
"bdd" = "No product behavior or BDD scenario changes."
"fuzz" = "No product source or fuzz target changes."
"no-blob" = "No product fixture or test data changes."
"coverage" = "Product coverage is outside this control-only claim."
"coverage:report" = "No product coverage report is claimed."
"root-tests" = "No Rust integration-test changes."
"xtask-tests" = "No Rust xtask implementation changes."
"preflight:metadata" = "No Cargo or package metadata changes."
"preflight:doc-versions" = "No package version changes."
"preflight:public-surface" = "No package surface changes; baseline public-surface still runs."
"preflight:package:*" = "No package contents or Cargo surface changes."

[[focused]]
name = "proof-regressions"
kind = "control"
command = ["python3", "-B", "-m", "unittest", "discover", "-s", "scripts", "-p", "test_development_proof.py"]

[[focused]]
name = "file-policy"
kind = "control"
command = ["cargo", "xtask", "check-file-policy"]
```

Every actual skipped step needs both a reason in the raw receipt and an explicit
allowance in the reviewed plan. Exact names are preferred. The only supported
skip families are `test:*`, `feature-matrix:*`, and `preflight:package:*`.
A required successful step cannot also be allowed to skip. Skips remain skips
in the envelope.

Focused commands run as argv without a shell. The allowlist accepts bounded
test and repository-check command families and rejects arbitrary commands,
publication, help, listing, and build-only test requests. The reviewer must
use the explicit `python -m unittest ...` form for Python test commands; directly
executing a file named `test_*.py` is insufficient. The reviewer must
still examine the complete argv, selected tests, repository configuration,
execution environment, and claim coverage. A command name or exit code alone
does not establish test adequacy or a nonzero test count.

The wrapper does not change the Rust selector. At this source revision,
`cargo xtask pr` can select fuzz for crate-source changes, optional coverage
when its tool is installed, and package preflight for Cargo changes. This
wrapper adds no full-CI, fuzz, coverage, or publication command. Those existing
selection and cost policies need their own reviewed changes.

## Invocation and evidence contract

Before running anything, the wrapper replaces the latest envelope with a new
incomplete invocation. It rejects a dirty index, tracked changes, untracked
source, a missing base, a base that is not an ancestor, or a base equal to the
head. It also rejects index flags such as `assume-unchanged` and `skip-worktree`
that can conceal tracked edits. It records the exact head, committed tree, and
base, and sets
`XTASK_BASE_REF` to that exact base for every command.

The wrapper checks the subject before each command and after all commands. It
moves any previous raw xtask receipt into the invocation archive before the
final PR command, requires that command to exit successfully, and requires a
fresh receipt within that command's time window. A failed, interrupted, partial,
or detect-only invocation cannot produce a seal. Process-launch failures are
recorded as terminal failures with an inspectable error.

Successful envelopes have this versioned structure:

| Field | Meaning |
| --- | --- |
| `schema_version`, `kind` | Integer `1`, `uselesskey-development-proof` |
| `invocation_id` | New 32-character lowercase hexadecimal invocation ID |
| `subject.head`, `subject.tree`, `subject.base` | Exact 40-character Git object IDs |
| `started_at_ms`, `finished_at_ms`, `conclusion` | Invocation interval and terminal `success` |
| `plan.sha256`, `plan.toml` | Digest and exact UTF-8 bytes of the reviewed plan |
| `classification` | Sorted changed paths, classes, and selected profile |
| `outcomes` | Ordered name, argv, start, finish, and exit code for every planned command |
| `xtask_receipt.sha256`, `xtask_receipt.utf8` | Digest and exact UTF-8 bytes of the fresh schema-2 PR receipt |

Each plan or raw receipt is read once; validation, embedded text, and digest use
that same byte snapshot. The sealing operation likewise validates and hashes
one envelope snapshot. Matrix entries must agree with their corresponding
successful or explicitly skipped raw step. Missing required obligations,
duplicate steps or JSON fields, failed results, unknown statuses, changed
command outcomes, and mismatched digests are rejected.

The current envelope is `target/development-proof/receipt.json`. Each invocation
also has `target/development-proof/runs/<invocation_id>/receipt.json`. A successful
`--run` seals its own archive so a later invocation cannot change the evidence
it selects. Failed invocations invalidate the latest default receipt; older
archives remain available as historical evidence. To recheck an archive:

```bash
python3 -B scripts/seal-development-proof.py \
  --receipt target/development-proof/runs/<invocation_id>/receipt.json
```

Run one proof invocation per checkout. The wrapper lock excludes other wrapper
invocations; it does not exclude direct legacy xtask commands or arbitrary
processes writing the same output files. Do not run shared-output commands
concurrently. A lock surviving a hard process kill must be inspected before it
is cleared. Use a dedicated worktree for the proof run.

The envelope is locally produced evidence. It is not a signature, a trusted
runner attestation, a guarantee that candidate-controlled tests are adequate,
or proof that development changes compose correctly in canonical. The operator
must accept the plan and exact-head review, retain the envelope bytes at an
immutable locator, and bind both the envelope digest and the plan digest in the
canonical acceptance record. Committing evidence into the already-proven
candidate changes its head; use a separate evidence commit or other supported
immutable evidence location.

## Merge standard

A development pull request is mergeable only when:

- the exact final head has current focused proof under its accepted plan;
- `cargo xtask pr` completed successfully in the sealed invocation;
- the complete envelope and its digest are retrievable;
- substantive review is complete for the exact head;
- unresolved review threads are terminal;
- `known_blockers` is empty;
- the claim boundary states what was and was not established.

A push after proof or review invalidates the earlier exact-head disposition.
Run the invocation again against the new candidate and refresh review wherever
the change can affect prior findings. An older historical success does not
erase a later failed attempt or a known blocker.

## Promotion

One canonical promotion may compose one or more accepted development pull
requests. Each included pull request contributes a `[[development]]` entry with:

```toml
repository = "EffortlessMetrics/uselesskey-swarm"
pr = 123
head = "<exact accepted development head>"
proof_head = "<same exact head>"
proof_receipt_sha256 = "<development invocation envelope digest>"
```

The promotion also needs the immutable evidence locator, accepted plan digest,
and exact-head review disposition required by its canonical admission tooling.
The canonical pull request remains Draft while composition, compatibility, and
review are incomplete. True Ready means the exact canonical head is expected to
pass its first required integration run.

## Red handling

A failed local or owned proof blocks the development pull request. A failed
canonical integration proof blocks that promotion and dispatches a repair to the
actual owner. It does not authorize a bypass, an empty CI-poke commit, or a
repository-wide halt on unrelated development.

## Tool regression checks

```bash
python3 -B scripts/seal-development-proof.py --self-test
```

These tests use real temporary Git repositories and stub the Cargo execution
boundary. They establish the wrapper's failure handling and evidence checks;
they do not establish that Rust checks passed or that a live canonical lifecycle
was exercised.

## Boundaries

This process establishes repository-owned development proof and review for the
named exact heads. Release publication, crates.io pushes, GitHub Releases, and
performance evidence require their separately authorized operations. Mutation
testing is not a repository CI lane under
[ADR 0005](../adr/USELESSKEY-ADR-0005-mutation-testing-is-not-a-repo-ci-lane.md).
