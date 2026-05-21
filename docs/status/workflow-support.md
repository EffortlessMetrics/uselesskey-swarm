# Workflow Support

Source-of-truth artifact id: `USELESSKEY-STATUS-workflow-support`

This page maps task-first workflows to public claims, support tiers, proof
commands, receipts, and boundaries. It is the handoff between
[`PUBLIC_CLAIMS.md`](PUBLIC_CLAIMS.md), the generated
[`support-matrix.md`](../reference/support-matrix.md), and the how-to pages a
user copies from.

Rule:

```text
Workflow docs are user promises only when they bind to a claim, a support tier,
a proof command, and a boundary.
```

## Workflow Matrix

| Workflow | Support tier | Public claim | Primary docs | Proof commands | Receipts | Boundary |
| --- | --- | --- | --- | --- | --- | --- |
| Scanner-safe bundle handoff | stable bundle workflow | `scanner-safe-fixtures` | `docs/how-to/generate-scanner-safe-k8s-secret.md`, `docs/how-to/export-vault-kv-fixtures.md`, `docs/how-to/downstream-fixture-policy.md` | `cargo xtask scanner-safe-reference --check`; `cargo xtask no-blob`; `cargo xtask external-adoption-smoke --path .` | `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Scanner-safe does not mean scanner evasion or permission to commit runtime material. |
| OIDC/JWKS validation | stable contract pack | `oidc-jwks-contract-pack` | `docs/how-to/test-oidc-jwks-validation.md`, `examples/external/oidc-jwks-validation/README.md` | `cargo xtask bundle-proof --profile oidc --out target/release-evidence/oidc`; `cargo xtask external-adoption-smoke --path .`; `cargo xtask check-negative-fixtures` | `target/release-evidence/oidc/oidc-contract-pack-proof.json`, `target/external-adoption-smoke/report.json` | Proves generated JWKS/token-shape fixture paths, not provider compatibility or downstream verifier correctness. |
| JWT/token negative validation | stabilizing facade workflow | `jwt-token-negative-fixtures` | `docs/how-to/test-jwt-negative-validation.md`, `examples/external/rust-test-fixtures/README.md` | `cargo test -p uselesskey-token --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples`; `cargo xtask check-negative-fixtures` | `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves deterministic token-shaped positives/negatives, not production authorization or signature assurance. |
| Webhook verifier fixtures | stable contract pack | `webhook-contract-pack` | `docs/how-to/test-webhook-signature-validation.md`, `examples/external/webhook-verifier/README.md` | `cargo xtask bundle-proof --profile webhook --out target/release-evidence/webhook`; `cargo xtask external-adoption-smoke --path .`; `cargo xtask no-blob` | `target/release-evidence/webhook/webhook-contract-pack-proof.json`, `target/external-adoption-smoke/report.json` | Proves deterministic HMAC verifier fixtures, not provider compatibility, delivery behavior, or production secret custody. |
| TLS chain validation | stable contract pack | `tls-contract-pack` | `docs/how-to/test-tls-chain-validation.md`, `examples/external/tls-chain-validation/README.md` | `cargo xtask bundle-proof --profile tls --out target/release-evidence/tls`; `cargo xtask external-adoption-smoke --path .`; `cargo xtask no-blob` | `target/release-evidence/tls/tls-contract-pack-proof.json`, `target/external-adoption-smoke/report.json` | Proves generated TLS fixture paths, not mTLS, revocation, browser trust stores, or production CA custody. |
| Installed bundle audit | stabilizing installed CLI workflow | `metadata-only-audit-packets` | `docs/how-to/share-installed-bundle-audit.md`, `docs/how-to/use-uselesskey-in-downstream-ci.md` | `cargo xtask external-adoption-smoke --path .`; `cargo xtask adoption-regression --external`; `cargo test -p uselesskey-cli --all-features audit_bundle`; `cargo xtask no-blob` | `target/external-adoption-smoke/report.json`, `target/adoption-regression/adoption-regression.json`, `target/uselesskey-audit/bundle-audit.json` | Binds the metadata-only packet claim to the installed CLI workflow; a downstream receipt proves only local bundle consistency and metadata classification, not release readiness or unrelated public claims. |

## Support Tier Interpretation

| Tier | Meaning |
| --- | --- |
| stable contract pack | Public bundle profile with claim-ledger proof and no-blob evidence. |
| stable facade workflow | Public crate/API path whose example is compiled by external adoption smoke. |
| stable bundle workflow | Installed CLI bundle path covered by external adoption smoke and metadata receipts. |
| stabilizing facade workflow | Public crate/API path with stable IDs and proof commands, while the support boundary is still tightening. |
| stabilizing installed CLI workflow | Installed command surface with a claim-ledger boundary and local receipts, while CLI options may still refine before 1.0. |

Support tiers here summarize user workflow posture. Crate-level semver and
publish status remain governed by
[`docs/reference/support-matrix.md`](../reference/support-matrix.md).

## Boundaries

No workflow on this page proves:

- production security;
- provider compatibility;
- scanner evasion;
- downstream verifier correctness;
- release readiness;
- crates.io publish state for unreleased `main` work.

Installed CLI commands must not execute claim-ledger command strings or shell
out to `xtask`. Repo-local proof remains under `cargo xtask`.
