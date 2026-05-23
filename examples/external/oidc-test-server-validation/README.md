# OIDC Test Server Validation Example

Use this clean-project example when an integration test needs deterministic
OIDC discovery and JWKS HTTP routes without committing generated key material.

```toml
[dev-dependencies]
uselesskey-core = "0.9.1"
uselesskey-rsa = "0.9.1"
uselesskey-test-server = "0.9.1"
```

The test crate covers:

- discovery metadata with a local issuer URL and JWKS URI;
- deterministic RSA JWKS materialization through the HTTP route;
- phase-driven JWKS rotation with stable `kid` changes;
- cache headers, ETags, and `304 Not Modified` handling;
- route flags such as disabled discovery returning `404`.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

This example proves local test-server wiring, deterministic OIDC/JWKS response
shapes, phase switching, and cache-header behavior. It does not prove
production IdP behavior, provider compatibility, network security, release
readiness, downstream verifier correctness, or production signing-key custody.
