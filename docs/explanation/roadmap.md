# Roadmap

This roadmap reflects the strategic direction for uselesskey as a **test-fixture
layer** (not a crypto library). It describes development direction in
`uselesskey-swarm`; it does not move release, publish, signing, tag, crates.io,
GitHub Release, or source-sync authority out of `EffortlessMetrics/uselesskey`.

`v0.10.0` is the current published baseline. The source repository's
[`v0.10.0` post-release audit](https://github.com/EffortlessMetrics/uselesskey/blob/main/docs/release/post-release-audit-v0.10.0.md)
records all intended publish crates at `0.10.0`, docs.rs success, and passing
published-version CLI/facade adoption smoke. Swarm still needs a reviewed
post-release reconciliation of package metadata and generated copyable snippets;
do not represent newer swarm-only work as part of the already-published
`v0.10.0` payload.

## Now

*Restore a truthful post-v0.10 development baseline and close real fixture-user gaps*

- Keep `uselesskey-swarm` as the high-throughput development workspace for
  fixture, CLI, docs, proof, and package-readiness improvements.
- Reconcile the completed `v0.10.0` source release back into current swarm
  status without overwriting later swarm work or importing release authority.
- Prioritize concrete fixture/consumer correctness over catalog expansion. The
  current product program is tracked in
  [`#655`](https://github.com/EffortlessMetrics/uselesskey-swarm/issues/655),
  beginning with the webhook tampered-body correction and portable downstream
  consumer proof for webhook, TLS, and signed JWT workflows.
- Keep source-path, packaged-candidate, and published-version evidence distinct.
  A successful checkout smoke is not publication proof, and a metadata receipt
  is not downstream verifier proof.
- Keep the committed source-of-truth graph healthy: `.rails/index.toml`,
  `.rails/migration-status.md`, `.uselesskey/goals/`, `policy/*.toml`,
  `docs/status/*`, plans, handoffs, and release records must agree. Draft
  Rails PR #641 is the current integration point; this roadmap does not activate
  a competing lane.
- Treat `Uselesskey Rust Small Result` and `Source of Truth Advisory` as the
  current swarm PR proof surfaces while keeping their claim boundaries explicit.
  Compile-only fallback proof does not substitute for a focused behavioral test.
- Preserve the source/release boundary: swarm can prepare and validate release
  readiness, but does not publish, tag, sign, push to crates.io, create GitHub
  releases, or move source-sync authority without an explicit source release
  lane.

## Next

*Turn the existing fixture surface into portable, executable consumer proof*

- Correct the released webhook tampered-payload negative so the delivered body
  and preserved signature actually mismatch while valid fixture identity remains
  unchanged.
- Make advertised external examples literally portable: no unpublished workspace
  helper may be silently patched into a project claimed as independently
  copyable.
- Promote existing owner-crate behavior into small downstream recipes rather
  than rebuilding protocol stacks: real webhook HMAC verification, rustls
  handshake/certificate negatives, and signed JWT claim-policy negatives.
- Extend existing adoption smoke with resolved dependency-origin and executed
  test-case receipts instead of adding another parallel harness.
- Compare released and candidate fixture identity/serialized outputs at the
  touched family boundary before accepting dependency or serializer drift.
- Define bundle recipe/version and interrupted-rerun behavior only where current
  evidence shows the installed workflow needs a stronger contract; do not turn
  every future improvement into a release prerequisite.
- Select the next source handoff from coherent completed user value. A narrow
  correctness release can proceed without waiting for every optional dependency
  major, OIDC rotation experiment, economics study, or adoption follow-up.

## Completed Swarm Lanes

- v0.10.0 release readiness packet, package dry-run, installed CLI smoke, and
  facade smoke; the source release subsequently published on June 20, 2026.
- Source-of-truth control plane, PR body generation, repo contract reporting,
  and closeout generation.
- Contract-pack and workflow binding for OIDC/JWKS, JWT/token negatives,
  webhook, TLS/X.509, downstream CI recipes, and metadata-only audit packets.
- Historical v0.7.0 follow-up execution is archived in
  [`roadmap-followups-0251.md`](roadmap-followups-0251.md).

## Shipped

### v0.10.0 (2026-06-20)

*External adoption, installed audit, and real workflow closure*

- Published all intended public/facade/adapter/CLI crates at `0.10.0` from
  source tag `5ea65e1cc6309042731cf4ec91cb39f00a91253a`.
- Added and exercised clean-project external-adoption paths, installed bundle
  audit/CI receipts, profile discovery and inspection UX, and downstream policy
  recipes.
- Expanded realistic workflow and negative-fixture coverage around OIDC/JWKS,
  webhook, token, TLS/X.509, and related public fixture families.
- Passed published-version CLI/facade/CI-recipe smoke and crates.io install smoke
  from isolated Cargo state after publication.
- See the source repository
  [`post-release audit`](https://github.com/EffortlessMetrics/uselesskey/blob/main/docs/release/post-release-audit-v0.10.0.md)
  for registry, docs.rs, tag, package, and published-smoke receipts.

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
- [x] **BDD test suite** (38 feature files, 523 declared scenarios and
  scenario outlines; declaration count, not a semantic coverage claim)
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
