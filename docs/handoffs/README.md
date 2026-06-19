# Handoffs

Handoffs record what changed, what proof exists, and what the next operator
should do after a lane changes state.

Use handoffs for release gates, PR queue transitions, failed external
dependencies, and lane closeout. Do not use handoffs as the active task source;
current agent state belongs under `.uselesskey/goals/`.

Start with [agent-bootstrap.md](agent-bootstrap.md) when resuming agent work.
It defines the read order from `.rails/index.toml` to active lanes, archived
goal state, plans, specs, claim reports, and validation commands.

Use [local-validation.md](local-validation.md) when reporting local PR evidence
or deciding what `pr-lite`, hosted CI, targeted mutation, and release evidence
each prove.

## Recent Handoffs

- [2026-06-19-v0-10-release-adoption-closure-closeout.md](2026-06-19-v0-10-release-adoption-closure-closeout.md) - v0.10 release-adoption closeout and source-boundary handoff.
- [2026-05-21-v0-10-0-release-readiness-closeout.md](2026-05-21-v0-10-0-release-readiness-closeout.md) - Release-readiness closeout and next release-boundary handoff.
- [2026-05-21-source-of-truth-control-plane-closeout.md](2026-05-21-source-of-truth-control-plane-closeout.md) - Source-of-truth control-plane closeout and carried product surfaces.

## Required Shape

Use [the handoff template](../templates/handoff.md). A handoff should include:

- Current state
- Relevant PRs, issues, commits, or releases
- Proof already run
- Known blockers
- Next safe action
- Explicit non-goals
