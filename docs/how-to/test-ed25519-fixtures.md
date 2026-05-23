# Test Ed25519 fixtures

Use this when a downstream verifier, parser, or policy layer needs Ed25519
PKCS#8 private keys and SPKI public keys without committing generated PEM or
DER blobs.

A copyable downstream-style test crate lives at
[`../../examples/external/ed25519-fixture-validation/`](../../examples/external/ed25519-fixture-validation/).

## Add the test dependencies

```toml
[dev-dependencies]
ed25519-dalek = { version = "2.2.0", features = ["pkcs8", "pem"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-ed25519 = "0.9.1"
```

Run the repo proof path with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Generate parseable key material

```rust
use ed25519_dalek::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};
use ed25519_dalek::{SigningKey, VerifyingKey};
use uselesskey_core::{Factory, Seed};
use uselesskey_ed25519::{Ed25519FactoryExt, Ed25519Spec};

let fx = Factory::deterministic(Seed::from_env_value("ed25519-test").unwrap());
let key = fx.ed25519("release-signer", Ed25519Spec::new());

let private = SigningKey::from_pkcs8_pem(key.private_key_pkcs8_pem()).unwrap();
let public = VerifyingKey::from_public_key_pem(key.public_key_spki_pem()).unwrap();

assert_eq!(key.label(), "release-signer");
assert_eq!(private.verifying_key().as_bytes(), public.as_bytes());
```

Use DER when the downstream API expects bytes:

```rust
let private = SigningKey::from_pkcs8_der(key.private_key_pkcs8_der()).unwrap();
let public = VerifyingKey::from_public_key_der(key.public_key_spki_der()).unwrap();

assert_eq!(private.verifying_key().as_bytes(), public.as_bytes());
```

## Test negative key paths

Use corrupt PEM for parser rejection paths:

```rust
use uselesskey_core::negative::CorruptPem;

let bad_pem = key.private_key_pkcs8_pem_corrupt(CorruptPem::BadBase64);

assert!(SigningKey::from_pkcs8_pem(&bad_pem).is_err());
```

Use truncated DER for byte-level parse failures:

```rust
let truncated = key.private_key_pkcs8_der_truncated(16);

assert_eq!(truncated.len(), 16);
assert!(SigningKey::from_pkcs8_der(&truncated).is_err());
```

Use a mismatched public key for policy paths that compare public and private
material:

```rust
let mismatched = VerifyingKey::from_public_key_der(
    &key.mismatched_public_key_spki_der(),
)
.unwrap();
let original = VerifyingKey::from_public_key_der(key.public_key_spki_der()).unwrap();

assert_ne!(mismatched.as_bytes(), original.as_bytes());
```

## What this proves

- Your tests can create deterministic Ed25519 PKCS#8 and SPKI material without
  committed key fixtures.
- Your parser path accepts valid PEM and DER produced from the same fixture.
- Your negative parser or policy path rejects corrupt, truncated, or mismatched
  key material.
- Your debug and review surfaces can avoid exposing generated key material.

## What this does not prove

- Production signing security, key custody, or key-rotation behavior.
- Provider compatibility or all Ed25519 consumers agreeing on policy.
- Release readiness, downstream verifier correctness, or production security.

## Scanner-safety boundary

Generated Ed25519 PEM and DER values are secret-shaped. Keep them in memory or
under `target/`, do not paste them into docs or source fixtures, and use
`cargo xtask no-blob` to catch accidental committed payloads.

## See also

- [`../../crates/uselesskey-ed25519/README.md`](../../crates/uselesskey-ed25519/README.md)
  - crate-level Ed25519 fixture overview.
- [`../../examples/external/ed25519-fixture-validation/`](../../examples/external/ed25519-fixture-validation/)
  - copyable downstream test wiring.
- [`downstream-fixture-policy.md`](downstream-fixture-policy.md) - policy
  framing for generated test material and scanner-safe review.
