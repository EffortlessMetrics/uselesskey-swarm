# Rails Framework Footprint Closeout

Date: 2026-05-21
Owner: codex
Lane: RAILS-LANE-0001

## What Landed

- `.rails/` is now the durable Rails control-plane footprint.
- `.rails/index.toml` is the first read for Rails-aware humans and agents.
- Rails-owned directories exist for proposals, specs, ADRs, lanes, templates,
  and closeouts.
- `.rails/migration-status.md` maps current source-of-truth authority, Rails
  migration state, and proof rules.
- Portable Rails templates now include IDs/kinds, cross-artifact links,
  support-tier impact, policy impact, required evidence, non-goals, claim
  boundary, and rollback fields.
- `docs/rails.md` and `docs/contributing/rails.md` explain how `.rails/` fits
  beside the existing uselesskey source-of-truth stack.
- PR #56, the older `.uselesskey-spec/` namespace proposal, was closed as
  superseded after `.rails/` landed.

## Proof Executed

- PR #64: `cargo xtask docs-sync --check`
- PR #64: `cargo xtask typos`
- PR #64: `cargo xtask pr`
- PR #64: `git diff --check`
- PR #64 hosted checks: `Uselesskey Rust Small Result` green and
  `Source of Truth Advisory` green.
- PR #66: `cargo xtask docs-sync --check`
- PR #66: `cargo xtask typos`
- PR #66: TOML parse for `.rails/index.toml` and the Rails lane manifest.
- PR #66: `git diff --check`
- PR #66 hosted checks: `Uselesskey Rust Small Result` green and
  `Source of Truth Advisory` green.
- Closeout PR: `cargo xtask docs-sync --check`
- Closeout PR: `cargo xtask typos`
- Closeout PR: TOML parse for `.rails/index.toml` and the Rails lane manifest.
- Closeout PR: `git diff --check`
- Template hardening PR: `cargo xtask docs-sync --check`
- Template hardening PR: `cargo xtask typos`
- Template hardening PR: TOML parse for `.rails/templates/lane-tracker.toml`
  and `.rails/templates/policy-reference.toml`.
- Template hardening PR: `git diff --check`

## Current Boundaries

- Existing uselesskey artifacts under `docs/proposals/`, `docs/specs/`,
  `docs/adr/`, `plans/`, `.uselesskey/goals/`, `docs/status/`, and `policy/`
  remain authoritative until a later migration updates all affected checkers and
  links.
- `target/source-of-truth/` remains generated evidence, not committed source.
- `.codex/`, `.spec/`, `.claude/`, and `.jules/` remain awareness-only
  namespaces for Rails.
- This lane did not move release, publish, signing, tag, GitHub release,
  crates.io, or source-sync authority into `uselesskey-swarm`.
- This lane did not add product behavior or make production security, provider
  compatibility, or downstream verifier correctness claims.

## Follow-up Work

- Add Rails-specific checks only after the layout has carried more real work and
  the failure modes are clear.
- If existing proposals, specs, ADRs, plans, or goals are mirrored into
  `.rails/`, update `.rails/index.toml`, `.rails/migration-status.md`,
  `policy/doc-artifacts.toml`, and affected docs links in the same PR.
- Keep the next lane tied to release or user-path value unless a Rails check
  directly protects an active workflow.
