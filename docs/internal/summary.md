# BDD Test Coverage Analysis - Summary

## Overview

This document is the current inventory and claim-boundary summary for the
repository's BDD and related test surfaces. The linked analysis, architecture
diagram, and scenario plan are historical planning artifacts; they provide
context for how the current surfaces were built, but they are not a live
backlog or an exhaustive coverage report.

## Documents Created

1. **[`bdd-test-coverage-analysis.md`](bdd-test-coverage-analysis.md)** - Historical snapshot with gap analysis and improvement plan
2. **[`bdd-scenarios-implementation-plan.md`](bdd-scenarios-implementation-plan.md)** - Historical BDD scenario design context
3. **[`test-architecture-diagram.md`](test-architecture-diagram.md)** - Historical snapshot of the test architecture

## Key Findings

### Current State
- **38 BDD feature files** contain **523 declared scenarios and scenario outlines**.
  This is a repository declaration count, not an execution-coverage or semantic
  completeness claim.
- **24 workspace crates** expose a `tests/` directory in the current checkout;
  this directory inventory includes test-support and integration crates, so it
  is not a claim that every test surface has equal user-facing depth.
- **5/5 adapter crates** have external test directories, including rustls and
  RustCrypto
  ([rustls tests](../../crates/uselesskey-rustls/tests/),
  [RustCrypto tests](../../crates/uselesskey-rustcrypto/tests/))
- **BDD features cover** JWT, TLS, and edge case integration scenarios
- **Cross-adapter compatibility tests** verify ring/RustCrypto interoperability
- **Key rotation workflow tests** cover JWT, JWKS, and TLS rotation scenarios

### Resolved Gaps (since initial analysis)

#### BDD Test Gaps — Resolved
- RSA RS384/RS512 variants — added to `rsa.feature`
- HMAC HS384/HS512 variants — added to `hmac.feature`
- X.509 CRL/revoked leaf scenarios — added to `x509.feature`
- X.509 hostname mismatch scenarios — added to `x509.feature`
- JWT integration tests — new `jwt.feature`
- TLS integration tests — new `tls.feature`
- Edge cases — new `edge_cases.feature`
- JWKS rotation scenarios — added to `jwks.feature`

#### Unit Test Gaps — Resolved
- uselesskey-jsonwebtoken: comprehensive JWT test suite added
- uselesskey-ring: comprehensive ring key type tests added
- uselesskey-aws-lc-rs: comprehensive aws-lc-rs key type tests added
- uselesskey-x509: 14 external unit tests added (spec, chain, tempfile, negative fixtures) ✓
- uselesskey-hmac: 10 external unit tests added (secret lengths, JWK, determinism, debug safety) ✓
- uselesskey-jwk: 7 external unit tests added (serde rename, Display, Debug, kid delegation) ✓
- uselesskey-rustls: config-builder unit tests plus integration, handshake, property,
  and negative tests are present ✓
- uselesskey-rustcrypto: inline, integration, property, cross-verification, and
  snapshot tests are present ✓

#### Integration Test Gaps — Resolved
- Key rotation workflows: 10 tests (JWT, HMAC, JWKS, TLS rotation) ✓
- Cross-adapter compatibility: 8 tests (ECDSA, Ed25519, RSA cross-verify; identity checks) ✓

### Remaining Gaps

#### Remaining Test Risks

The current inventory does not support an untested-adapter claim. Core
concurrency and cache behavior are exercised by
[`concurrency_stress.rs`](../../crates/uselesskey-core/tests/concurrency_stress.rs),
[`edge_cases.rs`](../../crates/uselesskey-core/tests/edge_cases.rs),
[`determinism_regression.rs`](../../crates/uselesskey-core/tests/determinism_regression.rs),
and the cache unit tests in
[`cache.rs`](../../crates/uselesskey-core/src/srp/cache.rs).

Derivation-version migration is not listed as a current gap because the
repository has no migration contract or active compatibility requirement for
it. If that requirement is introduced, define the contract and proof in a
builder-ready issue before adding tests.

## Historical Improvement Plan (Completed)

### Phase 1: Expand Existing BDD Features — DONE
- ~~Add RSA RS384/RS512 variant scenarios~~ ✓
- ~~Add HMAC HS384/HS512 variant scenarios~~ ✓
- ~~Add X.509 CRL/revoked leaf scenarios~~ ✓
- ~~Add X.509 hostname mismatch scenarios~~ ✓
- ~~Add JWKS rotation scenarios~~ ✓

### Phase 2: New BDD Features — DONE
- ~~**jwt.feature**: JWT signing/verification with all key types~~ ✓
- ~~**tls.feature**: TLS server/client config and mTLS scenarios~~ ✓
- ~~**edge_cases.feature**: Label edge cases, cache behavior, determinism~~ ✓

### Phase 3: Unit Test Expansion — DONE
- ~~uselesskey-x509 unit tests (certificate parsing, chain validation, SAN handling)~~ ✓
- ~~uselesskey-hmac unit tests (secret generation, JWK conversion)~~ ✓
- ~~uselesskey-jwk unit tests (JWKS builder, kid generation)~~ ✓
- ~~uselesskey-jsonwebtoken tests~~ ✓
- ~~uselesskey-rustls unit tests (config builders), integration tests, and TLS BDD coverage~~ ✓
- ~~uselesskey-ring tests~~ ✓
- ~~uselesskey-rustcrypto inline and external tests~~ ✓
- ~~uselesskey-aws-lc-rs tests~~ ✓

### Phase 4: Integration Tests — DONE
- ~~JWT end-to-end tests~~ ✓ (via jwt.feature BDD)
- ~~TLS handshake tests~~ ✓ (via tls.feature BDD)
- ~~mTLS scenarios~~ ✓ (via tls.feature BDD)
- ~~Key rotation workflows~~ ✓ (10 tests in tests/key_rotation.rs)
- ~~Cross-adapter compatibility~~ ✓ (8 tests in tests/cross_adapter.rs)

## Future Work Intake

This historical analysis is not a live backlog. New test gaps should be based
on current source and contract evidence, then captured in a builder-ready
issue with a focused proof path.

## Current Inventory Snapshot

| Metric | Current repository evidence | Claim boundary |
|--------|-----------------------------|----------------|
| BDD feature files | 38 | Files under `crates/uselesskey-bdd/features/`; not a semantic coverage claim |
| BDD scenario declarations | 523 | `Scenario` and `Scenario Outline` declarations; not executed-case coverage |
| Workspace crates with `tests/` directories | 24 | Directory inventory, including test-support and integration crates |
| Adapter crates with external tests | 5/5 | The five adapter crates listed in the workspace architecture |
| Cross-adapter tests | 8 | Existing integration-test inventory; not all provider combinations |
| Key rotation tests | 10 | Existing integration-test inventory; not production workflow validation |

## Notes

- BDD tests should remain focused on user-facing behavior
- Unit tests should cover implementation details and edge cases
- Integration tests should verify cross-crate compatibility
- Property-based tests should complement deterministic tests
- Negative fixtures should be first-class citizens in testing
- Test inventory proves that test surfaces exist; it does not by itself prove
  complete provider compatibility, production security, or release readiness.
