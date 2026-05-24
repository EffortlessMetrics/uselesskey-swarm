# Roadmap

This roadmap reflects the strategic direction for uselesskey as a **test-fixture
layer** (not a crypto library). It describes development direction in
`uselesskey-swarm`; it does not move release, publish, signing, tag, crates.io,
GitHub Release, or source-sync authority out of `EffortlessMetrics/uselesskey`.

Public install snippets stay on the current published version until a release
version exists on crates.io. v0.10.0 references in this repo are release-review
and source-handoff evidence, not a publication claim.

## Now

*Keep the swarm implementation lane green, useful, and release-boundary honest*

- Keep `uselesskey-swarm` as the high-throughput development workspace for
  fixture, CLI, docs, proof, and package-readiness improvements.
- Land small, reversible PRs that make downstream verifier testing easier:
  contract-pack docs, stable negative fixture IDs, metadata-only audit receipts,
  clean-project examples, and package boundary proof.
- Keep the committed source-of-truth graph healthy: `.rails/index.toml`,
  `.rails/migration-status.md`, `.uselesskey/goals/`, `policy/*.toml`,
  `docs/status/*`, plans, handoffs, and release records must agree.
- Treat `Uselesskey Rust Small Result` and `Source of Truth Advisory` as the
  current swarm proof surfaces for PR review.
- Preserve the source/release boundary: swarm can prepare and validate release
  readiness, but does not publish, tag, sign, push to crates.io, create GitHub
  releases, or move source-sync authority without an explicit release lane.

## Next

*Apply the completed v0.10.0 readiness packet at the source boundary*

- Use [`docs/release/source-release-handoff.md`](../release/source-release-handoff.md)
  and [`docs/release/v0.10.0-readiness-record.md`](../release/v0.10.0-readiness-record.md)
  as the current swarm handoff packet.
- In the source repo, rerun release-prep gates against the synced release
  candidate before any version bump, publish, tag, signing, or GitHub Release.
- Reconcile public install snippets only when the release version is actually
  published or the release lane explicitly marks them as candidate-only.
- After release execution, run published-version install smoke and update
  post-release audit records from the source/public boundary.
- Start the next product lane from repo truth, not chat history, and prefer
  user-path improvements over new infrastructure unless the release/user path
  needs it.

## Completed Swarm Lanes

- v0.10.0 release readiness packet, package dry-run, installed CLI smoke, and
  facade smoke.
- Source-of-truth control plane, PR body generation, repo contract reporting,
  and closeout generation.
- Contract-pack and workflow binding for OIDC/JWKS, JWT/token negatives,
  webhook, TLS/X.509, downstream CI recipes, and metadata-only audit packets.
- Historical v0.7.0 follow-up execution is archived in
  [`roadmap-followups-0251.md`](roadmap-followups-0251.md).

## Shipped

### v0.6.0 (2026-04-08)

*Lane-choice, materialization, and release economics*

- Added `uselesskey-entropy` for deterministic high-entropy byte fixtures and scanner-safe placeholder data.
- Added `uselesskey-cli materialize` / `verify` manifest workflows, including build-time `OUT_DIR` examples for shape-only fixtures and explicit RSA PKCS#8 materialization.
- Added `cargo xtask economics` and `cargo xtask audit-surface` so lane cost and advisory-island receipts are generated as repo artifacts.
- Reframed public docs around lane choice first: entropy, token, runtime semantic fixtures, and build-time materialized fixtures.

### v0.5.1 (2026-03-27)

*X.509 negative-fixture expansion and dependency-lane stabilization*

- Added the first X.509 chain-negative wave for not-yet-valid fixtures and
  intermediate path-validation failures, while preserving default deterministic
  certificate outputs.
- Landed the queued maintenance dependency refreshes, including `toml`,
  `insta`, and `sha2`, plus the supporting RustCrypto/HMAC compatibility fixes
  needed to keep adapters, fuzz targets, and CI aligned.
- Prepared the `0.5.1` release manifests, changelog, and release-facing
  dependency snippets.

### v0.5.0 (2026-03-25)

*Adapter-wave release and docs/infrastructure alignment*

- Added a reusable adapter-scaffold template and established adapter acceptance
  requirements.
- Added `uselesskey-jose-openid` and `uselesskey-pgp-native` adapter
  microcrates with runtime examples and smoke/integration coverage.
- Added docs metadata source, `docs-sync`, and examples-smoke coverage to PR
  checks, and aligned release-facing docs to avoid drift.

### v0.4.0 (2026-03)

*RNG boundary cleanup and API hardening*

- [x] Hide rand ABI behind seed boundaries
- [x] Public API no longer leaks rand types
- [x] `Seed` is now the stable boundary between user code and RNG
  implementation
- [x] Support crates and fuzz targets consume the seed-oriented helper APIs

### v0.3.0 (2026-03)

*Facade ergonomics and lightweight token path*

- [x] Empty facade defaults (no default features)
- [x] Token-only lightweight path
- [x] `Seed::from_text` for ergonomic seed creation
- [x] `Factory::deterministic_from_str` convenience method
- [x] Dogfooding smoke coverage via test fixtures
- [x] Updated documentation and README examples

### v0.2.x

