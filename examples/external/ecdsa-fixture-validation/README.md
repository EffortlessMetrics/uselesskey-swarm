# ECDSA Fixture Validation Example

Use this example when downstream tests need ES256 or ES384 ECDSA PKCS#8
private keys and SPKI public keys for parser, policy, or key-mismatch paths
without committing generated key material.

```toml
[dev-dependencies]
p256 = { version = "0.14.0-rc.9", features = ["ecdsa", "pkcs8", "pem"] }
p384 = { version = "0.14.0-rc.9", features = ["ecdsa", "pkcs8", "pem"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-ecdsa = "0.9.1"
```

The test crate covers:

- ES256/P-256 PKCS#8 private key PEM and DER parse paths;
- ES256/P-256 SPKI public key PEM and DER parse paths;
- ES384/P-384 PKCS#8 and SPKI parse paths;
- corrupt private-key PEM rejection;
- truncated private-key DER rejection;
- mismatched public-key material for negative policy paths;
- debug output that omits generated key material.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

ECDSA fixtures are test inputs, not production signing keys, key custody,
provider compatibility, release readiness, downstream verifier correctness, or
production security. Generated PEM and DER values are secret-shaped. Keep them
in memory or under `target/`, and use metadata-only receipts when reviewers
need evidence.
