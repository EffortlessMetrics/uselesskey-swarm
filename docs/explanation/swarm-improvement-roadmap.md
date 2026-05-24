# Swarm Improvement Roadmap

This roadmap governs improvement work in `EffortlessMetrics/uselesskey-swarm`.

It is not a release roadmap and does not authorize publishing, tagging, signing,
GitHub releases, crates.io pushes, or source synchronization.

The roadmap exists to improve the swarm repository as a staging, proof, and
coordination environment. It is the default work selector when no active Rails
lane and no active `.uselesskey` goal are available.

## Current State

- No active Rails lane.
- Current `.uselesskey` goal is archived.
- External-adoption proof matrix is broad.
- Workflow-support matrix needs curation.
- Improvement work should optimize signal, reviewability, and maintainability.

## Roadmap Rules

### Allowed work when no active lane/goal exists

- Fix a failing check.
- Reduce duplicated docs/examples.
- Improve evidence quality and receipt clarity.
- Improve source-of-truth reporting.
- Audit workflow-support rows.
- Add tests preventing known drift.
- Produce closeout or decision memos.

### Disallowed work when no active lane/goal exists

- Add workflow surfaces, examples, claims, or support rows without this roadmap
  explicitly asking for it.
- Add public-claim scope without user need and proof boundaries.
- Prepare release/publish/source-sync artifacts.

### New workflow admission rule

A new workflow-support row requires all of:

1. Named user task.
2. Distinct value versus existing rows.
3. Clean-project proof.
4. Claim/tier/boundary alignment.
5. Maintenance owner/class.
6. Cost estimate.
7. Exit/retirement condition.

### New proof admission rule

New proof commands, examples, or lanes must identify the unique new failure
class they catch. “Adds matrix size” is insufficient rationale.

## Horizon 0: Swarm throttle and no-active-work rules

**Outcome:** agents do not invent plausible but low-value work when no lane is
active.

### Acceptance criteria

- `repo-contract-report` no-active-work guidance points to this roadmap.
- `AGENTS.md` states no-active-work agents must consult this roadmap.
- A regression test covers no-active-work guidance text.

## Horizon 1: Proof quality over proof quantity

**Outcome:** proof becomes integrated, comparable, and cheaper to review.

### Workstreams

- Integrated swarm-health profile (command or documented recipe).
- Proof receipts index (`docs/status/proof-receipts.md`) mapping command →
  receipt → boundary.
- Proof delta summaries in PRs (“what new failure this catches”).

### Acceptance criteria

- Reviewers can pick the right proof command by work type.
- Every proof surface declares receipt and boundary.
- New proof additions include “new failure caught” rationale.

## Horizon 2: Workflow/support matrix curation

**Outcome:** supported workflows are intentional instead of exhaustive.

### Workstreams

- Classify each workflow row (`core`, `supported`, `example-only`,
  `candidate`, `merge`, `retire`).
- Audit file: `docs/status/workflow-support-audit.md`.
- Consolidation pass for overlapping workflows.

### Acceptance criteria

- Every workflow row has a curation class.
- Candidate rows include promotion criteria.
- Example-only items are not framed as public promises.

## Horizon 3: Adoption lab

**Outcome:** clean-project examples simulate realistic downstream use.

### Workstreams

- Downstream persona map (3–5 personas).
- Scenario-based example organization (job-first, not crate-first).
- Published-version simulation mode distinct from path-patched mode.
- Friction log: `docs/learnings/swarm-adoption-friction.md`.

### Acceptance criteria

- Examples map to named personas/scenarios.
- Path-patched versus published-version proof is explicit.
- Friction entries drive concrete follow-ups.

## Horizon 4: Agent governance and observability

**Outcome:** swarm behavior is measurable and steerable.

### Workstreams

- PR intent taxonomy (“improvement class”).
- Drift budget rule (cap docs-only streaks).
- Swarm digest template (`docs/status/swarm-digest-template.md`).
- Review-capacity status in PR checklists.

### Acceptance criteria

- PRs disclose intent class and review status.
- Drift patterns are summarized periodically.
- No-active-work agents use a bounded menu.

## Horizon 5: Debt burn-down and simplification

**Outcome:** the repo removes low-value surface area faster than it adds it.

### Workstreams

- Docs index audit (`keep/merge/archive/delete`).
- Example matrix cost audit (`runtime/deps/unique failure/keep`).
- Claim/support terminology audit for over-strong wording.
- Artifact hygiene checks only when tied to named failure classes.

### Acceptance criteria

- Surface-area reductions are measurable.
- Example count follows scenario coverage, not crate count.
- Docs are easier to navigate after each pass.

## Horizon 6: Promotion, parking, and retirement rules

**Outcome:** every proposal follows a lifecycle with clear decision points.

### Lifecycle states

`idea → candidate → proved → supported → consolidated/parked/retired`

### Promotion minimums

- Named user task.
- Clean-project proof.
- Guarded/compiling docs snippet.
- Tier/claim/boundary alignment.
- Owner/maintenance class.
- No simpler existing workflow covers the need.

### Retirement triggers

- Duplicate scope.
- No distinct persona.
- Runtime cost without unique failures.
- Stale docs/unpublished behavior.
- Net user confusion.

## Parking Lot

These require a new active goal or Rails lane before execution:

- More adapter crates.
- More provider-specific examples.
- More fixture families.
- More scanner-specific workflows.
- More support-tier promotions.
- More public claims.
- More release-handoff automation.

## Non-goals

This roadmap does not authorize:

- publishing crates;
- tagging releases;
- signing artifacts;
- creating GitHub releases;
- syncing code into `EffortlessMetrics/uselesskey`;
- adding new public claims;
- promoting support tiers;
- adding new fixture families/adapter crates merely because they are possible;
- adding workflow-support rows without evidence of user value.

## Suggested PR Sequence

1. Add this roadmap and cross-links from docs and `AGENTS.md`.
2. Add follow-up checklist with owners and acceptance criteria.
3. Route no-active-work `repo-contract-report` guidance here and add regression tests.
4. Add workflow-support curation audit.
5. Add proof-receipts index and cross-links.

## Metrics

| Metric | Target |
| --- | --- |
| New workflow rows without curation class | 0 |
| New examples without unique-failure rationale | 0 |
| Docs-only/index-only PR streak | ≤ 3 |
| External-adoption examples with persona mapping | 100% |
| Workflow rows with explicit boundary | 100% |
| Workflow rows with proof command and receipt | 100% |
| PRs with review status disclosed | 100% |
| No-active-work PRs linked to roadmap item | 100% |
| Parked ideas executed without lane/goal | 0 |

## Review Cadence

- Re-check roadmap status during each no-active-lane/no-active-goal cycle.
- Update acceptance criteria when a horizon transitions from planning to
  execution.
- Archive completed follow-up items with evidence links.
