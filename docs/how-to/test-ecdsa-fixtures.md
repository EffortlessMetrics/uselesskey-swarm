# Test ECDSA fixtures

Use this when a downstream verifier, parser, or policy layer needs ES256 or
ES384 ECDSA PKCS#8 private keys and SPKI public keys without committing
generated PEM or DER blobs.

A copyable downstream-style test crate lives at
[`../../examples/external/ecdsa-fixture-validation/`](../../examples/external/ecdsa-fixture-validation/).

## Add the test dependencies

```toml
[dev-dependencies]
p256 = { version = "0.14.0-rc.9", features = ["ecdsa", "pkcs8", "pem"] }
p384 = { version = "0.14.0-rc.9", features = ["ecdsa", "pkcs8", "pem"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-ecdsa = "0.9.1"
```

Run the repo proof path with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Generate parseable ES256 material

```rust
use p256::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};
use uselesskey_core::{Factory, Seed};
use uselesskey_ecdsa::{EcdsaFactoryExt, EcdsaSpec};

let fx = Factory::deterministic(Seed::from_env_value("ecdsa-test").unwrap());
let key = fx.ecdsa("release-signer-es256", EcdsaSpec::es256());

let private = p256::SecretKey::from_pkcs8_pem(key.private_key_pkcs8_pem()).unwrap();
let public = p256::PublicKey::from_public_key_pem(key.public_key_spki_pem()).unwrap();

assert_eq!(key.label(), "release-signer-es256");
assert_eq!(key.spec(), EcdsaSpec::es256());
assert!(!private.to_bytes().is_empty());
assert!(!public.to_sec1_bytes().is_empty());
```

Use DER when the downstream API expects bytes:

```rust
p256::SecretKey::from_pkcs8_der(key.private_key_pkcs8_der()).unwrap();
p256::PublicKey::from_public_key_der(key.public_key_spki_der()).unwrap();
```

## Generate parseable ES384 material

```rust
use p384::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};

let key = fx.ecdsa("release-signer-es384", EcdsaSpec::es384());

p384::SecretKey::from_pkcs8_pem(key.private_key_pkcs8_pem()).unwrap();
p384::PublicKey::from_public_key_pem(key.public_key_spki_pem()).unwrap();
p384::SecretKey::from_pkcs8_der(key.private_key_pkcs8_der()).unwrap();
p384::PublicKey::from_public_key_der(key.public_key_spki_der()).unwrap();
```

## Test negative key paths

Use corrupt PEM for parser rejection paths:

```rust
use uselesskey_core::negative::CorruptPem;

let bad_pem = key.private_key_pkcs8_pem_corrupt(CorruptPem::BadBase64);

assert!(p256::SecretKey::from_pkcs8_pem(&bad_pem).is_err());
```

Use truncated DER for byte-level parse failures:

```rust
let truncated = key.private_key_pkcs8_der_truncated(16);

assert_eq!(truncated.len(), 16);
assert!(p256::SecretKey::from_pkcs8_der(&truncated).is_err());
```

Use a mismatched public key for policy paths that compare public and private
material:

```rust
let mismatched = key.mismatched_public_key_spki_der();

assert_ne!(mismatched, key.public_key_spki_der());
p256::PublicKey::from_public_key_der(&mismatched).unwrap();
```

## What this proves

- Your tests can create deterministic ES256 and ES384 PKCS#8/SPKI material
  without committed key fixtures.
- Your parser path accepts valid PEM and DER produced from the fixture family.
- Your negative parser or policy path rejects corrupt, truncated, or mismatched
  key material.
- Your debug and review surfaces can avoid exposing generated key material.

## What this does not prove

- Production signing security, key custody, key rotation, or algorithm policy.
- Provider compatibility or all ECDSA consumers agreeing on curve handling.
- Release readiness, downstream verifier correctness, or production security.

## Scanner-safety boundary

Generated ECDSA PEM and DER values are secret-shaped. Keep them in memory or
under `target/`, do not paste them into docs or source fixtures, and use
`cargo xtask no-blob` to catch accidental committed payloads.

## See also

- [`../../crates/uselesskey-ecdsa/README.md`](../../crates/uselesskey-ecdsa/README.md)
  - crate-level ECDSA fixture overview.
- [`../../examples/external/ecdsa-fixture-validation/`](../../examples/external/ecdsa-fixture-validation/)
  - copyable downstream test wiring.
- [`test-ed25519-fixtures.md`](test-ed25519-fixtures.md) - Ed25519 parser and
  policy fixture workflow.
- [`downstream-fixture-policy.md`](downstream-fixture-policy.md) - policy
  framing for generated test material and scanner-safe review.
