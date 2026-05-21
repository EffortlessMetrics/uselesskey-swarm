+++
id = "USELESSKEY-PROP-0002"
kind = "proposal"
title = "Source-of-truth control plane"
status = "proposed"
owner = "EffortlessMetrics"
created = "2026-05-21"
milestone = "control-plane"
linked_specs = [
  "USELESSKEY-SPEC-0001",
]
linked_adrs = [
  "USELESSKEY-ADR-0002",
  "USELESSKEY-ADR-0003",
]
support_tier_impact = []
policy_impact = [
  "policy/doc-artifacts.toml",
]
+++

# USELESSKEY-PROP-0002: Source-of-Truth Control Plane

## Problem

`uselesskey-swarm` already has the seed of a repo-native source-of-truth model:
proposals, specs, ADRs, implementation plans, active goal manifests, claim
ledgers, policy ledgers, handoffs, learnings, receipts, and generated evidence.
Those artifacts are valuable because `uselesskey` makes security-sensitive test
infrastructure, and users need precise claims with explicit proof and
boundaries.

The missing layer is a complete operating control plane. A later agent should be
able to enter the repo cold, read the active goal, follow linked plan/spec/ADR
artifacts, make one PR-sized change, run the listed proof, update affected
ledgers, write a PR body from repo truth, and close out the work without
reconstructing intent from chat history.

Without that control plane:

- fixture-contract work can become disconnected docs and ledgers;
- public claims can drift away from proof commands;
- active goals can become stale or archived without a replacement;
- PR bodies can omit claim, policy, rollback, or proof boundaries;
- later agents must infer next action from old prompts instead of current repo
  state.

## Users and Surfaces

Maintainers need to answer what is active, why it exists, what behavior must
hold, what proof validates it, what public claim it supports, and what remains
after merge.

Security and platform reviewers need public claims, support tiers, metadata-only
receipts, and policy ledgers that can be inspected without generated secret
material.

Downstream users need task-first docs and stable fixture-contract surfaces, but
they should not need to understand repo internals before they can copy a useful
command or test.

Codex and other repo agents need a current execution source in
`.uselesskey/goals/active.toml`, linked to one plan item and one proof set at a
time.

## Success Criteria

The lane succeeds when the repo can answer these questions from committed
artifacts:

- What work is active?
- Why does it exist?
- What behavior must hold?
- What proof command validates it?
- What public claim does it support?
- What policy ledger changed?
- What should Codex do next?
- What happened after merge?

The successful operating loop is:

```text
proposal
  -> spec
    -> ADR, where durable decision exists
      -> implementation plan
        -> active goal manifest
          -> PR
            -> proof command
              -> support-tier / public-claim map
                -> policy ledger update, if needed
                  -> closeout / handoff
```

## Proposed Stack

Build the full source-of-truth control plane inside `uselesskey-swarm` first:

- proposals own why, affected users, alternatives, and success criteria;
- specs own behavior contracts, proof, non-goals, and CI evidence;
- ADRs own durable architecture and policy decisions;
- implementation plans own PR sequence, rollback, promotion, and closeout
  shape;
- `.uselesskey/goals/active.toml` owns current agent work state;
- support-tier docs own stability and claim boundaries;
- `policy/claim-ledger.toml` owns public claim to proof mapping;
- `policy/doc-artifacts.toml` owns source-of-truth artifact inventory;
- fixture policy ledgers own stable negative fixture classes and other governed
  product state;
- PR templates and generated PR bodies own review packets;
- closeouts and handoffs own landed work, proof, remaining risks, and next safe
  action.

The fixture-contract payload is the first meaningful product object carried by
this stack: negative fixture IDs, bundle schemas, material classification,
task-first docs, public-surface status, support-tier claims, audit proof, and
closeout.

## Alternatives Considered

Keep using chat prompts as the lane source.

Rejected because chat is not durable repo state. It is useful operator intent,
but it should not be the only place where proof commands, boundaries, and next
work items live.

Build only the fixture-contract ledgers and docs.

Rejected because the fixture-contract payload needs the control plane around it.
Stable fixture IDs, schemas, support tiers, claims, PR bodies, and closeouts
compound when they are linked, and later work becomes cheaper because the repo
answers the next question.

Build a generic proof-stack platform immediately.

