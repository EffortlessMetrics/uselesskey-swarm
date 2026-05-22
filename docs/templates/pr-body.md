# PR Body Template

## Summary

What changed in this PR.

## Links

Proposal: `USELESSKEY-PROP-0000`
Spec: `USELESSKEY-SPEC-0000`
ADR: none
Plan item: `first-work-item`
Issue: none

## Scope

Files and behavior included.

## Non-goals

What is explicitly out of scope.

## Release/source boundary

Check one:

- [ ] no release, publish, signing, crates.io push, GitHub release, tag, or source-sync work
- [ ] release/source boundary touched and explicitly approved in linked issue/spec

## Support-Tier Impact

- [ ] none
- [ ] updates `docs/status/SUPPORT_TIERS.md`
- [ ] updates `policy/claim-ledger.toml`

## Policy Impact

- [ ] none
- [ ] doc artifacts
- [ ] negative fixture ledger
- [ ] claim ledger
- [ ] CI lane
- [ ] package boundary
- [ ] lint / Clippy
- [ ] no-panic
- [ ] file policy

## Required Evidence

```bash
cargo xtask docs-sync --check
cargo xtask typos
git diff --check
```

## Claim Boundary

What this PR proves, and what it does not prove.

## Rollback

How to revert safely.
