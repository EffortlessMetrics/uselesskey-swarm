# Linking Model

Source-of-truth links make repo state traversable without chat history. Links
should be explicit, stable, and narrow enough for a checker to validate later.

## Stable IDs

Use the existing uselesskey ID families for current product artifacts:

| Kind | ID pattern | Example |
| --- | --- | --- |
| Proposal | `USELESSKEY-PROP-0000` | `USELESSKEY-PROP-0001` |
| Spec | `USELESSKEY-SPEC-0000` | `USELESSKEY-SPEC-0001` |
| ADR | `USELESSKEY-ADR-0000` | `USELESSKEY-ADR-0003` |
| Plan item | Lowercase words with hyphens | `doc-artifact-ledger` |
| Claim | Lowercase words with hyphens | `metadata-only-audit-packets` |
| Negative fixture ID | Lowercase words with underscores | `jwks_duplicate_kid` |

Use Rails-scoped IDs such as `RAILS-PROP-*`, `RAILS-SPEC-*`, `RAILS-ADR-*`,
and `RAILS-LANE-*` only for portable Rails framework artifacts under `.rails/`.
Existing `USELESSKEY-*` IDs remain valid for current uselesskey proposals,
specs, ADRs, and plans.

Do not change a stable ID after it appears in docs, ledgers, receipts, schemas,
or generated artifacts. Supersede it with a new ID and a replacement link.

## Front Matter Links

Specs, proposals, and ADRs use TOML front matter. Prefer fields that already
exist in the local templates:

```toml
linked_proposal = "USELESSKEY-PROP-0001"
linked_specs = ["USELESSKEY-SPEC-0001"]
linked_adrs = ["USELESSKEY-ADR-0003"]
linked_plan = "plans/spec-system/implementation-plan.md"
support_tier_impact = []
policy_impact = []
```

Accepted specs should link to a proposal unless they include a clear standalone
reason. ADRs should link to the proposal and specs they constrain. Plans should
name the proposal, specs, ADRs, proof commands, rollback, and closeout shape for
the lane.

## Active Goal Links

Active goal manifests use `status = "active"` for the current lane and
`status = "archived"` after closeout. Work items should be small enough to
review as one PR:

```toml
[[work_item]]
id = "goal-manifest-checker"
status = "ready"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-0023"
plan = "plans/source-of-truth-control-plane/implementation-plan.md"
commands = ["cargo xtask check-goals", "git diff --check"]
```

Work-item status values should stay simple:

```text
planned
ready
active
blocked
done
```

Blocked items need a concrete `blocked_by` value. Done items need proof commands
or receipt references.

## Link Direction

Use forward execution links for active work:

```text
active goal -> plan item -> spec -> proposal / ADR
```

Use backward evidence links for review and closeout:

```text
closeout -> PR -> proof command -> receipt -> claim / policy / spec
```

This keeps Codex from using a broad proposal as a task list and keeps reviewers
from treating a PR body as durable policy.

## Drift Rules

When a change touches one artifact, update only the linked artifacts whose truth
changed:

- Behavior or proof changes update specs.
- Durable decisions update ADRs.
- PR sequencing changes update plans and active goals.
- Public promise changes update claim ledger and support tiers.
- Negative fixture classifications update the negative-fixture ledger and
  matrix.
- Generated proof changes update receipts or endpoints through their generator.

Do not duplicate support-tier truth inside specs. Do not duplicate CI lane truth
inside specs. Link to the owning artifact instead.
