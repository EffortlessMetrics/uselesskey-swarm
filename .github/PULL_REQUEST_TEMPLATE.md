## Summary

What changed in this PR.

## Links

Proposal:
Spec:
ADR:
Rails lane:
Active goal:
Work item:
Plan item:
Issue:

## Scope

Files and behavior included.

## Non-goals

What is explicitly out of scope.

## Release/source boundary

Check one:

- [ ] no release, publish, signing, crates.io push, GitHub release, tag, or source-sync work
- [ ] release/source boundary touched and explicitly approved in linked issue/spec

## Support-tier impact

- [ ] none
- [ ] updates `docs/status/SUPPORT_TIERS.md`
- [ ] updates `policy/claim-ledger.toml`

## Policy impact

- [ ] none
- [ ] doc artifacts
- [ ] negative fixture ledger
- [ ] claim ledger
- [ ] CI lane
- [ ] package boundary
- [ ] lint / Clippy
- [ ] no-panic
- [ ] file policy

## Proof

Accepted proof-plan SHA-256 and review reference:

Exact existing ancestor base:

Focused proof commands are part of the plan and invocation envelope:

```bash
```

Final exact-head development proof:

```bash
python3 -B scripts/seal-development-proof.py --run \
  --base <full-existing-ancestor-commit> \
  --plan target/development-proof-plan.toml
```

Invocation envelope: `target/development-proof/runs/<invocation_id>/receipt.json`

Immutable envelope locator (retain the complete bytes separately from the proven candidate):

```toml
head = "0000000000000000000000000000000000000000"
proof_head = "0000000000000000000000000000000000000000"
proof_receipt_sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
```

- [ ] envelope is bound to the exact final head, tree, and existing ancestor base
- [ ] the complete invocation succeeded, including every required obligation
- [ ] focused proof and the plan's adequacy have been reviewed
- [ ] no failed or unknown proof result remains
- [ ] every skipped result is preserved and explicitly justified in the accepted plan
- [ ] immutable envelope bytes match the recorded digest
- [ ] substantive review covers the exact final head
- [ ] known blockers are empty

## Claim boundary

What this PR proves, and what it does not prove.

## Rollback

How to revert safely.
