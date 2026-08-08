+++
id = "USELESSKEY-ADR-0005"
kind = "adr"
title = "Mutation testing is not a repo CI lane"
status = "accepted"
owner = "EffortlessMetrics"
created = "2026-08-08"
linked_specs = [
  "USELESSKEY-SPEC-0006",
  "USELESSKEY-SPEC-0007",
  "USELESSKEY-SPEC-0010",
]
linked_adrs = []
support_tier_impact = []
policy_impact = []
+++

# USELESSKEY-ADR-0005: Mutation Testing Is Not a Repo CI Lane

## Decision

`uselesskey` does not run mutation testing as a repository CI lane.

The `cargo-mutants` integration is removed: the xtask commands (`mutants`,
`mutants-pr`, `mutants-nightly`), the nightly mutation workflow, the mutation
survivor ledger, the tool's presence in the routed CI runner image, and the
targeted-mutation routing that selected crates for it.

The tests written to kill mutants are kept. They are ordinary tests that assert
real behavior, and `cargo test` continues to run them.

## Context

Mutation testing was the most expensive signal in the repo and the least
proportionate to what `uselesskey` ships. It carried a nightly workflow, a
survivor ledger with expiries, a routing layer that decided which crates to
mutate on which PRs, and a source-built `cargo-mutants` in the routed runner
image.

That machinery answers "would a concrete mutant survive the test suite?" for a
*test-fixture* library whose correctness properties are deterministic identity,
shape, and scanner-safety — properties already proven directly by the
deterministic regression tests, the BDD suite, the negative-fixture taxonomy,
and the bundle/no-blob proofs.

The routing layer had its own cost: `requires_targeted_mutation` threaded
through impacted-evidence, PR-lite heavy routing, and the PR summary, and
existed mainly to decide when to pay for mutation.

## Consequences

CI is cheaper and simpler. The routed runner image no longer source-builds
`cargo-mutants`. `cargo xtask ci` loses its slowest stage. PR evidence keeps one
advisory weak-oracle signal (`ripr`) instead of a static signal plus a runtime
mutation lane plus the routing between them.

The repo loses the mutation-adequacy signal. Nothing now answers "would a
concrete mutant survive?" — this is accepted, not mitigated. Claim boundaries
that already disclaim mutation adequacy (coverage receipts, PR-lite receipts)
stay accurate and are deliberately retained.

`ripr` is no longer described as a cheap precursor to a runtime mutation
backstop, because there is no backstop behind it. It is advisory on its own
terms.

## Required Evidence

- `cargo xtask ci` completes without a mutation stage.
- `cargo test --workspace --all-features` still runs the retained
  mutation-hardening tests.
- `cargo xtask check-file-policy` reports no unmatched or unused entries after
  the mutation config and ledger files are removed.
- No active command, workflow, configuration, or operational document
  references a removed `mutants*` command. This ADR and `CHANGELOG.md` name
  them deliberately; historical records keep the trail readable.

## Claim Boundary

Records that mutation testing is not part of this repository's CI. It does not
claim mutation testing lacks value in general, does not claim the retained
hardening tests provide equivalent assurance, and makes no claim about fixture
or cryptographic behavior.

## Alternatives Considered

Keep mutation testing but only on nightly. Rejected: the nightly workflow,
survivor ledger, routing layer, and runner-image cost all remain, which is most
of the weight, for a signal no PR would see.

Keep the xtask commands for on-demand local use while removing them from CI.
Rejected as a default: it retains the config, survivor ledger, routing, and
docs surface — the bulk of the complexity — while leaving commands nothing in
CI exercises. Reinstating the integration is a revert away if wanted.

Delete the mutation-hardening tests along with the tooling. Rejected: those
tests assert real behavior and pass under `cargo test` independently of how they
were originally motivated.

## Non-goals

Does not change the deterministic regression tests, BDD suite, negative-fixture
taxonomy, fuzz lane, coverage lane, or release evidence beyond removing the
mutation step.

Does not weaken any claim boundary. Negative claims such as
`not_mutation_adequacy` remain in coverage and PR-lite receipts precisely
because they are still true.

## Rollback

Revert the removal commit. That restores the xtask commands, the nightly
workflow, `.cargo/mutants.toml`, `policy/mutation-survivors.toml`, the runner
image install, and the targeted-mutation routing together.

## Follow-up Specs / Plans

`USELESSKEY-SPEC-0006`, `USELESSKEY-SPEC-0007`, and `USELESSKEY-SPEC-0010` are
amended in the same change to describe the lanes that actually run.
