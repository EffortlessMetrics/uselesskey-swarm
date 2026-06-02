# TLS Chain Validation Fixtures

Use this downstream-shaped example when a TLS verifier or adapter test needs
deterministic certificate-chain fixtures and rustls config construction.

## Copy this

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
```

## What you get

The example proves a clean Rust project can use:

- the `uselesskey` facade crate for deterministic X.509 chain fixtures;
- the `uselesskey-rustls` adapter for server and client config construction;
- generated PEM material at test runtime without committed fixture payloads.

## Positive path

```text
Factory::deterministic_from_str("external-tls-chain-validation")
  -> fx.x509_chain("service", ChainSpec::new("valid.tls.uselesskey.test"))
  -> PEM chain plus rustls server/client config construction
```

## Negative path

Use the installed CLI `tls` bundle profile when a downstream test needs
file-based negative fixtures:

```text
negative-expired-leaf.pem    -> expired leaf rejection
negative-not-yet-valid.pem   -> not-yet-valid rejection
negative-wrong-hostname.pem  -> hostname mismatch rejection
negative-untrusted-root.pem  -> untrusted-root rejection
```

## Verify

```bash
cargo test
```

In repo-local adoption smoke, `cargo xtask external-adoption-smoke --path .`
copies this project under `target/` and patches the dependencies to the current
checkout.

## Audit / receipt

For generated CLI bundles, use:

```bash
uselesskey bundle --profile tls --out target/uselesskey-tls
uselesskey verify-bundle target/uselesskey-tls
uselesskey inspect-bundle target/uselesskey-tls
uselesskey audit-bundle target/uselesskey-tls --out target/uselesskey-tls-audit
```

The installed audit output is metadata-only. It records paths, counts, profile
metadata, fixture posture, and boundaries without copying PEM private keys or
generated certificate payloads into reviewer packets.

## What this does not prove

- It proves fixture and adapter construction for test code.
- It does not prove production PKI, revocation, certificate transparency,
  mTLS, browser trust-store behavior, production CA custody, release readiness,
  or downstream verifier correctness.
