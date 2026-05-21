# Test Webhook Signature Validation

Use this guide when a webhook consumer needs deterministic signed requests and
realistic rejection cases for GitHub-style, Stripe-style, or Slack-style HMAC
verification.

## Copy this

For Rust tests:

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["webhook"] }
```

```rust
use uselesskey::{Factory, NearMissScenario, WebhookFactoryExt, WebhookPayloadSpec};

let fx = Factory::deterministic_from_str("webhook-signature-tests");
let fixture = fx.webhook_stripe("payment", WebhookPayloadSpec::Canonical);

let stale = fixture.near_miss_stale_timestamp(300);
let wrong_secret = fixture.near_miss_wrong_secret();
let tampered = fixture.near_miss_tampered_payload();

assert_eq!(stale.scenario, NearMissScenario::StaleTimestamp);
assert_eq!(wrong_secret.scenario, NearMissScenario::WrongSecret);
assert_eq!(tampered.scenario, NearMissScenario::TamperedPayload);
```

For file-based CI fixtures:

```bash
uselesskey bundle --profile webhook --out target/webhook-fixtures
uselesskey verify-bundle --path target/webhook-fixtures
uselesskey audit-bundle --path target/webhook-fixtures --summary
```

## What you get

The Rust path gives a signed request fixture with:

- provider-shaped headers such as `Stripe-Signature`,
  `X-Hub-Signature-256`, or `X-Slack-Signature`;
- canonical or raw payload bytes;
- a deterministic fixture secret;
- near-miss request fixtures for freshness, secret, and body mismatch paths.

The installed CLI bundle writes:

| File | Stable class | Intended failure |
| --- | --- | --- |
| `requests/valid.json` | positive control | verifier accepts the provider base string and HMAC |
| `requests/negative-tampered-body.json` | `webhook_tampered_body` | verifier rejects a body that no longer matches signed bytes |
| `requests/negative-wrong-secret.json` | `webhook_wrong_secret` | verifier rejects a signature made with another secret |
| `requests/negative-stale-timestamp.json` | `webhook_stale_timestamp` | verifier rejects a timestamp outside tolerance |
| `requests/negative-missing-signature.json` | `webhook_missing_signature` | verifier rejects a missing signature header |
| `requests/negative-malformed-signature.json` | `webhook_malformed_signature` | verifier rejects an unparsable signature header |

The bundle also writes `manifest.json`, `evidence/webhook-profile.md`, and
metadata receipts under `receipts/`.

## Positive path

A positive webhook test reconstructs the provider base string, recomputes
HMAC-SHA256 with the fixture secret, and compares it with the fixture header.

| Profile | Header | Base string |
| --- | --- | --- |
| GitHub | `X-Hub-Signature-256: sha256=<hex>` | `body` |
| Stripe | `Stripe-Signature: t=<ts>,v1=<hex>` | `format!("{ts}.{body}")` |
| Slack | `X-Slack-Signature: v0=<hex>` and `X-Slack-Request-Timestamp` | `format!("v0:{ts}:{body}")` |

The external example in `examples/external/webhook-verifier` proves a clean
Rust project can generate the Stripe-shaped positive fixture through the facade
crate.

## Negative path

Use the near-miss constructors when a Rust test should assert a specific
verifier rejection:

| Constructor | Stable class | Expected downstream rejection |
| --- | --- | --- |
| `near_miss_stale_timestamp(300)` | `webhook_stale_timestamp` | freshness window rejects before signature acceptance |
| `near_miss_wrong_secret()` | `webhook_wrong_secret` | signature compare rejects unknown secret |
| `near_miss_tampered_payload()` | `webhook_tampered_body` | signature compare rejects changed body bytes |

Use the installed bundle when the test needs JSON request fixtures and
metadata-only audit receipts.

## Verify

For the clean-project facade example:

```bash
cargo test --manifest-path examples/external/webhook-verifier/Cargo.toml
```

For the installed bundle path:

```bash
uselesskey verify-bundle --path target/webhook-fixtures
uselesskey inspect-bundle --path target/webhook-fixtures
```

Repo-local proof for this workflow:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
cargo xtask external-adoption-smoke --path .
cargo xtask no-blob
```

## Audit / receipt

Write a metadata-only reviewer packet:

```bash
uselesskey audit-bundle \
  --path target/webhook-fixtures \
  --out target/webhook-fixtures-audit \
  --ci
```

Attach:

```text
target/webhook-fixtures-audit/bundle-audit.json
target/webhook-fixtures-audit/bundle-audit.md
```

The audit receipt records paths, counts, profile metadata, stable failure
classes, and boundaries. It must not copy webhook request bodies, secrets, or
signature headers into reviewer packets.

## What this does not prove

- It does not prove production provider compatibility.
- It does not prove webhook delivery, retry, or back-off behavior.
- It does not prove replay protection beyond timestamp tolerance checks.
- It does not prove production secret custody or rotation policy.
- It does not prove transport security.

## See also

- [`../../crates/uselesskey-webhook/README.md`](../../crates/uselesskey-webhook/README.md)
- [`../specs/USELESSKEY-SPEC-0011-webhook-contract-pack.md`](../specs/USELESSKEY-SPEC-0011-webhook-contract-pack.md)
- [`test-jwt-negative-validation.md`](test-jwt-negative-validation.md)
