# Webhook Verifier Fixtures

Use this downstream-shaped example when a webhook consumer test needs
deterministic HMAC request fixtures plus realistic rejection cases.

## Copy this

```toml
[dev-dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["webhook"] }
hmac = "0.13.0-rc.6"
sha2 = "0.11"
hex = "0.4"
serde_json = "1"
```

The example keeps verification outside `uselesskey`: it reconstructs the
GitHub-, Stripe-, and Slack-style signing inputs with RustCrypto HMAC and uses
the fixture secret as the verifier's configured test secret.

## What it exercises

For each provider profile:

- the valid fixture's delivered bytes and headers verify;
- the wrong-secret near miss is checked against the original verifier secret and rejects;
- the tampered body keeps the original signed headers and rejects;
- restoring only the original signed body makes those preserved headers verify again;
- missing and malformed signature headers reject;
- Stripe and Slack stale timestamps reject under an explicit 300-second test window.

GitHub's HMAC header does not carry the fixture timestamp, so this example does
not pretend the HMAC check itself implements replay-window policy for GitHub.

It also proves two byte-ordering points that matter to real webhook consumers:

- a raw body with whitespace/newline verifies only when the exact delivered bytes are used;
- malformed JSON can still have a valid HMAC, after which application JSON parsing fails as a separate step.

## Verify

```bash
cargo test
```

In repo-local adoption smoke, `cargo xtask external-adoption-smoke --path .`
copies this project under `target/` and patches the `uselesskey` dependency to
the current checkout. Registry-version proof is a separate mode; source-path
success is not publication evidence.

## Installed bundle path

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

Library-only cases and installed-bundle cases are not automatically identical.
Keep their inventories explicit rather than claiming parity from one path.

## What this does not prove

This is a small reference verifier for deterministic test fixtures. It does not
prove every provider SDK, production secret management, complete replay
defenses, delivery/retry behavior, transport security, release readiness, or
arbitrary downstream verifier correctness.
