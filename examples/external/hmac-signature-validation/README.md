# HMAC Signature Validation Example

Use this example when downstream tests need deterministic shared-secret
material for code paths that branch on HS256, HS384, or HS512 and reject
tampered messages, wrong secrets, or wrong algorithms.

```toml
[dev-dependencies]
sha2 = "0.11.0"
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-hmac = "0.9.1"
```

The test crate covers:

- HS256, HS384, and HS512 secret sizes and algorithm labels;
- deterministic shared-secret generation for verifier-shaped tests;
- positive signature-policy acceptance;
- tampered-message, wrong-secret, and wrong-algorithm rejection;
- debug output that omits generated secret bytes.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

The digest helper in this example is a small downstream policy harness, not a
production HMAC implementation. In application code, pass
`secret.secret_bytes()` to the HMAC, JWT, or webhook library your verifier
already uses. These fixtures do not prove production secret custody, provider
compatibility, webhook or JWT contract-pack completeness, release readiness,
downstream verifier correctness, or production security.
