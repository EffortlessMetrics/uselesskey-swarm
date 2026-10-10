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

Focused proof for the changed behavior:

```bash
```

Final exact-head development proof:

```bash
cargo xtask pr
python3 scripts/seal-development-proof.py
```

Receipt: `target/xtask/receipt.json`

```toml
head = "0000000000000000000000000000000000000000"
proof_head = "0000000000000000000000000000000000000000"
proof_receipt_sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
```

- [ ] receipt is bound to the exact final head
- [ ] no failed or unknown proof result remains
- [ ] every skipped result is appropriate and explained
- [ ] substantive review is complete
- [ ] known blockers are empty

## Claim boundary

What this PR proves, and what it does not prove.

## Rollback

How to revert safely.
