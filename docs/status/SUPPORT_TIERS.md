# Support Tiers

Source-of-truth artifact id: `USELESSKEY-STATUS-support-tiers`

This page maps user-facing surfaces to support tiers, public claims, proof
commands, docs, boundaries, and release lanes. It is the status-level companion
to [`PUBLIC_CLAIMS.md`](PUBLIC_CLAIMS.md) and
[`policy/claim-ledger.toml`](../../policy/claim-ledger.toml).

Crate-level API and publish support remains in the generated
[`docs/reference/support-matrix.md`](../reference/support-matrix.md). This page
covers claim and workflow support.

## Tier Definitions

| Tier | Meaning |
| --- | --- |
| Stable | Public users may depend on the documented behavior when the listed proof commands pass. Changes require claim-ledger and docs updates. |
| Stabilizing | The surface is documented and proof-backed, but the exact support boundary or automation may still tighten before promotion. |
| Experimental | Repo-local or early user workflow. Behavior may change without a compatibility promise. |
| Advisory | Evidence or reviewer signal. Useful for decisions, but not a user-facing guarantee by itself. |
| Not supported | Explicitly outside the `uselesskey` promise. Do not advertise as a claim or branch downstream CI on it. |

Support tier is a workflow posture, not the same field as claim status in
[`PUBLIC_CLAIMS.md`](PUBLIC_CLAIMS.md) or `policy/claim-ledger.toml`. A workflow
can be stabilizing while the linked public claim remains advisory. In that case,
README, release, and handoff wording must use the stricter public-claim
boundary.

## Claim Support Map