Rejected for this repo. `uselesskey-swarm` should prove the pattern with its own
product pressure first. Productization belongs later in `tokmd` or another
generic tool once the worked example is honest.

Make all source-of-truth checks blocking immediately.

Rejected because new graph checks should burn in as advisory evidence before
branch protection depends on them.

## Specs to Create or Update

- Keep `USELESSKEY-SPEC-0001` as the source-of-truth model seed.
- Add `USELESSKEY-SPEC-0023` for the source-of-truth enforcement contract.
- Keep fixture-contract specs such as `USELESSKEY-SPEC-0016`,
  `USELESSKEY-SPEC-0017`, `USELESSKEY-SPEC-0021`, and `USELESSKEY-SPEC-0022`
  linked to the first product payload carried by the control plane.

## Architecture Decisions Needed

Existing ADRs already cover the most important decisions:

- `USELESSKEY-ADR-0002`: Public claims require command-backed evidence.
- `USELESSKEY-ADR-0003`: Repo goals are the agent control plane.

Add a new ADR only if enforcement, CI promotion, or productization creates a
durable decision that outlives one implementation plan.

## Implementation Campaign Shape

Use small PRs in this order:

1. Source-of-truth doctrine.
2. Control-plane templates.
3. This proposal.
4. Enforcement spec.
5. Doc artifact ledger.
6. `cargo xtask check-doc-artifacts`.
7. Support-tier and claim mapping.
8. Goal manifest and goal checker.
9. Agent operating contract.
10. PR and issue templates.
11. Advisory source-of-truth CI.
12. Repo contract report, PR body generator, and closeout generator.
13. Apply the stack to OIDC/JWKS, JWT, webhook, TLS, start-here docs,
    downstream CI recipes, and release handoff.

The dedicated implementation plan should live at
`plans/source-of-truth-control-plane/implementation-plan.md` once that PR is
reached.

## Evidence Plan

Docs-only PRs should run:

```bash
cargo xtask docs-sync --check
cargo xtask typos
git diff --check
```

Proposal and spec PRs should additionally run:

```bash
cargo xtask spec-check --strict
```

Tooling PRs should add narrow xtask tests first, then run the new checker:

```bash
cargo test -p xtask <module_or_test_filter>
cargo xtask <new-checker>
git diff --check
```

CI promotion should start advisory. Blocking promotion requires clean burn-in on
real PRs.

## Risks

The graph can become performative if artifacts are added without proof commands
or closeout. Each artifact must own one job and link to the next artifact that
needs it.

The active goal can become stale if closeout does not advance, archive, or
supersede it.

Support-tier truth can fragment if specs, docs, and claim ledgers duplicate the
same tier language. Support-tier state should live in the status map and be
linked elsewhere.

New checks can create false positives if made blocking too early. Start
advisory and promote only the stable core.

## Non-goals

This lane does not move release, publish, signing, crates.io, GitHub release,
tag, or source-sync authority into `uselesskey-swarm`.

This lane does not add new fixture behavior by itself.

This lane does not make advisory checks blocking before burn-in.

This lane does not productize a generic `tokmd` proof-stack system.

This lane does not duplicate support-tier truth inside specs or CI lane truth
inside specs.

## Claim Boundary

This proposal supports a future claim that `uselesskey-swarm` has a repo-native
control plane for proposals, specs, ADRs, plans, active goals, public claims,
policy ledgers, proof commands, PR bodies, and closeouts.

It does not prove fixture correctness, provider compatibility, production token
security, production PKI behavior, downstream verifier correctness, or release
readiness.

## Rollback

Rollback is document-first. Revert a single PR when its artifact creates drift
or false authority. If the lane direction changes, mark this proposal
`superseded`, link the replacement proposal, and archive or supersede the active
goal when it exists.

## Exit Criteria

The proposal is complete when:

- the source-of-truth doctrine and templates exist;
- `USELESSKEY-SPEC-0023` defines the enforcement contract;
- doc artifacts, goals, support tiers, claims, and negative fixtures are
  machine-checkable where intended;
- PR and issue templates route changes through the graph;
- advisory source-of-truth CI has run on real PRs;
- generated repo contract reports, PR bodies, and closeouts can be created from
  committed source-of-truth artifacts;
- the fixture-contract payload is bound through support tiers, claims, ledgers,
  docs, proof, and closeout.
