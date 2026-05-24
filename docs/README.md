# Documentation

See the [project README](../README.md) for a quick start.

This documentation follows the [Diátaxis framework](https://diataxis.fr/), organizing content by purpose:

## Architecture Decisions

Architecture Decision Records (ADRs) capture significant design choices and their rationale.

- [ADR Index](adr/README.md) — Overview and index of all decisions
- [0001-use-adr-template.md](adr/0001-use-adr-template.md) — ADR format and process
- [0002-seed-boundary-abstraction.md](adr/0002-seed-boundary-abstraction.md) — RNG boundary abstraction for v0.4
- [0003-order-independent-determinism.md](adr/0003-order-independent-determinism.md) — Order-independent derivation design
- [0004-microcrate-architecture.md](adr/0004-microcrate-architecture.md) — Microcrate decomposition strategy

## Source of Truth

Repository operating artifacts are split by job so public claims, proof, and
agent state do not drift into one document.

- [proposals](proposals/README.md) - Why a lane exists, who benefits, and what alternatives were considered
- [specs](specs/README.md) - What behavior is promised, not promised, and how it is proven
- [source-of-truth](source-of-truth/README.md) - Artifact roles, linking rules, and agent operating model
- [templates](templates/) - Starting shapes for source-of-truth artifacts and review packets
- [status](status/README.md) - Public claim, support tier, and proof mapping indexes
- [handoffs](handoffs/README.md) - Closeout notes and operator handoffs after a lane changes state
- [learnings](learnings/README.md) - Durable lessons from releases, incidents, and proof lanes
- [rails.md](rails.md) - Rails control-plane footprint, ownership model, and awareness-only namespaces
- [plans](../plans/README.md) - PR sequencing and rollback plans
- [active goals](../.uselesskey/goals/README.md) - Machine-readable current agent lane state
- [agent bootstrap](handoffs/agent-bootstrap.md) - Read order and validation defaults for agents resuming work
- [local validation](handoffs/local-validation.md) - Local proof boundaries, workspace target-dir guidance, and hosted-CI handoff rules

## How-to Guides

Task-oriented instructions for common workflows.

- [choose-lane.md](how-to/choose-lane.md) — Pick the cheapest correct lane first
- [start-here.md](how-to/start-here.md) — Pick a user job and copy the first command or dependency snippet
- [migration.md](how-to/migration.md) — Migrating between uselesskey versions
- [migrate-to-v0.8.md](how-to/migrate-to-v0.8.md) - Migrating from v0.7.x internal shim crates to v0.8.0 owner-crate `srp::*` modules
- [publishing.md](how-to/publishing.md) — Publishing crates to crates.io
- [recover-partial-publish.md](how-to/recover-partial-publish.md) - Recovering from a partial publish using crates.io as registry truth
- [release.md](how-to/release.md) — Cutting a release
- [choose-features.md](how-to/choose-features.md) — Choosing feature sets by need
- [downstream-fixture-policy.md](how-to/downstream-fixture-policy.md) — Policy pack for downstream bots and reviewers
- [use-uselesskey-in-downstream-ci.md](how-to/use-uselesskey-in-downstream-ci.md) — Installed CLI bundle audit in downstream CI
- [use-uselesskey-in-github-actions.md](how-to/use-uselesskey-in-github-actions.md) — GitHub Actions recipes for bundle audit receipts
- [use-downstream-policy-pack.md](how-to/use-downstream-policy-pack.md) - Downstream policy presets, CI recipe pack, and metadata-only reviewer packets
- [verify-a-fixture-bundle.md](how-to/verify-a-fixture-bundle.md) — Generate, verify, inspect, and audit installed fixture bundles
- [share-installed-bundle-audit.md](how-to/share-installed-bundle-audit.md) — Share metadata-only bundle audit packets with reviewers
- [adapter-template.md](how-to/adapter-template.md) — Scaffolding and validating new adapter crates
- [test-oidc-jwks-validation.md](how-to/test-oidc-jwks-validation.md) — Using the OIDC/JWKS contract pack in validator tests
- [test-oidc-jwks-test-server.md](how-to/test-oidc-jwks-test-server.md) - Using local OIDC discovery and JWKS HTTP routes in verifier tests
- [test-jwt-negative-validation.md](how-to/test-jwt-negative-validation.md) — Using JWT-shaped negatives for policy rejection tests
- [test-jsonwebtoken-adapter.md](how-to/test-jsonwebtoken-adapter.md) - Using `jsonwebtoken` adapter keys in signing and verification tests
- [test-webhook-signature-validation.md](how-to/test-webhook-signature-validation.md) — Using webhook signature fixtures and negative request classes
- [test-hmac-signature-validation.md](how-to/test-hmac-signature-validation.md) - Using HMAC shared-secret fixtures to test signature policy paths
- [test-tls-chain-validation.md](how-to/test-tls-chain-validation.md) — Using TLS/X.509 chain fixtures and negative certificate classes
- [test-ssh-fixtures.md](how-to/test-ssh-fixtures.md) - Using SSH key and certificate fixtures to test parser and policy paths.
- [test-pgp-fixtures.md](how-to/test-pgp-fixtures.md) - Using PGP key fixtures to test parser and policy paths.
- [test-ecdsa-fixtures.md](how-to/test-ecdsa-fixtures.md) - Using ECDSA key fixtures to test parser and policy paths
- [test-ed25519-fixtures.md](how-to/test-ed25519-fixtures.md) - Using Ed25519 key fixtures to test parser and policy paths
- [test-webauthn-validation.md](how-to/test-webauthn-validation.md) — Using WebAuthn-shaped fixtures in relying-party tests
- [use-pkcs11-mock-fixtures.md](how-to/use-pkcs11-mock-fixtures.md) — Using PKCS#11 mock fixtures for token-backed tests
- [test-entropy-byte-fixtures.md](how-to/test-entropy-byte-fixtures.md) - Using deterministic byte fixtures for placeholder and policy inputs
- [generate-scanner-safe-k8s-secret.md](how-to/generate-scanner-safe-k8s-secret.md) — Exporting scanner-safe Kubernetes and Vault-shaped payloads
- [export-vault-kv-fixtures.md](how-to/export-vault-kv-fixtures.md) — Exporting scanner-safe Vault KV-shaped payloads
- [materialize-fixtures-in-build-rs.md](how-to/materialize-fixtures-in-build-rs.md) — Materializing disposable fixtures from `build.rs`
- [verify-uselesskey-public-claims.md](how-to/verify-uselesskey-public-claims.md) — Verifying badge, scanner-safe, contract-pack, claim-proof, and release-smoke claims
- [share-uselesskey-verification-pack.md](how-to/share-uselesskey-verification-pack.md) — Collecting metadata-only public-claim receipts for security and platform review

## Contract Packs

Deterministic fixture bundles with documented positive and negative verifier
paths.

- [contract-packs](contract-packs/README.md) — Generate, verify, prove, and understand scanner-safe, TLS, OIDC/JWKS, and webhook profiles
- [external examples](../examples/external/README.md) — Downstream-shaped CI, verifier, facade, webhook, OIDC/JWKS, and TLS examples

## Contributing

Repository operating rules for agents and maintainers.

- [pr-disposition.md](contributing/pr-disposition.md) — Reviewing keeper, duplicate, stale, declined, and automation PRs
- [rails.md](contributing/rails.md) - Adding or updating Rails source-of-truth artifacts without breaking current uselesskey checkers

## CI and Evidence

Validation lanes and the claim boundaries behind them.

- [VERIFICATION.md](VERIFICATION.md) — README badge meanings, generated endpoints, and PR evidence boundaries
- [coverage.md](ci/coverage.md) — Coverage workflow scope, artifacts, and advisory status
- [routed-rust-workflow.md](ci/routed-rust-workflow.md) - Swarm routed Rust CI targets, self-hosted runner discovery, and normalized check boundaries
- [test-evidence-lanes.md](ci/test-evidence-lanes.md) — PR, targeted mutation, nightly mutation, and release evidence model

## Release

Release-candidate proof and public promise evidence.

- [v0.10.0-readiness-record.md](release/v0.10.0-readiness-record.md) - v0.10.0 release-readiness proof, hosted CI, non-blockers, and release-boundary handoff
- [v0.10.0-public-surface-inventory.md](release/v0.10.0-public-surface-inventory.md) - v0.10.0 release-facing surfaces, user commands, proof, risk, and owners
- [v0.10.0-version-snippet-reconciliation.md](release/v0.10.0-version-snippet-reconciliation.md) - v0.10.0 current-stable versus release-candidate snippet decisions
- [v0.10.0-installed-cli-smoke.md](release/v0.10.0-installed-cli-smoke.md) - v0.10.0 installed CLI smoke evidence for doctor, bundle, verify, inspect, and audit
- [v0.10.0-facade-release-smoke.md](release/v0.10.0-facade-release-smoke.md) - v0.10.0 facade example smoke evidence
- [v0.10.0-package-dry-run.md](release/v0.10.0-package-dry-run.md) - v0.10.0 package dry-run and packaged-content evidence
- [source-release-handoff.md](release/source-release-handoff.md) - Swarm-to-source handoff checklist and release-authority boundary
- [evidence-matrix-v0.9.0.md](release/evidence-matrix-v0.9.0.md) — v0.9.0 command-backed claim, verification-pack, PR-lite, and webhook proof matrix
- [post-release-audit-v0.9.0.md](release/post-release-audit-v0.9.0.md) - v0.9.0 crates.io, docs.rs, claim-proof, verification-pack, and contract-pack audit
- [evidence-matrix-v0.9.1.md](release/evidence-matrix-v0.9.1.md) — v0.9.1 adoption-confidence patch proof matrix
- [post-release-audit-v0.9.1.md](release/post-release-audit-v0.9.1.md) — v0.9.1 crates.io, docs.rs, adoption-regression, and claim-proof audit
- [evidence-matrix-v0.7.0.md](release/evidence-matrix-v0.7.0.md) — v0.7.0 public fixture promise evidence matrix
- [v0.7.0-checklist.md](release/v0.7.0-checklist.md) — Release-governance issue map for v0.7.0 checklist lines
- [v0.7.0-category-notes.md](release/v0.7.0-category-notes.md) — Release note category map and draft-audit guidance for v0.7.0
- [v0.7.0-lessons-learned.md](release/v0.7.0-lessons-learned.md) - v0.7.0 publish-lane failure classes, fixes, and guardrail lessons
- [v0.8.0-tls-profile-design.md](release/v0.8.0-tls-profile-design.md) - v0.8.0 TLS contract-pack profile design and evidence routing
- [publish-recovery.md](release/publish-recovery.md) - Registry-truth and partial-publish recovery rules
- [post-release-audit.md](release/post-release-audit.md) — Post-publish verification checklist for public fixture promises
- [scanner-safe-bundle](../examples/scanner-safe-bundle/README.md) — Reference manifest, receipts, and Kubernetes/Vault payload shapes for the default bundle lane

## Explanation

Understanding-oriented material on design and direction.

- [architecture.md](explanation/architecture.md) — Workspace structure and crate map
- [cli-proof-handoff-boundary.md](explanation/cli-proof-handoff-boundary.md) — Why installed CLI proof remains a handoff instead of an executable proof runner
- [public-surface.md](architecture/public-surface.md) — Supported public crates versus published internal implementation shards
- [roadmap.md](explanation/roadmap.md) — Current swarm direction, completed lanes, and source/release boundary
- [requirements.md](explanation/requirements.md) — Problem statement and design requirements

## Reference

Specifications and formal definitions.

- [dependency-economics.md](reference/dependency-economics.md) — Current lane cost receipts
- [audit-surface.md](reference/audit-surface.md) — Current audit/island receipts
- [bundle-audit-json.md](reference/bundle-audit-json.md) — Stable installed bundle audit JSON contract
- [bundle-inspect-vs-audit.md](reference/bundle-inspect-vs-audit.md) — When to use quick bundle inspection versus durable audit receipts
- [verification-badges.md](reference/verification-badges.md) — Generated README badge endpoint rules and boundaries
- [requirements-v0.3.md](reference/requirements-v0.3.md) — v0.3 acceptance specification
- [requirements-v0.4.md](reference/requirements-v0.4.md) — v0.4 RNG boundary refactor specification
- [failure-atlas.md](reference/failure-atlas.md) — Failure classes covered by protocol-shaped negative fixtures

## Internal

Historical planning artifacts (not user-facing).

- [summary.md](internal/summary.md)
- [bdd-test-coverage-analysis.md](internal/bdd-test-coverage-analysis.md)
- [bdd-scenarios-implementation-plan.md](internal/bdd-scenarios-implementation-plan.md)
- [pr-bundle-reduction.md](internal/pr-bundle-reduction.md)
- [test-architecture-diagram.md](internal/test-architecture-diagram.md)
