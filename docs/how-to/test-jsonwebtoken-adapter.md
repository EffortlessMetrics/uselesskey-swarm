# I need to test jsonwebtoken signing and verification

Use this guide when a downstream Rust test already uses
[`jsonwebtoken`](https://crates.io/crates/jsonwebtoken) and needs deterministic
RSA or HMAC fixture material without committing PEM files or shared secrets.

## Copy this

```toml
[dev-dependencies]
jsonwebtoken = { version = "10", features = ["use_pem", "rust_crypto"] }
serde = { version = "1", features = ["derive"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-hmac = "0.9.1"
uselesskey-jsonwebtoken = { version = "0.9.1", features = ["rsa", "hmac"] }
uselesskey-rsa = "0.9.1"
```

```rust
use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uselesskey_core::Factory;
use uselesskey_jsonwebtoken::JwtKeyExt;
use uselesskey_rsa::{RsaFactoryExt, RsaSpec};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

let fx = Factory::deterministic_from_str("jsonwebtoken-fixtures");
let issuer = fx.rsa("issuer", RsaSpec::rs256());
let claims = Claims { sub: "user-123".into(), exp: 2_000_000_000 };

let token = encode(&Header::new(Algorithm::RS256), &claims, &issuer.encoding_key())?;
let decoded = decode::<Claims>(&token, &issuer.decoding_key(), &Validation::new(Algorithm::RS256))?;
```

For a copyable downstream test crate, see
[`../../examples/external/jsonwebtoken-adapter-validation/`](../../examples/external/jsonwebtoken-adapter-validation/).

## What you get

`uselesskey-jsonwebtoken` implements `JwtKeyExt` for fixture key types so tests
can call:

- `encoding_key()` for `jsonwebtoken::EncodingKey`;
- `decoding_key()` for `jsonwebtoken::DecodingKey`.

The adapter supports RSA, ECDSA, Ed25519, and HMAC when the matching adapter
features are enabled. The clean-project example uses RSA and HMAC because they
cover asymmetric and shared-secret verifier paths without adding every fixture
family to the first copyable snippet.

## Positive path

Sign a token with the fixture's `encoding_key()`, then verify it with the same
fixture's `decoding_key()` and your normal `Validation` policy. This proves that
the test is using the same `jsonwebtoken` entry points as production code, while
the key material itself remains generated test fixture material.

## Negative path

Use adjacent deterministic fixtures to exercise rejection branches:

- verify an RS256 token with a different RSA fixture's `decoding_key()`;
- verify an HS256 token with a different HMAC fixture's `decoding_key()`;
- decode an HS256 token with an RS256 `Validation` policy and RSA key.

Assert the downstream rejection class your application cares about, not merely
that token parsing failed.

## Verify

Clean-project proof from this repo:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

Adapter crate proof:

```bash
cargo test -p uselesskey-jsonwebtoken --all-features
```

## What this does not prove

- It does not prove production token security.
- It does not prove provider compatibility.
- It does not prove issuer, audience, or authorization policy completeness.
- It does not prove downstream verifier correctness.
- It does not prove release readiness or crates.io publish state.

Generated JWT strings and HMAC secrets are runtime material. Keep captured token
values under `target/` or generate them in-process during tests rather than
committing them.
