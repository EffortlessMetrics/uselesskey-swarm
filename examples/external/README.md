# External Examples

These examples are shaped like downstream repositories. Use them when you want
copyable test, CI, and verifier wiring without learning the workspace internals
first.

## Pick A Job

| Job | Example | Proof path |
| --- | --- | --- |
| Bundle, verify, inspect, and audit in downstream CI | [downstream-ci-bundle-audit](downstream-ci-bundle-audit/) | `cargo xtask external-adoption-smoke --path .` |
| Copy GitHub Actions and regression recipes | [ci-recipes](ci-recipes/) | `cargo xtask external-adoption-smoke --path . --ci-recipes --format json` |
| Test ECDSA key parsing and policy paths | [ecdsa-fixture-validation](ecdsa-fixture-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test Ed25519 key parsing and policy paths | [ed25519-fixture-validation](ed25519-fixture-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test deterministic byte fixtures | [entropy-byte-fixtures](entropy-byte-fixtures/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test OIDC/JWKS validation | [oidc-jwks-validation](oidc-jwks-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test HMAC signature validation | [hmac-signature-validation](hmac-signature-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test PGP key parsing and policy paths | [pgp-fixture-validation](pgp-fixture-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test PKCS#11 mock validation | [pkcs11-mock-validation](pkcs11-mock-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Use facade fixtures in Rust tests | [rust-test-fixtures](rust-test-fixtures/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test SSH key and certificate validation | [ssh-fixture-validation](ssh-fixture-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test TLS chain validation | [tls-chain-validation](tls-chain-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test WebAuthn ceremony validation | [webauthn-ceremony-validation](webauthn-ceremony-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test webhook signature validation | [webhook-verifier](webhook-verifier/) | `cargo xtask external-adoption-smoke --path . --library-examples` |

## Review Boundary

Keep generated fixture payloads under `target/` or another ignored output
directory. Upload metadata-only receipts such as `bundle-audit.json` and
`bundle-audit.md` when reviewers need CI evidence.

These examples prove local fixture wiring and documented failure classes. They
do not prove production security, provider compatibility, scanner-policy
approval, release readiness, or downstream verifier correctness.
