# Start Here

Use this page when you know the test problem and want the shortest path to a
working fixture.

`uselesskey` is a test-fixture factory. It is not production key generation,
certificate management, provider compatibility certification, scanner evasion,
or cryptographic assurance.

## Pick Your Job

| I need to... | Copy this | Full workflow |
| --- | --- | --- |
| write Rust tests | `uselesskey = { version = "0.9.1", features = ["rsa", "jwk"] }` | [choose-features.md](choose-features.md) |
| test webhook signatures | `uselesskey bundle --profile webhook --out target/uselesskey-webhook` | [test-webhook-signature-validation.md](test-webhook-signature-validation.md) |
| test TLS chains | `uselesskey bundle --profile tls --out target/uselesskey-tls` | [test-tls-chain-validation.md](test-tls-chain-validation.md) |
| test OIDC/JWKS validation | `uselesskey bundle --profile oidc --out target/uselesskey-oidc` | [test-oidc-jwks-validation.md](test-oidc-jwks-validation.md) |
| test JWT negative validation | `uselesskey = { version = "0.9.1", default-features = false, features = ["token"] }` | [test-jwt-negative-validation.md](test-jwt-negative-validation.md) |
| test token-only Rust code | `uselesskey = { version = "0.9.1", default-features = false, features = ["token"] }` | [choose-features.md](choose-features.md) |
| install CI fixtures | `uselesskey bundle --profile scanner-safe --out target/uselesskey-scanner-safe` | [verify-a-fixture-bundle.md](verify-a-fixture-bundle.md) |
| test negative verifier paths | `uselesskey bundle --profile oidc --out target/uselesskey-oidc` | [test-oidc-jwks-validation.md](test-oidc-jwks-validation.md) |
| collect review evidence | `uselesskey audit-bundle target/uselesskey-webhook --ci --out target/uselesskey-webhook-audit` | [verify-a-fixture-bundle.md](verify-a-fixture-bundle.md) |
| keep runtime material disposable | `uselesskey bundle --profile tls --out target/uselesskey-tls` | [test-tls-chain-validation.md](test-tls-chain-validation.md) |
| fail CI on installed bundle drift | `uselesskey audit-bundle target/uselesskey-webhook --ci --expect-profile webhook --policy strict --out target/uselesskey-webhook-audit` | [use-uselesskey-in-downstream-ci.md](use-uselesskey-in-downstream-ci.md) |
| upload metadata-only receipts | `uses: actions/upload-artifact@v7` | [use-uselesskey-in-github-actions.md#upload-audit-receipts](use-uselesskey-in-github-actions.md#upload-audit-receipts) |
| prove public claims from a repo checkout | `cargo xtask verification-pack --out target/uselesskey-verification` | [verify-uselesskey-public-claims.md](verify-uselesskey-public-claims.md) |

For clean downstream project examples, use
[`../../examples/external/README.md`](../../examples/external/README.md).

## Install the CLI

Use the installed CLI outside this workspace:

```bash
cargo install uselesskey-cli --version 0.9.1 --locked
uselesskey doctor
uselesskey profiles
uselesskey bundle --profile webhook --explain
```

Run `doctor` once after install or before a CI rollout. It checks installed CLI
concerns only: version reporting, profile discovery, JSON output, and a writable
`target/` probe path. It does not generate, inspect, or copy fixture payloads.

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
uselesskey verify-bundle target/uselesskey-webhook
uselesskey inspect-bundle target/uselesskey-webhook
uselesskey audit-bundle target/uselesskey-webhook --ci --expect-profile webhook --policy strict --out target/uselesskey-webhook-audit
```

For GitHub Actions, upload the audit receipts and not the generated fixture
payloads:

```yaml
- name: Upload uselesskey audit receipts
  uses: actions/upload-artifact@v7
  if: always()
  with:
    name: uselesskey-webhook-audit
    path: |
      target/uselesskey-webhook-audit/bundle-audit.json
      target/uselesskey-webhook-audit/bundle-audit.md
    if-no-files-found: error
```

Attach `bundle-audit.md` and `bundle-audit.json` when a reviewer needs local
bundle metadata. The audit proves local bundle consistency and metadata
classification only. It does not prove broader repo public claims by itself,
release readiness, provider compatibility, production security, or downstream
verifier correctness.

Keep generated payloads under `target/`. Commit metadata, docs, and policy when
needed, not generated secret-shaped payloads.

## Reviewer Proof

For public-claim evidence from a repo checkout:

```bash
cargo xtask verification-pack --out target/uselesskey-verification
```

`verification-pack` runs the relevant claim-proof handlers and copies the
receipts into the review pack. Run `cargo xtask claim-proof --claim <id>` only
when you need standalone claim-proof receipts outside the pack.

Do not attach generated PEM, DER, JWT, key, Kubernetes Secret, Vault payload, or
webhook request files to reviewer packets.

For the workflow-to-claim map, see
[`../status/workflow-support.md`](../status/workflow-support.md).
