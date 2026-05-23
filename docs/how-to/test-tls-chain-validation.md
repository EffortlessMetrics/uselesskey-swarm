# Test TLS Chain Validation

Use this guide when a downstream TLS verifier, hostname check, or adapter test
needs deterministic certificate-chain fixtures and standard rejection paths.

## Copy this

For a clean Rust test project:

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["x509"] }
uselesskey-rustls = { version = "0.9.1", features = ["tls-config", "rustls-ring"] }
```

```rust
use uselesskey::{ChainSpec, Factory, X509FactoryExt};
use uselesskey_rustls::{RustlsClientConfigExt, RustlsServerConfigExt};

let fx = Factory::deterministic_from_str("external-tls-chain-validation");
let chain = fx.x509_chain(
    "service",
    ChainSpec::new("valid.tls.uselesskey.test"),
);

let server = chain.server_config_rustls();
let client = chain.client_config_rustls();

assert!(server.alpn_protocols.is_empty());
assert!(client.alpn_protocols.is_empty());
```

For file-based CI fixtures:

```bash
uselesskey bundle --profile tls --out target/tls-fixtures
uselesskey verify-bundle target/tls-fixtures
uselesskey audit-bundle target/tls-fixtures --ci --expect-profile tls --policy strict --out target/tls-fixtures-audit
```

## What you get

The Rust path gives deterministic certificate-chain material and adapter config
construction through `uselesskey-rustls`.

The installed CLI bundle writes:

| File | Stable class | Intended failure |
| --- | --- | --- |
| `certs/valid-leaf.pem` | positive control | verifier accepts the leaf for the expected hostname |
| `certs/valid-chain.pem` | positive control | verifier builds leaf -> intermediate -> root |
| `certs/negative-expired-leaf.pem` | `x509_expired_leaf` | verifier rejects `notAfter` in the past |
| `certs/negative-not-yet-valid.pem` | `x509_not_yet_valid_leaf` | verifier rejects `notBefore` in the future |
| `certs/negative-wrong-hostname.pem` | `x509_wrong_hostname` | verifier rejects SAN/CN mismatch |
| `certs/negative-untrusted-root.pem` | `x509_untrusted_root` | verifier rejects unknown issuer or trust anchor |

The bundle also writes `manifest.json`, `evidence/tls-profile.md`, and metadata
receipts under `receipts/`.

## Positive path

Use `valid.tls.uselesskey.test` as the expected hostname. Configure the
downstream verifier with the bundle root as a trust anchor and validate
`certs/valid-chain.pem`.

The external example in `examples/external/tls-chain-validation` proves a clean
Rust project can build the chain and construct rustls server/client configs
without committing fixture payloads.

## Negative path

Use the installed bundle for file-based negative tests:

| Fixture or API surface | Stable class | Expected downstream rejection |
| --- | --- | --- |
| `negative-expired-leaf.pem` | `x509_expired_leaf` | date-bounds check rejects expired leaf |
| `negative-not-yet-valid.pem` | `x509_not_yet_valid_leaf` | date-bounds check rejects future leaf |
| `negative-wrong-hostname.pem` | `x509_wrong_hostname` | hostname verifier rejects `wrong.tls.uselesskey.test` when validating `valid.tls.uselesskey.test` |
| `negative-untrusted-root.pem` | `x509_untrusted_root` | chain builder rejects a leaf outside the configured trust root |
| `ChainNegative::RevokedLeaf` | `x509_revoked_leaf` | revocation-aware tests reject a deterministic revoked leaf fixture |
| `X509Negative::WrongKeyUsage` | `x509_invalid_key_usage` | verifier policy rejects a certificate missing required usage bits |

Keep these assertions specific. A useful test distinguishes hostname mismatch
from an unknown issuer, and a date-bound failure from a parse failure.

The current installed TLS bundle exposes the first four negative classes; the
revocation and key-usage classes are implemented Rust surfaces for tests that
need those branches.

## Verify

For the clean-project facade/adapter example:

```bash
cargo test --manifest-path examples/external/tls-chain-validation/Cargo.toml
```

For the installed bundle path:

```bash
uselesskey verify-bundle target/tls-fixtures
uselesskey inspect-bundle target/tls-fixtures
```

Repo-local proof for this workflow:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
cargo xtask external-adoption-smoke --path .
cargo xtask no-blob
```

## Audit / receipt

Write a metadata-only reviewer packet with the strict CI preset:

```bash
uselesskey audit-bundle \
  target/tls-fixtures \
  --ci \
  --expect-profile tls \
  --policy strict \
  --out target/tls-fixtures-audit
```

Attach:

```text
target/tls-fixtures-audit/bundle-audit.json
target/tls-fixtures-audit/bundle-audit.md
```

The audit receipt records paths, counts, profile metadata, fixture posture, and
boundaries. It must not copy PEM private keys or generated certificate payloads
into reviewer packets.

## What this does not prove

- It does not prove production PKI, browser trust-store behavior, or pinning.
- It does not prove revocation, OCSP, OCSP stapling, CRL consumption, or CT log
  behavior.
- It does not prove ALPN, SNI routing, cipher suite negotiation, or mTLS
  client-cert chains.
- It does not prove production CA custody or production crypto guarantees.
- It does not replace adapter-specific tests when native downstream types
  matter.

## See also

- [`test-oidc-jwks-validation.md`](test-oidc-jwks-validation.md)
- [`../release/v0.8.0-tls-profile-design.md`](../release/v0.8.0-tls-profile-design.md)
- [`../specs/USELESSKEY-SPEC-0021-material-classification.md`](../specs/USELESSKEY-SPEC-0021-material-classification.md)
