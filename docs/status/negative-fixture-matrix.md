# Negative Fixture Matrix

This page is the human mirror of
[`policy/negative-fixtures.toml`](../../policy/negative-fixtures.toml). It
answers whether a taxonomy class is available product behavior, accepted but
planned, deferred, or out of scope.

The taxonomy contract lives in
[`USELESSKEY-SPEC-0016`](../specs/USELESSKEY-SPEC-0016-negative-fixture-taxonomy.md).
Machine-readable product surfaces should branch on stable IDs, not display
labels.

The `Claim` column mirrors `policy/negative-fixtures.toml` so each stable ID
has an explicit public-claim or support boundary owner.

## Status Values

| Status | Meaning |
| --- | --- |
| `implemented` | The class has an owner crate or bundle surface, tests, docs/status mapping, and stable ID. |
| `accepted_planned` | The class is accepted by taxonomy, but not exposed as current product behavior. |
| `deferred` | The class is useful, but parked outside the current product lane. |
| `out_of_scope` | The class would overclaim provider compatibility, production security, or scanner-evasion behavior. |

## JWK / JWKS

| Stable ID | Status | Claim | Public surface | Bundle exposed | Proof |
| --- | --- | --- | --- | --- | --- |
| `jwk_missing_kid` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwk::MissingKid` | no | `cargo test -p uselesskey-jwk --all-features` |
| `jwk_wrong_kty` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwk::WrongKty` | no | `cargo test -p uselesskey-jwk --all-features` |
| `jwk_unsupported_alg` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwk::UnsupportedAlg` | no | `cargo test -p uselesskey-jwk --all-features` |
| `jwk_malformed_base64url` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwk::MalformedBase64url` | no | `cargo test -p uselesskey-jwk --all-features` |
| `jwk_mismatched_parameters` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwk::MismatchedParameters` | no | `cargo test -p uselesskey-jwk --all-features` |
| `jwks_empty_keys` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwks::EmptyKeys` | no | `cargo test -p uselesskey-jwk --all-features` |
| `jwks_missing_kid` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwks::MissingKid` | `oidc` | `cargo test -p uselesskey-cli --all-features bundle` |
| `jwks_duplicate_kid` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwks::DuplicateKid` | `oidc` | `cargo test -p uselesskey-cli --all-features bundle` |
| `jwks_duplicate_key` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwks::DuplicateKey` | no | `cargo test -p uselesskey-jwk --all-features` |
| `jwks_mixed_valid_invalid` | `implemented` | `oidc-jwks-contract-pack` | `NegativeJwks::MixedValidInvalid` | no | `cargo test -p uselesskey-jwk --all-features` |

## JWT / Token

| Stable ID | Status | Claim | Public surface | Bundle exposed | Proof |
| --- | --- | --- | --- | --- | --- |
| `jwt_bad_segment_count` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::MalformedJwtSegmentCount` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_malformed_base64url` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::BadBase64UrlSegment` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_invalid_header_shape` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::InvalidJwtHeaderShape` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_missing_alg` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::MissingAlg` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_alg_none` | `implemented` | `oidc-jwks-contract-pack` | `NegativeToken::AlgNone` | `oidc` | `cargo test -p uselesskey-cli --all-features bundle` |
| `jwt_missing_kid` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::MissingKid` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_mismatched_kid` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::MismatchedKid` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_expired` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::ExpiredClaims` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_not_yet_valid` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::NotYetValidClaims` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_bad_issuer` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::BadIssuer` | no | `cargo test -p uselesskey-token --all-features` |
| `jwt_bad_audience` | `implemented` | `oidc-jwks-contract-pack` | `NegativeToken::BadAudience` | `oidc` | `cargo test -p uselesskey-cli --all-features bundle` |
| `token_malformed_bearer` | `implemented` | `jwt-token-negative-fixtures` | `NegativeToken::MalformedBearer` | no | `cargo test -p uselesskey-token --all-features` |
| `token_near_miss` | `implemented` | `scanner-safe-fixtures` | `NegativeToken::NearMissApiKey` | `scanner-safe` | `cargo test -p uselesskey-cli --all-features bundle` |

## Webhook

| Stable ID | Status | Claim | Public surface | Bundle exposed | Proof |
| --- | --- | --- | --- | --- | --- |
| `webhook_near_miss_signature` | `accepted_planned` | `webhook-contract-pack` | none yet | no | taxonomy only |
| `webhook_tampered_body` | `implemented` | `webhook-contract-pack` | `uselesskey bundle --profile webhook` | `webhook` | `cargo test -p uselesskey-cli --all-features webhook` |
| `webhook_wrong_secret` | `implemented` | `webhook-contract-pack` | `uselesskey bundle --profile webhook` | `webhook` | `cargo test -p uselesskey-cli --all-features webhook` |
| `webhook_stale_timestamp` | `implemented` | `webhook-contract-pack` | `uselesskey bundle --profile webhook` | `webhook` | `cargo test -p uselesskey-cli --all-features webhook` |
| `webhook_missing_signature` | `implemented` | `webhook-contract-pack` | `uselesskey bundle --profile webhook` | `webhook` | `cargo test -p uselesskey-cli --all-features webhook` |
| `webhook_malformed_signature` | `implemented` | `webhook-contract-pack` | `uselesskey bundle --profile webhook` | `webhook` | `cargo test -p uselesskey-cli --all-features webhook` |
| `webhook_malformed_canonical_payload` | `accepted_planned` | `webhook-contract-pack` | none yet | no | taxonomy only |

## X.509 / TLS

| Stable ID | Status | Claim | Public surface | Bundle exposed | Proof |
| --- | --- | --- | --- | --- | --- |
| `x509_expired_leaf` | `implemented` | `tls-contract-pack` | `ChainNegative::ExpiredLeaf` | `tls` | `cargo test -p uselesskey-cli --all-features tls` |
| `x509_not_yet_valid_leaf` | `implemented` | `tls-contract-pack` | `ChainNegative::NotYetValidLeaf` | `tls` | `cargo test -p uselesskey-cli --all-features tls` |
| `x509_wrong_hostname` | `implemented` | `tls-contract-pack` | `ChainNegative::HostnameMismatch` | `tls` | `cargo test -p uselesskey-cli --all-features tls` |
| `x509_untrusted_root` | `implemented` | `tls-contract-pack` | `ChainNegative::UnknownCa` | `tls` | `cargo test -p uselesskey-cli --all-features tls` |
| `x509_revoked_leaf` | `implemented` | `tls-contract-pack` | `ChainNegative::RevokedLeaf` | no | `cargo test -p uselesskey-x509 --all-features` |
| `x509_invalid_key_usage` | `implemented` | `tls-contract-pack` | `X509Negative::WrongKeyUsage` | no | `cargo test -p uselesskey-x509 --all-features` |

## Boundary

Implemented means `uselesskey` exposes a deterministic fixture class and proof
path. It does not prove provider compatibility, production security, scanner
evasion, release readiness, or downstream verifier correctness.
