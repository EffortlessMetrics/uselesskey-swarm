# Test JWT Negative Validation

Use this guide when a downstream token parser or validator needs
JWT-shaped valid and invalid inputs that are realistic enough to reach claim or
header policy checks.

## Copy this

For a clean Rust test project:

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["token"] }
```

```rust
use uselesskey::{Factory, NegativeToken, TokenFactoryExt, TokenSpec};

let fx = Factory::deterministic_from_str("jwt-negative");
let token = fx.token("issuer", TokenSpec::oauth_access_token());

let valid = token.value();
let alg_none = token.negative_value(NegativeToken::AlgNone);
let bad_audience = token.negative_value(NegativeToken::BadAudience);

assert_eq!(NegativeToken::AlgNone.stable_id(), "jwt_alg_none");
assert_eq!(NegativeToken::BadAudience.stable_id(), "jwt_bad_audience");
assert!(valid.split('.').count() == 3);
assert!(alg_none.split('.').count() == 3);
assert!(bad_audience.split('.').count() == 3);
```

For file-based tests, use the installed CLI OIDC profile:

```bash
uselesskey bundle --profile oidc --out target/oidc-fixtures
uselesskey verify-bundle target/oidc-fixtures
uselesskey audit-bundle target/oidc-fixtures --summary
```

## What you get

The Rust path gives deterministic values from the facade crate without writing
payloads to the repository.

The bundle path writes token-shaped files under the selected output directory:

| File | Stable class | Intended failure |
| --- | --- | --- |
| `tokens/valid-rs256.json` | positive control | parser accepts JWT shape and expected metadata |
| `tokens/negative-alg-none.json` | `jwt_alg_none` | validator rejects unsigned-token policy |
| `tokens/negative-bad-audience.json` | `jwt_bad_audience` | validator rejects the wrong `aud` claim |

Each file is JSON with metadata and a `value` field containing the token-shaped
string. Load `value` in the downstream test and assert the specific policy
failure.

## Positive path

Use `TokenSpec::oauth_access_token()` or `tokens/valid-rs256.json` as the
positive parser control. It is JWT-shaped and deterministic, so tests can check
that the validator reaches the intended claim/header path before adding negative
fixtures.

## Negative path

Use taxonomy-backed `NegativeToken` variants for realistic failure modes:

| Variant | Stable class | Expected downstream rejection |
| --- | --- | --- |
| `NegativeToken::MalformedJwtSegmentCount` | `jwt_bad_segment_count` | parser rejects the wrong number of JWT segments |
| `NegativeToken::BadBase64UrlSegment` | `jwt_malformed_base64url` | parser rejects an invalid base64url segment |
| `NegativeToken::InvalidJwtHeaderShape` | `jwt_invalid_header_shape` | parser or validator rejects a decoded header that is not an object |
| `NegativeToken::MissingAlg` | `jwt_missing_alg` | policy cannot select an allowed algorithm |
| `NegativeToken::AlgNone` | `jwt_alg_none` | policy rejects `alg: none` |
| `NegativeToken::MissingKid` | `jwt_missing_kid` | key selection cannot identify the verification key |
| `NegativeToken::MismatchedKid` | `jwt_mismatched_kid` | key-selection or policy logic rejects inconsistent key identity |
| `NegativeToken::BadAudience` | `jwt_bad_audience` | claim validation rejects the wrong audience |
| `NegativeToken::BadIssuer` | `jwt_bad_issuer` | claim validation rejects the wrong issuer |
| `NegativeToken::ExpiredClaims` | `jwt_expired` | claim validation rejects an expired token |
| `NegativeToken::NotYetValidClaims` | `jwt_not_yet_valid` | claim validation rejects a future `nbf` window |
| `NegativeToken::MalformedBearer` | `token_malformed_bearer` | bearer parser rejects malformed authorization syntax |
| `NegativeToken::NearMissApiKey` | `token_near_miss` | API-key parser rejects a scanner-safe near miss |

Keep the test assertion specific. A useful test distinguishes "rejected because
the algorithm is forbidden" from "failed to parse any token at all."

## Verify

For the clean-project facade example:

```bash
cargo test --manifest-path examples/external/rust-test-fixtures/Cargo.toml
```

For the installed bundle path:

```bash
uselesskey verify-bundle target/oidc-fixtures
uselesskey inspect-bundle target/oidc-fixtures
```

Repo-local proof for this task-doc path:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
cargo xtask external-adoption-smoke --path .
cargo xtask check-negative-fixtures
```

## Audit / receipt

For bundle users, write a metadata-only audit packet:

```bash
uselesskey audit-bundle \
  target/oidc-fixtures \
  --out target/oidc-fixtures-audit \
  --ci
```

Attach:

```text
target/oidc-fixtures-audit/bundle-audit.json
target/oidc-fixtures-audit/bundle-audit.md
```

The audit receipt records paths, counts, profile metadata, stable failure
classes, and boundaries. It must not copy token values into reviewer packets.

## What this does not prove

- It does not prove production signature validation by itself.
- It does not prove production issuer, audience, clock, or key-selection
  configuration.
- It does not prove provider compatibility.
- It does not prove cryptographic assurance.
- It does not replace adapter-specific tests such as `uselesskey-jsonwebtoken`
  when native downstream types matter.
