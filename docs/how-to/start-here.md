# Start Here

Use this page when you know the test problem and want the shortest path to a
working fixture.

`uselesskey` is a test-fixture factory. It is not production key generation,
certificate management, provider compatibility certification, scanner evasion,
or cryptographic assurance.

## Pick Your Job

| I need to... | Copy this | Full workflow |
| --- | --- | --- |
| use fake RSA/JWK fixtures in Rust tests | `uselesskey = { version = "0.9.1", features = ["rsa", "jwk"] }` | [choose-features.md](choose-features.md) |
| test JWT/token parser or claim rejection | `uselesskey = { version = "0.9.1", default-features = false, features = ["token"] }` | [test-jwt-negative-validation.md](test-jwt-negative-validation.md) |
| generate a scanner-safe bundle | `uselesskey bundle --profile scanner-safe --out target/uselesskey-scanner-safe` | [share-installed-bundle-audit.md](share-installed-bundle-audit.md) |
| test TLS verifier behavior | `uselesskey bundle --profile tls --out target/uselesskey-tls` | [test-tls-chain-validation.md](test-tls-chain-validation.md) |
| test OIDC/JWKS validator behavior | `uselesskey bundle --profile oidc --out target/uselesskey-oidc` | [test-oidc-jwks-validation.md](test-oidc-jwks-validation.md) |
| test webhook signature negatives | `uselesskey bundle --profile webhook --out target/uselesskey-webhook` | [test-webhook-signature-validation.md](test-webhook-signature-validation.md) |
| fail CI on installed bundle drift | `uselesskey audit-bundle --path target/uselesskey-webhook --ci --expect-profile webhook --policy strict` | [use-uselesskey-in-downstream-ci.md](use-uselesskey-in-downstream-ci.md) |
| share what an installed bundle contains | `uselesskey audit-bundle --path target/uselesskey-webhook --out target/uselesskey-webhook-audit` | [share-installed-bundle-audit.md](share-installed-bundle-audit.md) |
| prove public claims from a repo checkout | `cargo xtask verification-pack --out target/uselesskey-verification` | [verify-uselesskey-public-claims.md](verify-uselesskey-public-claims.md) |

## Install the CLI

Use the installed CLI outside this workspace:

```bash
cargo install uselesskey-cli --version 0.9.1
uselesskey profiles
uselesskey bundle --profile webhook --explain
```

Inside this workspace, maintainers may run the same CLI subcommands through
Cargo while changing the CLI itself:

```bash
cargo run -p uselesskey-cli -- bundle --profile webhook --out target/uselesskey-webhook
```

`cargo xtask` is for maintainers, repo-local reviewers, and release evidence. A
new installed user should not need it before getting a working fixture.

## Rust Test Fixtures

Add the smallest feature set that preserves your test semantics:

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", features = ["rsa", "jwk"] }
```

Then generate fixtures at runtime:

```rust
use uselesskey::{Factory, RsaFactoryExt, RsaSpec};

let fx = Factory::deterministic_from_str("my-test-seed");
let rsa = fx.rsa("issuer", RsaSpec::rs256());

let pkcs8_pem = rsa.private_key_pkcs8_pem();
let jwk = rsa.public_jwk();
```

This gives deterministic fixture material for tests without committed PEM, DER,
JWK, or token blobs. It does not prove production key generation, production
secret storage, or downstream verifier correctness.

## Bundle Fixtures

Generate, verify, inspect, and audit a bundle:

```bash
uselesskey bundle --profile webhook --out target/uselesskey-webhook
uselesskey verify-bundle --path target/uselesskey-webhook
uselesskey inspect-bundle --path target/uselesskey-webhook
uselesskey audit-bundle --path target/uselesskey-webhook --out target/uselesskey-webhook-audit
```

Attach `bundle-audit.md` and `bundle-audit.json` when a reviewer needs local
bundle metadata. The audit proves local bundle consistency and metadata
classification only. It does not prove repo public claims, release readiness,
provider compatibility, production security, or downstream verifier
correctness.

Keep generated payloads under `target/`. Commit metadata, docs, and policy when
needed, not generated secret-shaped payloads.

## Reviewer Proof

For public-claim evidence from a repo checkout:

```bash
cargo xtask verification-pack --out target/uselesskey-verification
cargo xtask claim-proof --all-stable
```

Do not attach generated PEM, DER, JWT, key, Kubernetes Secret, Vault payload, or
webhook request files to reviewer packets.

For the workflow-to-claim map, see
[`../status/workflow-support.md`](../status/workflow-support.md).
