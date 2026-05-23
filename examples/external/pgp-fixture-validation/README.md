# PGP Fixture Validation Example

Use this example when downstream tests need OpenPGP armored or binary key
shapes without committing generated PGP key blocks.

```toml
[dev-dependencies]
pgp = { version = "0.19.0", default-features = false }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-pgp = "0.9.1"
```

The test crate covers:

- armored private and public key parsing;
- binary private and public key parsing;
- deterministic user ID and fingerprint checks;
- mismatched public-key rejection hooks;
- corrupted armor and truncated binary rejection hooks;
- debug output that omits generated key material.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

These fixtures are parser and policy inputs for tests. They do not prove
production PGP key custody, Web of Trust policy, OpenPGP provider
compatibility, production signing or encryption security, release readiness, or
downstream verifier correctness.
