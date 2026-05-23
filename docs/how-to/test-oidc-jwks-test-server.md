# I need to test OIDC/JWKS HTTP discovery

Use this guide when a downstream integration test needs local OIDC discovery
and JWKS HTTP routes, not only in-memory JWKS values.

## Copy this

Add the test-server crate next to the core factory and whichever key family the
JWKS should expose:

```toml
[dev-dependencies]
uselesskey-core = "0.9.1"
uselesskey-rsa = "0.9.1"
uselesskey-test-server = "0.9.1"
```

```rust
use uselesskey_core::Factory;
use uselesskey_rsa::RsaSpec;
use uselesskey_test_server::{JwksSpec, OidcServerSpec, OidcTestServer};

let server = OidcTestServer::start(
    Factory::deterministic_from_str("oidc-http"),
    OidcServerSpec::localhost_static(JwksSpec::single_rsa("issuer", RsaSpec::rs256())),
)
.await?;

let discovery_url = server.discovery_url();
let jwks_url = server.jwks_url();
```

Point the verifier under test at `discovery_url` or `jwks_url`, then assert the
verifier accepts the positive local route and rejects the adjacent negative
state you configure.

For a copyable downstream test crate, see
[`../../examples/external/oidc-test-server-validation/`](../../examples/external/oidc-test-server-validation/).

## What you get

`uselesskey-test-server` starts a local HTTP server with:

- deterministic OIDC discovery metadata;
- deterministic JWKS materialized from RSA, ECDSA, or Ed25519 fixture specs;
- static or phase-driven JWKS rotation;
- optional cache headers and ETags;
- route flags for enabling discovery and JWKS endpoints independently.

The server owns generated runtime material in memory. Keep any captured HTTP
responses under `target/` if a downstream test writes them to disk.

## Positive path

The normal verifier path should load discovery, follow `jwks_uri`, parse the
JWKS, select the fixture key, and accept the expected token or key policy path.
The clean-project example proves the route layer by fetching discovery and JWKS
JSON through `reqwest`.

## Rotation path

Use `JwksRotation::Sequence` and named `JwksPhase` values when a verifier needs
to exercise key refresh behavior:

```rust
use uselesskey_test_server::{IssuerUrlMode, JwksPhase, JwksRotation};

let spec = OidcServerSpec {
    issuer_url_mode: IssuerUrlMode::RandomPortLocalhost,
    jwks_rotation: JwksRotation::Sequence(vec![
        JwksPhase::new("primary", JwksSpec::single_rsa("issuer", RsaSpec::rs256())),
        JwksPhase::new("rotated", JwksSpec::single_rsa("issuer-rotated", RsaSpec::rs256())),
    ]),
    cache_headers: None,
    serve_discovery: true,
    serve_jwks: true,
};

let server = OidcTestServer::start(Factory::deterministic_from_str("oidc-rotation"), spec).await?;
server.with_phase("rotated")?;
```

The useful assertion is that the downstream verifier refreshes and reselects the
expected key, not merely that the HTTP request succeeds.

## Cache and route policy

Set `cache_headers` to test verifier behavior around `Cache-Control`, `ETag`,
and `If-None-Match`. Set `serve_discovery` or `serve_jwks` to `false` when the
application needs to cover missing-route policy branches.

## Verify

Clean-project proof from this repo:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

Crate-level proof:

```bash
cargo test -p uselesskey-test-server --all-features
```

## What this does not prove

- It does not prove production IdP behavior.
- It does not prove provider compatibility.
- It does not prove production network security.
- It does not prove production signing-key custody.
- It does not prove downstream verifier correctness.
- It does not prove release readiness.
