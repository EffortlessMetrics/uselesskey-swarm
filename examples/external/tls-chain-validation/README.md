# TLS Chain Validation Fixtures

Use this downstream-shaped example when a TLS verifier or adapter test needs
deterministic certificate-chain fixtures and an actual rustls handshake.

## Copy this

```toml
[dev-dependencies]
rustls = { version = "0.23", default-features = false, features = ["std", "ring"] }
uselesskey = { version = "0.9.1", default-features = false, features = ["x509"] }
uselesskey-rustls = { version = "0.9.1", features = ["tls-config", "rustls-ring"] }
```

The example uses `uselesskey` for the fixture, `uselesskey-rustls` for config
construction, and rustls itself as the downstream consumer. That keeps fixture
generation separate from the behavior being tested.

## What it exercises

The copied project runs three bounded consumer tests:

1. a valid intermediate-signed chain completes an in-memory rustls handshake and
   transfers application data;
2. an expired-leaf chain is rejected while the client trusts the same generated
   root used by the valid control;
3. a server chain is rejected when the client trusts an independently generated
   CA, with the hostname and provider otherwise unchanged.

All material stays in memory. The example does not install a trust root, bind a
network listener, or contact an external service.

## Positive path

```text
Factory::deterministic_from_str("external-tls-chain-validation")
  -> x509_chain("service", ChainSpec::new("valid.tls.uselesskey.test"))
  -> uselesskey-rustls server/client configs with an explicit ring provider
  -> rustls ClientConnection / ServerConnection handshake
  -> application bytes delivered to the server
```

The trust root and expected DNS name are explicit test inputs rather than
ambient system state.

## Negative path

This example currently executes the two negative paths that already have matching
owner-crate rustls handshake evidence:

```text
expired leaf   -> certificate validity rejection
unknown CA     -> trust-anchor rejection
```

The installed CLI `tls` bundle also contains:

```text
negative-not-yet-valid.pem
negative-wrong-hostname.pem
```

Those files are part of the TLS contract-pack shape, but this external example
must not claim rustls rejection for them until the corresponding focused
handshake tests execute. Issue #661 tracks that remaining evidence.

## Verify

```bash
cargo test
```

In repo-local adoption smoke, `cargo xtask external-adoption-smoke --path .`
copies this project under `target/` and patches the uselesskey package
references to the current checkout. Registry-version proof is a separate mode;
source-path success is not publication evidence.

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

The certificate-oriented bundle and this in-memory identity example are related
but not interchangeable proof. Do not add private material to the bundle merely
to make it act like this test.

## What this does not prove

- production PKI or CA custody;
- revocation or certificate transparency;
- browser or operating-system trust stores;
- universal rustls crypto-provider behavior;
- the not-yet-valid or wrong-hostname rustls rejection paths until those tests
  are added and executed;
- release readiness or arbitrary downstream verifier correctness.
