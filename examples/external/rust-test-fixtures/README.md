# Rust Test Fixtures

Use this downstream-shaped example when a Rust test crate needs deterministic
valid and invalid fixtures through the `uselesskey` facade crate without
committed payload blobs.

## Copy this

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["rsa", "jwk", "token"] }
```

```rust
use uselesskey::{Factory, NegativeToken, RsaFactoryExt, RsaSpec, TokenFactoryExt, TokenSpec};

let fx = Factory::deterministic_from_str("external-rust-test-fixtures");
let issuer = fx.rsa("issuer", RsaSpec::rs256());
let token = fx.token("issuer", TokenSpec::oauth_access_token());

let valid_jwt_shape = token.value();
let alg_none = token.negative_value(NegativeToken::AlgNone);
let bad_audience = token.negative_value(NegativeToken::BadAudience);
```

## What you get

The example proves a clean Rust project can use the facade crate for:

- deterministic RSA public JWK fixtures;
- scanner-safe API-key near misses for parser tests;
- JWT-shaped positive controls;
- JWT-shaped negative fixtures for parser, header-policy, and claim-policy
  checks.

## Positive path

```text
Factory::deterministic_from_str("external-rust-test-fixtures")
  -> fx.rsa("issuer", RsaSpec::rs256())
  -> public JWK with kty=RSA and alg=RS256

Factory::deterministic_from_str("external-rust-test-fixtures")
  -> fx.token("issuer", TokenSpec::oauth_access_token())
  -> JWT-shaped value accepted by the example claim validator
```

## Negative path

```text
fx.token("api", TokenSpec::api_key())
  -> negative_value(NegativeToken::NearMissApiKey)
  -> parser rejects a token-like value that does not start with uk_test_
```

```text
fx.token("issuer", TokenSpec::oauth_access_token())
  -> negative_value(NegativeToken::AlgNone)
  -> jwt_alg_none: validator rejects unsigned-token policy

fx.token("issuer", TokenSpec::oauth_access_token())
  -> negative_value(NegativeToken::BadAudience)
  -> jwt_bad_audience: validator rejects the wrong audience
```

The example also covers `jwt_bad_segment_count` and
`jwt_malformed_base64url` so parser failures stay distinct from claim-policy
failures.

## Verify

```bash
cargo test
```

In repo-local adoption smoke, `cargo xtask external-adoption-smoke --path .`
copies this project under `target/` and patches the dependency to the current
checkout.

## Audit / receipt

This Rust crate example does not write bundle payloads by itself. For generated
CLI bundles, use:

```bash
uselesskey audit-bundle --path target/oidc-fixtures --out target/oidc-fixtures-audit
```

The installed audit output is metadata-only. It records paths, counts, profile
metadata, stable failure classes, and boundaries without copying generated token
values into reviewer packets.

## What this does not prove

- It proves a clean Rust test project can use the facade crate without leaf
  crate imports.
- It does not prove production key generation, production authorization,
  provider compatibility, or downstream verifier correctness.
- It does not give permission to commit generated secret-shaped payloads.
