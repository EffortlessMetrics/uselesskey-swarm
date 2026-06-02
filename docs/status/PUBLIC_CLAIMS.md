# Public Claims

Source-of-truth artifact id: `USELESSKEY-STATUS-public-claims`

This page summarizes public `uselesskey` claims and points to the command-backed
ledger in [`policy/claim-ledger.toml`](../../policy/claim-ledger.toml).

The Markdown page is for readers. The TOML ledger is the parser target for
`cargo xtask spec-check` and the source for `cargo xtask claim-report`.

Generate the current claim report:

```bash
cargo xtask claim-report
```

Check this page against the ledger:

```bash
cargo xtask claim-report --check-public-claims
```

Run a proof receipt for one claim:

```bash
cargo xtask claim-proof --claim scanner-safe-fixtures
```

Collect reviewer-facing receipts without generated fixture payloads:

```bash
cargo xtask verification-pack --out target/uselesskey-verification
```

## Claim Statuses

| Status | Meaning |
| --- | --- |
| `stable` | User-facing claim that should be proven in normal PR, docs, or release evidence when touched. |
| `release-proof` | Claim whose primary proof is an external or shipped-state release lane. |
| `advisory` | Reviewer or agent evidence that informs work but does not make a user-facing guarantee by itself. |

Claim status and support tier are separate axes. Claim status controls how
strongly a public claim may appear in README, release, and handoff wording.
Support tiers in [`SUPPORT_TIERS.md`](SUPPORT_TIERS.md) describe the maturity of
the user workflow that carries the claim. When they differ, use the stricter
claim boundary for public wording.

## Current Claims

| Claim ID | Claim | Status | Proof commands | Boundary |
| --- | --- | --- | --- | --- |
| `scanner-safe-fixtures` | Scanner-safe fixtures | `stable` | `cargo xtask scanner-safe-reference --check`; `cargo xtask bundle-proof --profile scanner-safe --out target/release-evidence/scanner-safe`; `cargo xtask no-blob`; `cargo xtask badges --check` | Scanner-safe fixture material does not mean every encoded export is safe to commit. |
| `ripr-plus-evidence-endpoint` | `ripr+` evidence endpoint | `stable` | `cargo xtask badges --check`; `cargo xtask test-efficiency-report` | `ripr+` is repo-scoped static evidence, not coverage or correctness proof. |
| `tls-contract-pack` | TLS contract pack | `stable` | `cargo xtask bundle-proof --profile tls --out target/release-evidence/tls`; `cargo xtask no-blob` | TLS fixtures prove the documented pack paths and negatives, not mTLS, revocation, CT, browser trust stores, or production CA custody. |
| `oidc-jwks-contract-pack` | OIDC/JWKS contract pack | `stable` | `cargo xtask bundle-proof --profile oidc --out target/release-evidence/oidc`; `cargo xtask no-blob` | OIDC/JWKS fixtures prove key-set and token-shape contract paths, not production signing-key custody or full OpenID provider behavior. |
| `jwt-token-negative-fixtures` | JWT/token negative fixtures | `advisory` | `cargo test -p uselesskey-token --all-features`; `cargo xtask check-negative-fixtures` | JWT/token negative fixtures define stable failure-class IDs and deterministic token-shaped inputs, not production authorization or downstream verifier correctness. |
| `webhook-contract-pack` | Webhook contract pack | `stable` | `cargo xtask bundle-proof --profile webhook --out target/release-evidence/webhook`; `cargo xtask no-blob` | Webhook fixtures prove deterministic HMAC verifier paths and negative classes, not provider compatibility, production secret management, replay protection, transport security, or downstream verifier correctness. |
| `metadata-only-audit-packets` | Metadata-only audit packets | `advisory` | `cargo test -p uselesskey-cli --all-features audit_bundle`; `cargo xtask external-adoption-smoke --path . --ci-recipes --format json`; `cargo xtask check-audit-receipts`; `cargo xtask no-blob` | Audit packets expose metadata, counts, paths, failure classes, and boundaries, not raw fixture payloads or generated secret-shaped material. |
| `bundle-manifest-schema` | Bundle manifest schema | `advisory` | `cargo xtask check-bundle-schemas` | The schema proves the documented metadata shape for generated bundle manifests, not provider compatibility or downstream verifier correctness. |
| `negative-coverage-receipt` | Negative coverage receipt | `advisory` | `cargo xtask check-negative-fixtures`; `cargo xtask check-bundle-schemas` | The receipt records stable negative fixture IDs exposed by bundle profiles, not exhaustive verifier testing or production security. |
| `public-crate-surface-cleanup` | Public crate-surface cleanup | `stable` | `cargo xtask public-surface`; `cargo xtask publish-check`; `cargo xtask publish-preflight` | The supported public crate surface is the current published contract; removed internal shims are not promised as supported public crates. |
| `external-cratesio-install-smoke` | External crates.io install smoke | `release-proof` | `cargo xtask cratesio-smoke --version 0.9.1` | Crates.io smoke proves an external install path for a published version, not every downstream feature combination. |
| `generated-badge-endpoints` | Generated badge endpoints | `stable` | `cargo xtask badges`; `cargo xtask badges --check` | Badge JSON is a generated Shields endpoint receipt, not a hand-written slogan. |
| `ripr-pr-review-evidence` | `ripr` PR review evidence | `advisory` | `cargo xtask ripr-pr --check`; `cargo xtask ripr-review-comments --check`; `cargo xtask ripr-pr-summary --check` | PR evidence is diff-scoped and advisory; it is not the repo-scoped README `ripr+` badge. |

## Reader Rule

Public claims should answer four questions:

```text
what is promised?
what command proves it?
where is the generated artifact or receipt?
what is explicitly outside the boundary?
```

If a claim cannot answer those questions, it should stay out of the README
masthead and release headline until the proof path exists.

Use `cargo xtask claim-proof --claim <claim-id>` when a reviewer needs runnable
evidence for a supported claim. Use `cargo xtask verification-pack --out <dir>`
when they need claim reports, badge endpoints, contract-pack receipts, and
claim-proof receipts in one metadata-only bundle.

## Workflow Binding

Task-first workflow pages bind back to the same claims instead of creating
parallel promises. See [`workflow-support.md`](workflow-support.md) for the
matrix that maps installed CLI and Rust-test workflows to support tiers, proof
commands, receipts, and boundaries.
