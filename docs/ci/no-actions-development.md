# No-Actions Development

`uselesskey-swarm` is the internal implementation repository. GitHub issues,
branches, pull requests, comments, reviews, and merge history remain the work
control plane. GitHub Actions is not part of the ordinary development path.

The repository must be green before a pull request is merged. Removing hosted CI
does not make red work acceptable; it moves implementation proof and review
before the merge boundary.

## Development sequence

1. Start from a controlling issue with a bounded claim and proof plan.
2. Implement on a branch from current `main`.
3. Run the focused proof for the changed behavior.
4. Run the repository PR gate on the exact final head.
5. Seal the resulting receipt.
6. Complete substantive review and repair every accepted finding.
7. Merge only when proof and review are current and no known blocker remains.

The ordinary final commands are:

```bash
cargo xtask pr
python3 scripts/seal-development-proof.py
```

`cargo xtask pr` writes `target/xtask/receipt.json`. The sealing command refuses
a receipt that:

- is not schema version 2;
- is not bound to the current 40-character Git head;
- contains a failed or unknown step;
- contains a failed or unknown feature-matrix or BDD-matrix result;
- contains no successful executed step.

It prints the exact `head`, `proof_head`, and receipt SHA-256 fields required by
the canonical promotion packet.

A skipped step is not converted into a pass. It remains visible in the receipt
and must be appropriate to the selected proof. A pull request that needs proof
outside `cargo xtask pr` records the exact additional command and result in its
body before the receipt is sealed.

## Merge standard

A development pull request is mergeable only when:

- the exact final head has current focused proof;
- `cargo xtask pr` is green;
- the sealed receipt digest is recorded;
- substantive review is complete;
- unresolved review threads are terminal;
- `known_blockers` is empty;
- the claim boundary states what was and was not established.

A push after proof or review invalidates the earlier exact-head disposition.
Re-run the affected proof, reseal the receipt, and refresh review where the
change can affect prior findings.

## Promotion

One canonical promotion may compose one or more accepted development pull
requests. Each included pull request contributes a `[[development]]` entry with:

```toml
repository = "uselesskey/uselesskey-swarm"
pr = 123
head = "<exact merged development head>"
proof_head = "<same exact head>"
proof_receipt_sha256 = "<sealed receipt digest>"
```

The canonical pull request remains Draft while composition, compatibility, and
review are incomplete. True Ready means the exact canonical head is expected to
pass its first required integration run.

## Red handling

A failed local or owned proof blocks the development pull request. A failed
canonical integration proof blocks that promotion and dispatches a repair to the
actual owner. It does not authorize a bypass, an empty CI-poke commit, or a
repository-wide halt on unrelated development.

## Boundaries

This process establishes repository-owned development proof and review for the
named exact heads. It does not run release publication, crates.io pushes,
GitHub Releases, scheduled mutation, continuous coverage, or performance
evidence. Those remain explicit deep-evidence or release operations.
