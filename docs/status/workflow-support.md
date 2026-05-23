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
| Scanner-safe bundle handoff | stable bundle workflow | `scanner-safe-fixtures` | `docs/how-to/generate-scanner-safe-k8s-secret.md`, `docs/how-to/export-vault-kv-fixtures.md`, `docs/how-to/downstream-fixture-policy.md` | `cargo xtask scanner-safe-reference --check`; `cargo xtask no-blob`; `cargo xtask external-adoption-smoke --path .` | `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Scanner-safe does not mean scanner evasion, permission to commit runtime material, or production key management. |
| OIDC/JWKS validation | stable contract pack | `oidc-jwks-contract-pack` | `docs/how-to/test-oidc-jwks-validation.md`, `examples/external/oidc-jwks-validation/README.md` | `cargo xtask bundle-proof --profile oidc --out target/release-evidence/oidc`; `cargo xtask external-adoption-smoke --path .`; `cargo xtask check-negative-fixtures` | `target/release-evidence/oidc/oidc-contract-pack-proof.json`, `target/external-adoption-smoke/report.json` | Proves generated JWKS/token-shape fixture paths, not provider compatibility or downstream verifier correctness. |
| JWT/token negative validation | stabilizing facade workflow | `jwt-token-negative-fixtures` | `docs/how-to/test-jwt-negative-validation.md`, `examples/external/rust-test-fixtures/README.md` | `cargo test -p uselesskey-token --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples`; `cargo xtask check-negative-fixtures` | `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves deterministic token-shaped positives/negatives, not production authorization, provider compatibility, signature assurance, or downstream verifier correctness. |
| Webhook verifier fixtures | stable contract pack | `webhook-contract-pack` | `docs/how-to/test-webhook-signature-validation.md`, `examples/external/webhook-verifier/README.md` | `cargo xtask bundle-proof --profile webhook --out target/release-evidence/webhook`; `cargo xtask external-adoption-smoke --path .`; `cargo xtask no-blob` | `target/release-evidence/webhook/webhook-contract-pack-proof.json`, `target/external-adoption-smoke/report.json` | Proves deterministic HMAC verifier fixtures, not provider compatibility, delivery behavior, production secret custody, or downstream verifier correctness. |
| TLS chain validation | stable contract pack | `tls-contract-pack` | `docs/how-to/test-tls-chain-validation.md`, `examples/external/tls-chain-validation/README.md` | `cargo xtask bundle-proof --profile tls --out target/release-evidence/tls`; `cargo xtask external-adoption-smoke --path .`; `cargo xtask no-blob` | `target/release-evidence/tls/tls-contract-pack-proof.json`, `target/external-adoption-smoke/report.json` | Proves generated TLS fixture paths, not mTLS, revocation, browser trust stores, production CA custody, or downstream verifier correctness. |
| WebAuthn ceremony validation | stable facade workflow | `public-crate-surface-cleanup` | `docs/how-to/test-webauthn-validation.md`, `examples/external/webauthn-ceremony-validation/README.md` | `cargo xtask public-surface`; `cargo test -p uselesskey-webauthn --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples` | `target/xtask/public-surface/latest.json`, `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves the public WebAuthn fixture crate surface and downstream-style ceremony example compile and run, not production authentication security, authenticator behavior, FIDO Metadata Service compatibility, browser behavior, release readiness, or downstream verifier correctness. |
| PKCS#11 mock validation | stable facade workflow | `public-crate-surface-cleanup` | `docs/how-to/use-pkcs11-mock-fixtures.md`, `examples/external/pkcs11-mock-validation/README.md` | `cargo xtask public-surface`; `cargo test -p uselesskey-pkcs11-mock --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples` | `target/xtask/public-surface/latest.json`, `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves the public PKCS#11 mock fixture crate surface and downstream-style HSM-shaped adapter example compile and run, not a real cryptoki implementation, C ABI, FIPS validation, production HSM behavior, provider compatibility, release readiness, or production signing security. |
| SSH fixture validation | stable facade workflow | `public-crate-surface-cleanup` | `docs/how-to/test-ssh-fixtures.md`, `examples/external/ssh-fixture-validation/README.md` | `cargo xtask public-surface`; `cargo test -p uselesskey-ssh --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples` | `target/xtask/public-surface/latest.json`, `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves the public SSH fixture crate surface and downstream-style OpenSSH key/certificate example compile and run, not OpenSSH daemon/client policy, production key custody, SSH CA operations, host authorization, release readiness, downstream verifier correctness, or production security. |
| PGP fixture validation | stable facade workflow | `public-crate-surface-cleanup` | `docs/how-to/test-pgp-fixtures.md`, `examples/external/pgp-fixture-validation/README.md` | `cargo xtask public-surface`; `cargo test -p uselesskey-pgp --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples` | `target/xtask/public-surface/latest.json`, `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves the public PGP fixture crate surface and downstream-style armored/binary OpenPGP parser example compile and run, not production PGP key custody, Web of Trust policy, OpenPGP provider compatibility, release readiness, downstream verifier correctness, or production security. |
| HMAC signature validation | stable facade workflow | `public-crate-surface-cleanup` | `docs/how-to/test-hmac-signature-validation.md`, `examples/external/hmac-signature-validation/README.md` | `cargo xtask public-surface`; `cargo test -p uselesskey-hmac --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples` | `target/xtask/public-surface/latest.json`, `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves the public HMAC fixture crate surface and downstream-style shared-secret policy example compile and run, not production HMAC implementation correctness, production secret custody, provider compatibility, webhook or JWT contract-pack completeness, release readiness, downstream verifier correctness, or production security. |
| Entropy byte fixtures | stable facade workflow | `public-crate-surface-cleanup` | `docs/how-to/test-entropy-byte-fixtures.md`, `examples/external/entropy-byte-fixtures/README.md` | `cargo xtask public-surface`; `cargo test -p uselesskey-entropy --all-features`; `cargo xtask external-adoption-smoke --path . --library-examples` | `target/xtask/public-surface/latest.json`, `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves the public entropy fixture crate surface and downstream-style deterministic byte example compile and run, not production randomness, key derivation, token minting, scanner-policy approval, release readiness, downstream verifier correctness, or production security. |
| Installed bundle audit | stabilizing installed CLI workflow | `metadata-only-audit-packets` | `docs/how-to/share-installed-bundle-audit.md`, `docs/how-to/use-uselesskey-in-downstream-ci.md` | `cargo xtask external-adoption-smoke --path .`; `cargo xtask adoption-regression --external`; `cargo test -p uselesskey-cli --all-features audit_bundle`; `cargo xtask no-blob` | `target/external-adoption-smoke/report.json`, `target/adoption-regression/adoption-regression.json`, `target/uselesskey-audit/bundle-audit.json`, `target/uselesskey-audit/bundle-audit.md` | Binds the metadata-only packet claim to the installed CLI workflow; a downstream receipt proves only local bundle consistency and metadata classification, not release readiness or unrelated public claims. |
| Downstream policy pack recipes | stabilizing installed CLI workflow | `metadata-only-audit-packets` | `docs/how-to/use-downstream-policy-pack.md`, `examples/external/ci-recipes/README.md` | `cargo test -p uselesskey-cli --all-features audit_bundle`; `cargo xtask external-adoption-smoke --path . --ci-recipes --format json`; `cargo xtask no-blob` | `target/external-adoption-smoke/report.json`, `target/external-adoption-smoke/report.md` | Proves copyable downstream CI recipe wiring and metadata-only audit packet generation, not release readiness, scanner-policy approval, provider compatibility, or downstream verifier correctness. |

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
