# I need to test OIDC/JWKS validation

Use this guide when a downstream OIDC or JWT validator needs deterministic JWKS
fixtures with both valid and negative key-set shapes.

## Copy this

Installed CLI bundle path:

```bash
uselesskey bundle --profile oidc --out target/oidc-fixtures
uselesskey verify-bundle target/oidc-fixtures
uselesskey inspect-bundle target/oidc-fixtures
uselesskey audit-bundle target/oidc-fixtures --ci --out target/oidc-fixtures-audit
```

Rust test dependency path:

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["rsa", "jwk"] }
```

```rust
use uselesskey::jwk::{NegativeJwk, NegativeJwks};
use uselesskey::{Factory, RsaFactoryExt, RsaSpec};

let fx = Factory::deterministic_from_str("oidc-negative");
let issuer = fx.rsa("issuer", RsaSpec::rs256());
let valid = issuer.public_jwks().to_value();
let duplicate_kid = issuer.public_jwks().negative_value(NegativeJwks::DuplicateKid);
let wrong_kty = issuer.public_jwk().negative_value(NegativeJwk::WrongKty);
```

Replace the final assertions with your downstream validator's acceptance and
rejection checks.

## What you get

The installed `oidc` profile writes:

- `jwks/valid.json`
- `jwks/negative-duplicate-kid.json`
- `jwks/negative-missing-kid.json`
- `tokens/valid-rs256.json`
- `tokens/negative-alg-none.json`
- `tokens/negative-bad-audience.json`
- `receipts/materialization.json`
- `receipts/audit-surface.json`
- `receipts/bundle-verification.json`
- `receipts/scanner-safety.json`
- `receipts/negative-coverage.json`

The Rust library path also exposes broader JWKS/JWK taxonomy classes for tests
that do not need files:

- `jwks_empty_keys`
- `jwks_duplicate_key`
- `jwks_mixed_valid_invalid`
- `jwk_wrong_kty`
- `jwk_unsupported_alg`
- `jwk_malformed_base64url`

## Positive path: JWKS accepted

Load `target/oidc-fixtures/jwks/valid.json` in the downstream validator test.
The positive case should prove that the validator can load the deterministic
fixture pack and select a key from a normal RSA JWKS shape.

In Rust tests, the equivalent positive case is:

```rust
let jwks = issuer.public_jwks().to_value();
assert!(validator_accepts(jwks));
```

## Negative path

Use taxonomy-backed negative fixtures to assert the specific rejection branch:

| Stable ID | Source | Intended rejection |
| --- | --- | --- |
| `jwks_duplicate_kid` | `jwks/negative-duplicate-kid.json` or `NegativeJwks::DuplicateKid` | ambiguous key selection |
| `jwks_missing_kid` | `jwks/negative-missing-kid.json` or `NegativeJwks::MissingKid` | key selection cannot identify the key |
| `jwks_empty_keys` | `NegativeJwks::EmptyKeys` | no usable verification key |
| `jwks_duplicate_key` | `NegativeJwks::DuplicateKey` | duplicate material policy rejection |
| `jwks_mixed_valid_invalid` | `NegativeJwks::MixedValidInvalid` | invalid key-set member rejection |
| `jwk_wrong_kty` | `NegativeJwk::WrongKty` | verifier rejects key type |
| `jwk_unsupported_alg` | `NegativeJwk::UnsupportedAlg` | verifier policy rejects algorithm |
| `jwt_alg_none` | `tokens/negative-alg-none.json` | verifier policy rejects unsigned algorithm |
| `jwt_bad_audience` | `tokens/negative-bad-audience.json` | claim validation rejects token |

The useful downstream assertion is the validator's rejection reason, not merely
that JSON parsing failed.

The OIDC bundle exposes only the token-shape negatives needed to exercise
common validator policy branches. Use
[`test-jwt-negative-validation.md`](test-jwt-negative-validation.md) for the
broader JWT/token taxonomy.

## Verify

Installed CLI verification:

```bash
uselesskey verify-bundle target/oidc-fixtures
uselesskey inspect-bundle target/oidc-fixtures
```

Clean-project Rust example proof from the repo:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

Repo-local contract-pack proof:

```bash
cargo xtask bundle-proof --profile oidc --out target/release-evidence/oidc
```

## Audit / receipt

Write metadata-only audit receipts:

```bash
uselesskey audit-bundle target/oidc-fixtures --ci --out target/oidc-fixtures-audit
```

Attach:

```text
target/oidc-fixtures-audit/bundle-audit.json
target/oidc-fixtures-audit/bundle-audit.md
```

The OIDC profile is scanner-safe by default. Keep generated files under
`target/` and share audit receipts instead of committing generated payloads.

## What this does not prove

- It does not prove OpenID discovery behavior.
- It does not prove production signing-key custody.
- It does not prove issuer policy.
- It does not prove provider compatibility.
- It does not prove cryptographic correctness.
- It does not prove production verifier correctness.