*Core functionality - Key types, adapters, and X.509*

- [x] **ECDSA fixtures** (`uselesskey-ecdsa`)
  - P-256 (ES256), P-384 (ES384) via `p256`/`p384` crates
  - PKCS#8/SEC1 private key, SPKI public key
  - `EcdsaFactoryExt` trait
- [x] **Ed25519 fixtures** (`uselesskey-ed25519`)
  - Via `ed25519-dalek`
  - PKCS#8 private key, SPKI public key
  - `Ed25519FactoryExt` trait
- [x] **JWK output methods** on all key types
  - `private_key_jwk()`, `public_key_jwk()`
  - Deterministic `kid` derived from key material (stable in deterministic mode)
  - Symmetric keys (HS256/HS384/HS512) for completeness
- [x] **JWKS builder**
  - Combine multiple public keys into a JWKS
  - Stable key ordering in deterministic mode
- [x] **HMAC fixtures** (`uselesskey-hmac`)
  - HS256/HS384/HS512 secrets
  - JWK/JWKS (`kty=oct`)
- [x] **X.509 leaf certificates** (`uselesskey-x509`)
  - Self-signed certs via `rcgen`
  - Configurable: CN, SANs, validity period, key usage
  - `X509FactoryExt` trait
- [x] **X.509 cert chain fixtures** (`uselesskey-x509`)
  - Root CA -> Intermediate -> Leaf
  - Deterministic serial numbers and validity periods
  - Chain PEM (leaf + intermediate, no root) for standard TLS server usage
  - Individual cert access (root, intermediate, leaf)
- [x] **X.509 negative fixtures** (`uselesskey-x509`)
  - Expired leaf/intermediate certificates
  - Hostname mismatch (wrong SAN)
  - Unknown CA (untrusted root)
  - Revoked leaf with CRL signed by intermediate CA
  - Self-signed leaf, reversed chain, wrong issuer
- [x] **Token fixtures** (`uselesskey-token`)
  - API key, bearer token, and OAuth access token (JWT-shape) fixtures
  - `TokenFactoryExt` trait on `Factory`: `fx.token("issuer", TokenSpec::api_key())`
- [x] **OpenPGP fixtures** (`uselesskey-pgp`)
  - RSA 2048/3072 and Ed25519 transferable keys
  - Armored and binary keyblock outputs
  - `PgpFactoryExt` trait on `Factory`: `fx.pgp("issuer", PgpSpec::ed25519())`
- [x] **Deterministic corruption variants** (`uselesskey-core`)
  - `corrupt_pem_deterministic(pem, variant)` and `corrupt_der_deterministic(der, variant)`
  - Enables stable `corrupt:*` fixture patterns tied to artifact identity
- [x] **`no_std` support in `uselesskey-core`**
  - `std` is now an opt-out default feature
  - Deterministic derivation, artifact identity, and negative helpers compile without `std`
- [x] **Adapter crates**
  - `uselesskey-jsonwebtoken`: Returns `jsonwebtoken::EncodingKey` / `DecodingKey` directly
  - `uselesskey-rustls`: Returns `rustls::pki_types::PrivateKeyDer`, `CertificateDer`
  - `uselesskey-tonic`: Returns `tonic::transport::Identity` / `Certificate` from X.509 fixtures
  - `uselesskey-ring`: Native `ring` 0.17 signing key types
  - `uselesskey-aws-lc-rs`: Native `aws-lc-rs` key types with `native` feature for wasm-safe builds
  - `uselesskey-rustcrypto`: RustCrypto native types (`rsa::RsaPrivateKey`, `p256::ecdsa::SigningKey`, etc.)
- [x] **BDD test suite** (38 feature files, 150+ scenarios)
  - RSA, ECDSA, Ed25519, HMAC, X.509, JWK, JWKS, chains, cross-key, JWT, TLS, PGP, tokens, negative fixtures, edge cases
- [x] **Examples** (22 runnable examples)
  - JWT signing, TLS server chains, negative fixtures, tempfiles, JWKS builder, PGP keys, tokens, adapter integration, gRPC TLS

### v0.1.x

*Foundation - Core factory and RSA*

- [x] Core factory with random and deterministic modes
- [x] Order-independent derivation (BLAKE3 keyed hash)
- [x] DashMap-based concurrent caching
- [x] RSA fixtures via `RsaFactoryExt` trait
- [x] Output formats: PKCS#8 PEM/DER, SPKI PEM/DER
- [x] Tempfile outputs with restrictive permissions
- [x] Negative fixtures: corrupt PEM, truncated DER, mismatched keypairs

## Non-goals

These are explicitly out of scope:

- Production key management
- Hardware-backed keys (HSM, TPM)
- Rotation servers or key lifecycle management
- Perfect scanner evasion (if a scanner flags runtime output, that's a downstream issue)
- Signing/verification APIs (artifacts only)

## Versioning Policy

- **Derivation stability**: Changing the derivation algorithm requires bumping the derivation version field. Existing tests should not break.
- **Semver**: Breaking API changes bump the minor version until 1.0, then major version.
- **Feature flags**: New key types are opt-in via Cargo features to keep compile times reasonable.

[roadmap-followups]: roadmap-followups-0251.md
