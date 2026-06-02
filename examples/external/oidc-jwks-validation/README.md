# I need to test OIDC/JWKS validation

Use this clean-project example when a Rust test needs deterministic JWKS shapes
plus taxonomy-backed key-selection negatives.

## Copy this

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["rsa", "jwk"] }
```

```rust
use uselesskey::jwk::{NegativeJwk, NegativeJwks};
use uselesskey::{Factory, RsaFactoryExt, RsaSpec};

let fx = Factory::deterministic_from_str("external-oidc-jwks");
let issuer = fx.rsa("issuer", RsaSpec::rs256());
let valid = issuer.public_jwks().to_value();
let duplicate_kid = issuer.public_jwks().negative_value(NegativeJwks::DuplicateKid);
let wrong_kty = issuer.public_jwk().negative_value(NegativeJwk::WrongKty);
```

## What you get

The example builds a tiny downstream JWKS validator that accepts a valid RSA
JWKS and rejects deterministic negative shapes for:

- `jwks_empty_keys`
- `jwks_missing_kid`
- `jwks_duplicate_kid`
- `jwks_duplicate_key`
- `jwks_mixed_valid_invalid`
- `jwk_wrong_kty`
- `jwk_unsupported_alg`

## Positive path

```text
Factory::deterministic_from_str("external-oidc-jwks")
  -> fx.rsa("issuer", RsaSpec::rs256())
  -> public_jwks()
  -> validate_oidc_jwks(...) accepts the JWKS
```

## Negative path

The tests assert specific downstream rejection classes instead of merely
checking that JSON parsing fails:

```text
NegativeJwks::EmptyKeys         -> EmptyKeys
NegativeJwks::MissingKid        -> MissingKid
NegativeJwks::DuplicateKid      -> DuplicateKid
NegativeJwks::DuplicateKey      -> DuplicateKey
NegativeJwks::MixedValidInvalid -> MalformedMaterial
NegativeJwk::WrongKty           -> WrongKty
NegativeJwk::UnsupportedAlg     -> UnsupportedAlg
```

## Verify

```bash
cargo test
```

From the `uselesskey` repo, the clean-project proof is:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Audit / receipt

For the installed CLI bundle path, generate and audit the OIDC profile:

```bash
uselesskey bundle --profile oidc --out target/uselesskey-oidc
uselesskey verify-bundle target/uselesskey-oidc
uselesskey inspect-bundle target/uselesskey-oidc
uselesskey audit-bundle target/uselesskey-oidc --out target/uselesskey-oidc-audit
```

Attach only metadata receipts such as:

```text
target/uselesskey-oidc-audit/bundle-audit.json
target/uselesskey-oidc-audit/bundle-audit.md
```

## What this does not prove

- It does not prove OpenID discovery behavior.
- It does not prove production signing-key custody.
- It does not prove issuer policy.
- It does not prove provider compatibility.
- It does not prove release readiness.
- It does not prove downstream verifier correctness.
