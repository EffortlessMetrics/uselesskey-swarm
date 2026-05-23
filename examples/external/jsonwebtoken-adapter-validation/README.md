# Jsonwebtoken Adapter Validation Example

Use this clean-project example when downstream tests already use
`jsonwebtoken` and need deterministic signing and verification keys without
committing PEM files or shared secrets.

```toml
[dev-dependencies]
jsonwebtoken = { version = "10", features = ["use_pem", "rust_crypto"] }
serde = { version = "1", features = ["derive"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-hmac = "0.9.1"
uselesskey-jsonwebtoken = { version = "0.9.1", features = ["rsa", "hmac"] }
uselesskey-rsa = "0.9.1"
```

The test crate covers:

- RS256 signing and verification through `JwtKeyExt`;
- wrong RSA verifier key rejection;
- HS256 signing and verification through `JwtKeyExt`;
- wrong HMAC secret rejection;
- algorithm-policy rejection when an HS256 token is decoded as RS256.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

This example proves downstream adapter wiring and local verifier branch
coverage. It does not prove production token security, provider compatibility,
issuer policy, release readiness, or downstream verifier correctness.
