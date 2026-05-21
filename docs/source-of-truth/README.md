# Source-of-Truth Stack

`uselesskey` uses repo-native artifacts to keep product claims, proof, and
agent execution state separate. The stack is deliberately small per artifact:
each file owns one kind of truth and links to the next artifact that needs it.

The operating loop is:

```text
proposal
  -> spec
    -> ADR, where a durable decision exists
      -> implementation plan
        -> active goal manifest
          -> PR
            -> proof command
              -> support-tier / public-claim map
                -> policy ledger update, if needed
                  -> closeout / handoff
```

The source-of-truth model is defined by
[`USELESSKEY-SPEC-0001`](../specs/USELESSKEY-SPEC-0001-source-of-truth-model.md).
This directory explains how humans and agents should use that model during
normal repo work.

## Read Order

For active work, start from the narrowest current-state artifact:

1. `.uselesskey/goals/active.toml`
2. The linked implementation plan item
3. The linked spec
4. The linked ADR, when a durable decision is needed
5. The linked proposal, only for why and context
6. Affected policy ledgers, support tiers, docs, and receipts

Do not treat old chat prompts, stale handoffs, or historical plans as active
instructions when the active goal and linked plan disagree.

## Artifact Guides

- [artifact-taxonomy.md](artifact-taxonomy.md) defines what each artifact owns.
- [linking-model.md](linking-model.md) defines how artifacts refer to each
  other.
- [agent-operating-model.md](agent-operating-model.md) defines the cold-start
  workflow for Codex and other repo agents.
- [codex-operating-contract.md](codex-operating-contract.md) defines the
  Codex-specific PR, proof, ledger, and merge rules for this repo.

## Boundaries

This stack does not move release, publish, signing, tag, crates.io, or GitHub
release authority into `uselesskey-swarm`.

This stack does not make new CI checks blocking by itself. Enforcement starts as
explicit proof commands and becomes blocking only after a separate policy or CI
change promotes it.

This stack does not replace task-first user docs. User docs explain what to copy
and what a workflow proves; source-of-truth docs explain why the repo can make
that claim.
