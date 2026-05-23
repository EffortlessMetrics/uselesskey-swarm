# External Examples

These examples are shaped like downstream repositories. Use them when you want
copyable test, CI, and verifier wiring without learning the workspace internals
first.

## Pick A Job

| Job | Example | Proof path |
| --- | --- | --- |
| Bundle, verify, inspect, and audit in downstream CI | [downstream-ci-bundle-audit](downstream-ci-bundle-audit/) | `cargo xtask external-adoption-smoke --path .` |
| Copy GitHub Actions and regression recipes | [ci-recipes](ci-recipes/) | `cargo xtask external-adoption-smoke --path . --ci-recipes --format json` |
| Test OIDC/JWKS validation | [oidc-jwks-validation](oidc-jwks-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Use facade fixtures in Rust tests | [rust-test-fixtures](rust-test-fixtures/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test TLS chain validation | [tls-chain-validation](tls-chain-validation/) | `cargo xtask external-adoption-smoke --path . --library-examples` |
| Test webhook signature validation | [webhook-verifier](webhook-verifier/) | `cargo xtask external-adoption-smoke --path . --library-examples` |

## Review Boundary

Keep generated fixture payloads under `target/` or another ignored output
directory. Upload metadata-only receipts such as `bundle-audit.json` and
`bundle-audit.md` when reviewers need CI evidence.

These examples prove local fixture wiring and documented failure classes. They
do not prove production security, provider compatibility, scanner-policy
approval, release readiness, or downstream verifier correctness.
