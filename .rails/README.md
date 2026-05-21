# Rails

Rails is this repository's durable source-of-truth control plane.

Open `.rails/index.toml` first. It tells humans and agents which lane is active,
where the portable templates live, and which existing repo artifacts still own
truth while the Rails footprint is introduced.

For current migration state, read `.rails/migration-status.md`.

## Artifact Jobs

Each artifact owns one kind of truth:

- proposal: why work exists;
- spec: what behavior must be true;
- ADR: durable architecture decision;
- lane: PR-sized sequence, current status, dependencies, and proof commands;
- support tier / claim ledger: what users may believe and what proves it;
- policy ledger: governed exceptions and controlled state;
- receipt: machine or command evidence;
- closeout: what landed, what remains, and what must not be claimed.

## Durable Footprint

Rails owns these durable directories:

- `.rails/proposals/`
- `.rails/specs/`
- `.rails/adr/`
- `.rails/lanes/`
- `.rails/templates/`
- `.rails/closeouts/`

Existing artifacts under `docs/specs/`, `docs/proposals/`, `docs/adr/`,
`docs/status/`, `policy/`, `plans/`, and `.uselesskey/goals/` remain valid until
a later migration lane moves or mirrors them. Do not break existing checkers to
introduce Rails.

## Awareness-only Namespaces

Rails does not own durable repo truth under:

- `.codex/`
- `.spec/`
- `.claude/`
- `.jules/`

Those directories may contain tool/runtime state, but Rails-owned decisions,
plans, proof, and closeouts belong in committed `.rails/` artifacts or in the
existing ledgers indexed from `.rails/index.toml`.
