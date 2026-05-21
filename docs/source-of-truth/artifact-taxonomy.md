# Artifact Taxonomy

Each source-of-truth artifact owns one job. If a document starts owning more
than one job, split the extra truth into the artifact that owns it.

| Artifact | Owns | Must link to | Does not own |
| --- | --- | --- | --- |
| `README.md` | First-hour product truth and stable entry points | Task docs, status pages, proof badges | Full proof matrix, active lane state |
| Proposal | Why a lane exists, affected users, value, alternatives, success criteria | Expected specs, ADRs, plan | Detailed implementation steps |
| Spec | Behavior contract, non-goals, acceptance, evidence, CI proof | Proposal or standalone reason, ADRs, plan | Queue management or narrative rationale |
| ADR | Durable architecture or policy decision and consequences | Proposal and affected specs | PR sequence or proof receipts |
| Implementation plan | PR sequence, proof commands, rollback, closeout shape | Proposal, specs, ADRs, work items | Product claims or durable decisions |
| Active goal manifest | Current agent lane, ready work items, proof commands | Plan item, spec, proposal | Historical closeout |
| Claim ledger | Public claim to proof command, docs, artifacts, release lane | Specs, docs, support tiers | Long-form explanation |
| Policy ledger | Governed state, exceptions, classifications, or supported IDs | Specs, docs, checks | Public marketing copy |
| Support-tier status | Stability level and boundary for a surface or claim | Claim ledger, docs, proof command | Fixture implementation details |
| Receipt / generated artifact | Machine-readable proof output or generated endpoint | Producing command and schema, when present | Human rationale |
| PR body | Review packet for one change | Proposal, spec, plan item, proof | Permanent lane state |
| Closeout / handoff | Landed work, proof, remaining risks, next action | Goal, plan, receipts, PRs | Active instructions after archive |

## Public Claim Chain

Public claims use this chain:

```text
README or task doc claim
  -> policy/claim-ledger.toml
    -> spec
      -> proof command
        -> receipt, badge, or generated artifact
          -> support tier
            -> release or closeout evidence
```

A claim is not stable just because it appears in prose. Stable and stabilizing
claims need proof commands and a boundary that says what the proof does not
cover.

## Policy Ledger Chain

Policy-ledger entries use this chain:

```text
spec
  -> policy ledger
    -> status matrix or docs
      -> checker command, when implemented
        -> receipt or CI result
```

Policy ledgers should be machine-readable when downstream tools or agents must
branch on their contents.

## Closeout Chain

Closeouts preserve history without becoming active instructions:

```text
merged PRs
  -> proof commands and receipts
    -> affected claims and policies
      -> remaining risks
        -> next recommended goal or archived goal manifest
```

When a lane is closed, update or archive the active goal instead of relying on
the closeout as the next execution source.
