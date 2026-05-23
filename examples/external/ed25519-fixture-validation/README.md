# Ed25519 Fixture Validation Example

Use this example when downstream tests need Ed25519 PKCS#8 private keys and
SPKI public keys for parser, policy, or key-mismatch paths without committing
generated key material.

```toml
[dev-dependencies]
ed25519-dalek = { version = "2.2.0", features = ["pkcs8", "pem"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-ed25519 = "0.9.1"
```

The test crate covers:

- PKCS#8 private key PEM and DER parse paths;
- SPKI public key PEM and DER parse paths;
- corrupt private-key PEM rejection;
- truncated private-key DER rejection;
- mismatched public-key material for negative policy paths;
- debug output that omits generated key material.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

Ed25519 fixtures are test inputs, not production signing keys, key custody,
provider compatibility, release readiness, downstream verifier correctness, or
production security. Generated PEM and DER values are secret-shaped. Keep them
in memory or under `target/`, and use metadata-only receipts when reviewers
need evidence.
