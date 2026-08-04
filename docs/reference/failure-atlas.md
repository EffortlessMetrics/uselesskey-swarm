# Fixture Failure Atlas

Use this reference when choosing a negative fixture for a validator, parser, or
platform-handoff test. The goal is not random invalid data. The goal is an input
that is realistic enough to reach the downstream check and fail for the reason
the test is meant to cover.

`uselesskey` is a test-fixture layer. These fixtures do not prove production
cryptographic correctness, production key-management behavior, or scanner
evasion.

## OIDC and JWKS

| Fixture | Failure class | Expected downstream rejection | Generate with |
| --- | --- | --- | --- |
| `jwks/valid.json` | valid key-set shape | validator accepts the key set shape for positive-path tests | `uselesskey bundle --profile oidc` |
| `jwks/negative-duplicate-kid.json` | ambiguous key selection | validator refuses a JWKS where two distinct keys share one `kid` | `uselesskey bundle --profile oidc` |
| `jwks/negative-missing-kid.json` | key identification failure | validator refuses or cannot select a key without `kid` metadata | `uselesskey bundle --profile oidc` |
| `NegativeJwks::DuplicateKid` | ambiguous key selection in Rust tests | key-selection code rejects duplicate `kid` values | `uselesskey_jwk` or facade `jwk` feature |
| `NegativeJwk::MissingKid` | key identification failure in Rust tests | code requiring `kid` rejects a JWK without one | `uselesskey_jwk` or facade `jwk` feature |

The OIDC bundle profile is shape-first. Use it to test routing, key-set parsing,
manifest verification, and validator failure paths. If a downstream test must
verify real signatures, use runtime fixtures and the appropriate adapter in that
test.

## JWT and Token Shapes

| Fixture | Failure class | Expected downstream rejection | Generate with |
| --- | --- | --- | --- |
| `tokens/valid-rs256.json` | valid JWT-shaped positive control | parser accepts the JWT shape and expected metadata | `uselesskey bundle --profile oidc` |
| `tokens/negative-alg-none.json` | insecure algorithm | validator rejects `alg: none` or refuses unsigned-token policy | `uselesskey bundle --profile oidc` |
| `tokens/negative-bad-audience.json` | authorization failure | validator rejects the wrong `aud` claim | `uselesskey bundle --profile oidc` |
| `NegativeToken::AlgNone` | insecure algorithm in Rust tests | validator rejects an unsigned-algorithm header | `uselesskey-token` or facade `token` feature |
| `NegativeToken::MissingKid` | key identification failure in Rust tests | key-selection code rejects a JWT without a header `kid` | `uselesskey-token` or facade `token` feature |
| `NegativeToken::BadAudience` | authorization failure in Rust tests | validator rejects a token with the wrong audience | `uselesskey-token` or facade `token` feature |
| `NegativeToken::NearMissApiKey` | scanner-safe parser test | parser rejects a token-like value that is close to, but not, the real test prefix | `uselesskey-token` or facade `token` feature |

Token fixtures are protocol-shaped test values. They are not production JWTs and
do not make an authorization decision meaningful by themselves.

## Webhook Signatures

| Fixture | Failure class | Expected downstream rejection | Generate with |
| --- | --- | --- | --- |
| `requests/negative-wrong-secret.json` | signature verification failure | verifier rejects a signature made with another secret | `uselesskey bundle --profile webhook` |
| `requests/negative-stale-timestamp.json` | replay-window failure | verifier rejects a timestamp outside tolerance | `uselesskey bundle --profile webhook` |
| `requests/negative-malformed-signature.json` | signature parse failure | parser or verifier rejects an unparsable signature header | `uselesskey bundle --profile webhook` |
| `WebhookFixture::near_miss_signature` | digest comparison failure in Rust tests | verifier rejects a request whose signature is one hex digit off, with a valid header shape | `uselesskey-webhook` or facade `webhook` feature |
| `WebhookFixture::near_miss_malformed_canonical_payload` | canonicalization failure in Rust tests | a verifier that canonicalizes the request body before checking the digest rejects an unparsable payload | `uselesskey-webhook` or facade `webhook` feature |

The webhook bundle profile emits provider-shaped HMAC-SHA256 request fixtures.
The crate-level near-miss helpers cover digest-comparison and canonicalization
rejection branches for Rust tests. They are deterministic verifier fixtures, not
provider compatibility, replay-protection completeness, or production
secret-management proofs.

## Scanner-Safe Bundle Handoff

| Fixture or payload | Failure class | Expected downstream rejection | Generate with |
| --- | --- | --- | --- |
| scanner-safe HMAC JWK shape | non-usable symmetric material | parser handles the JWK shape without a real shared secret | `uselesskey bundle --profile scanner-safe` |
| near-miss token shape | scanner-safe parser test | parser or scanner-policy tests reject a non-real token value | `uselesskey bundle --profile scanner-safe` |
| Kubernetes Secret export | platform shape handoff | CI can load the Secret-shaped payload without committed runtime material | `uselesskey export k8s` |
| Vault KV JSON export | platform shape handoff | CI can load Vault-shaped JSON without committed runtime material | `uselesskey export vault-kv-json` |

The scanner-safe profile emits public material, invalid symmetric JWK shape data,
and near-miss token shapes. It is for parser, configuration, and platform
plumbing tests. Use `--profile runtime` only when a downstream test really needs
private or symmetric fixture material.

## Evidence Commands

Run the same evidence lane that the release uses when changing these fixtures or
docs:

```bash
cargo xtask bundle-proof --profile scanner-safe --out target/release-evidence/scanner-safe
cargo xtask bundle-proof --profile oidc --out target/release-evidence/oidc
cargo xtask no-blob
cargo xtask examples-smoke
```

For behavior changes in owner crates, add targeted evidence:

```bash
cargo xtask impacted-evidence --base origin/main
cargo xtask mutants-pr --changed
```
