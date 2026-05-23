# Webhook Verifier Fixtures

Use this downstream-shaped example when a webhook consumer test needs
deterministic HMAC request fixtures plus realistic rejection cases.

## Copy this

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["webhook"] }
```

```rust
use uselesskey::{Factory, NearMissScenario, WebhookFactoryExt, WebhookPayloadSpec};

let fx = Factory::deterministic_from_str("external-webhook-verifier");
let fixture = fx.webhook_stripe("payment", WebhookPayloadSpec::Canonical);

let stale = fixture.near_miss_stale_timestamp(300);
let wrong_secret = fixture.near_miss_wrong_secret();
let tampered = fixture.near_miss_tampered_payload();
```

## What you get

The example proves a clean Rust project can use the facade crate for:

- a Stripe-shaped signed request fixture;
- a redacted `Debug` representation that does not print the fixture secret;
- stale timestamp, wrong-secret, and tampered-payload near misses.

## Positive path

```text
Factory::deterministic_from_str("external-webhook-verifier")
  -> fx.webhook_stripe("payment", WebhookPayloadSpec::Canonical)
  -> signed request with Stripe-Signature header and canonical payload
```

## Negative path

```text
near_miss_stale_timestamp(300)
  -> webhook_stale_timestamp: verifier rejects freshness window

near_miss_wrong_secret()
  -> webhook_wrong_secret: verifier rejects signature from another secret

near_miss_tampered_payload()
  -> webhook_tampered_body: verifier rejects changed body bytes
```

## Verify

```bash
cargo test
```

In repo-local adoption smoke, `cargo xtask external-adoption-smoke --path .`
copies this project under `target/` and patches the dependency to the current
checkout.

## Audit / receipt

For generated CLI bundles, use:

```bash
uselesskey bundle --profile webhook --out target/uselesskey-webhook
uselesskey verify-bundle target/uselesskey-webhook
uselesskey inspect-bundle target/uselesskey-webhook
uselesskey audit-bundle target/uselesskey-webhook --out target/uselesskey-webhook-audit
```

The installed audit output is metadata-only. It records paths, counts, profile
metadata, stable failure classes, and boundaries without copying request bodies,
fixture secrets, or signature headers into reviewer packets.

## What this does not prove

- It proves fixture generation and near-miss wiring for test code.
- It does not prove provider compatibility, production secret management,
  replay protection completeness, delivery behavior, or transport security.