| Surface | Tier | Claim | Proof command | Docs | Boundary | Release lane |
| --- | --- | --- | --- | --- | --- | --- |
| Scanner-safe fixtures | Stable | `scanner-safe-fixtures` | `cargo xtask scanner-safe-reference --check`; `cargo xtask bundle-proof --profile scanner-safe --out target/release-evidence/scanner-safe`; `cargo xtask no-blob`; `cargo xtask badges --check` | `docs/VERIFICATION.md`; `docs/how-to/downstream-fixture-policy.md`; `docs/how-to/export-vault-kv-fixtures.md`; `docs/how-to/generate-scanner-safe-k8s-secret.md` | Does not mean every encoded export is safe to commit or that `uselesskey` provides production key management. | `pr`, `patch`, `minor` |
| `ripr+` evidence endpoint | Stable | `ripr-plus-evidence-endpoint` | `cargo xtask badges --check`; `cargo xtask test-efficiency-report` | `docs/VERIFICATION.md` | Repo-scoped evidence counter, not coverage, runtime mutation proof, or correctness proof. | `pr`, `main`, `patch`, `minor` |
| Generated badge endpoints | Stable | `generated-badge-endpoints` | `cargo xtask badges`; `cargo xtask badges --check` | `badges/README.md`; `docs/VERIFICATION.md` | Badge JSON is a generated Shields receipt, not a hand-written status claim. | `pr`, `main`, `patch`, `minor` |
| Public crate surface | Stable | `public-crate-surface-cleanup` | `cargo xtask public-surface`; `cargo xtask publish-check`; `cargo xtask publish-preflight` | `docs/architecture/public-surface.md`; `docs/how-to/migrate-to-v0.8.md` | Removed internal compatibility shims are not promised as supported public crates. | `patch`, `minor` |
| OIDC/JWKS contract pack | Stable | `oidc-jwks-contract-pack` | `cargo xtask bundle-proof --profile oidc --out target/release-evidence/oidc`; `cargo xtask no-blob` | `docs/how-to/test-oidc-jwks-validation.md`; `docs/how-to/test-jwt-negative-validation.md` | Does not prove production signing-key custody, full OpenID provider behavior, or downstream validator correctness. | `minor` |
| Webhook contract pack | Stable | `webhook-contract-pack` | `cargo xtask bundle-proof --profile webhook --out target/release-evidence/webhook`; `cargo xtask no-blob` | `docs/how-to/test-webhook-signature-validation.md`; `docs/specs/USELESSKEY-SPEC-0011-webhook-contract-pack.md` | Does not prove provider compatibility, production secret management, replay protection completeness, or transport security. | `minor` |
| TLS contract pack | Stable | `tls-contract-pack` | `cargo xtask bundle-proof --profile tls --out target/release-evidence/tls`; `cargo xtask no-blob` | `docs/how-to/test-tls-chain-validation.md`; `docs/release/v0.8.0-tls-profile-design.md` | Does not prove mTLS, revocation, CT, browser trust-store behavior, production CA custody, or downstream verifier correctness. | `minor` |
| JWT/token negative fixtures | Stabilizing | `jwt-token-negative-fixtures` | `cargo test -p uselesskey-token --all-features`; `cargo xtask check-negative-fixtures` | `docs/how-to/test-jwt-negative-validation.md`; `docs/specs/USELESSKEY-SPEC-0016-negative-fixture-taxonomy.md` | Does not prove production authorization, provider compatibility, cryptographic signature assurance, or downstream verifier correctness. | `minor` |
| Metadata-only audit packets | Stabilizing | `metadata-only-audit-packets` | `cargo test -p uselesskey-cli --all-features audit_bundle`; `cargo xtask external-adoption-smoke --path . --ci-recipes --format json`; `cargo xtask no-blob` | `docs/how-to/share-installed-bundle-audit.md`; `docs/how-to/use-uselesskey-in-downstream-ci.md`; `docs/reference/bundle-audit-json.md`; `docs/reference/audit-failure-classes.md`; `docs/specs/USELESSKEY-SPEC-0014-installed-bundle-audit.md` | Must not include PEM private keys, JWT values, HMAC secrets, JWK private members, webhook request bodies, or generated secret-shaped payloads. | `minor` |
| Downstream policy pack recipes | Stabilizing | `metadata-only-audit-packets` | `cargo test -p uselesskey-cli --all-features audit_bundle`; `cargo xtask external-adoption-smoke --path . --ci-recipes --format json`; `cargo xtask no-blob` | `docs/how-to/use-downstream-policy-pack.md`; `docs/how-to/use-uselesskey-in-github-actions.md`; `examples/external/ci-recipes/README.md`; `docs/specs/USELESSKEY-SPEC-0020-downstream-policy-pack.md` | Supports copyable downstream CI recipe wiring and metadata-only audit packet generation. Does not prove release readiness, scanner-policy approval, provider compatibility, or downstream verifier correctness. | `minor` |
| Bundle manifest schema | Stabilizing | `bundle-manifest-schema` | `cargo xtask check-bundle-schemas` | `docs/specs/USELESSKEY-SPEC-0017-bundle-product-surface.md`; `docs/schemas/bundle-manifest.schema.json` | Proves generated bundle metadata shape, not provider compatibility or downstream verifier correctness. | `minor` |
| Negative coverage receipt | Stabilizing | `negative-coverage-receipt` | `cargo xtask check-negative-fixtures`; `cargo xtask check-bundle-schemas` | `docs/status/negative-fixture-matrix.md`; `docs/specs/USELESSKEY-SPEC-0016-negative-fixture-taxonomy.md` | Records stable negative IDs exposed by bundle profiles; does not prove exhaustive verifier testing. | `minor` |
| External crates.io install smoke | Advisory | `external-cratesio-install-smoke` | `cargo xtask cratesio-smoke --version 0.9.1` | `docs/release/post-release-audit.md` | Proves one published version install path, not every downstream feature combination or future registry state. | `post-release`, `minor` |
| `ripr` PR review evidence | Advisory | `ripr-pr-review-evidence` | `cargo xtask ripr-pr --check`; `cargo xtask ripr-review-comments --check`; `cargo xtask ripr-pr-summary --check` | `docs/VERIFICATION.md`; `docs/ci/test-evidence-lanes.md` | Diff-scoped reviewer evidence, not the repo-scoped README `ripr+` badge. | `pr` |

## Explicit Non-Support

| Surface | Tier | Claim | Proof command | Boundary |
| --- | --- | --- | --- | --- |
| Production key management | Not supported | none | none | `uselesskey` generates test fixtures and does not manage production keys or secrets. |
| Provider compatibility certification | Not supported | none | none | Contract packs exercise documented fixture paths; they do not certify Stripe, GitHub, OpenID provider, browser, CA, or TLS-client compatibility. |
| Downstream verifier correctness | Not supported | none | none | Fixtures can drive downstream tests, but the downstream verifier owns its own acceptance and rejection logic. |
| Release, publish, signing, or source-sync authority in swarm | Not supported | none | none | `EffortlessMetrics/uselesskey` remains the release and public-source boundary until deliberately moved. |

## Promotion Rule

Promote a surface only when the claim ledger, this support map, docs, proof
commands, and any affected policy ledger all agree. A stable row must have a
claim ID, at least one proof command, a docs path, a boundary, and a release
lane.
