# Post-Release Audit

Run this checklist after publishing a `uselesskey` release from the
release/source boundary repo. It verifies that the shipped registry state,
docs.rs state, installed user path, and public-claim receipts match the release
story.

This audit is release confidence only. It is not production cryptographic
assurance, provider compatibility certification, scanner-policy approval, or
permission to move release authority into `uselesskey-swarm`.

## Inputs

- Release tag, for example `v0.10.0`.
- Published crate version, for example `0.10.0`.
- Source release commit and release workflow URL.
- Release-specific evidence matrix or readiness record.
- Swarm handoff packet, when the release picked up work from
  `EffortlessMetrics/uselesskey-swarm`.

Set local variables before running commands:

```bash
TAG=v0.10.0
VERSION=0.10.0
```

## Immediate Checks

| Check | Command or surface | Pass condition | If it fails |
| --- | --- | --- | --- |
| GitHub release is visible | `gh release view "$TAG"` | Draft is published, tag points at the intended source commit, and release notes keep claim boundaries visible. | Fix release notes if only text is wrong; otherwise stop and investigate the tag or commit mismatch. |
| crates.io packages are visible | `cargo search uselesskey --limit 5` and crate pages for changed packages | The facade and intended publish crates show the published `$VERSION` after indexing delay. | Wait for indexing; if still missing, inspect publish logs and package pages. |
| External install smoke passes | `cargo xtask cratesio-smoke --version "$VERSION"` | A fresh registry install can use the facade and installed CLI path for the published version. | Treat as a release follow-up or patch blocker, depending on the broken surface. |
| docs.rs accepted the facade | `https://docs.rs/crate/uselesskey/$VERSION/status.json` | docs.rs reports a successful build or an explainable queued state. | Open a tracking issue for docs.rs lag or failure; do not republish solely for lag. |
| Release evidence is linkable | Release body and workflow artifacts | Evidence matrix, claim receipts, verification pack, and bundle proof artifacts are linked or reproducible. | Patch release notes or attach the missing receipt. |

## Registry And Installed User Path

Run the external crates.io smoke first:

```bash
cargo xtask cratesio-smoke --version "$VERSION"
```

Then spot-check the installed CLI user path when the release changed bundle,
audit, profile, or receipt behavior:

```bash
cargo install uselesskey-cli --version "$VERSION" --locked
uselesskey doctor
uselesskey profiles
uselesskey bundle --profile oidc --out target/post-release-oidc
uselesskey verify-bundle target/post-release-oidc
uselesskey inspect-bundle target/post-release-oidc
uselesskey audit-bundle target/post-release-oidc --ci --expect-profile oidc --policy strict --out target/post-release-oidc-audit
```

Swap `oidc` for `scanner-safe`, `webhook`, or `tls` when the release claim or
fix targets that profile. Keep generated payloads under `target/`.

## Public Claim Proof

From a clean checkout of the released source commit, regenerate the current
claim and support evidence:

```bash
cargo xtask claim-report --check-public-claims
cargo xtask contract-packs --check
cargo xtask check-support-tiers
cargo xtask check-doc-artifacts
cargo xtask docs-sync --check
cargo xtask badges --check
cargo xtask no-blob
```

Run stable public-claim proofs and the metadata-only reviewer bundle:

```bash
cargo xtask claim-proof --all-stable
cargo xtask verification-pack --out target/uselesskey-verification
```

For stable contract packs, regenerate release evidence receipts:

```bash
cargo xtask bundle-proof --profile tls --out target/release-evidence/tls
cargo xtask bundle-proof --profile oidc --out target/release-evidence/oidc
cargo xtask bundle-proof --profile webhook --out target/release-evidence/webhook
```

Attach or link metadata receipts only. Do not attach generated PEM, DER, JWT,
JWK private members, HMAC secrets, webhook request bodies, Kubernetes Secret
payloads, Vault payloads, or other generated secret-shaped material.

## Package And Source Checks

If the release included packaging, docs, schema, or public-surface changes,
repeat the non-publishing package checks from the source commit:

```bash
cargo xtask publish-preflight
cargo xtask publish-check
cargo xtask public-surface
git diff --check
```

These commands prove packageability and source consistency. They do not publish
again, create tags, sign artifacts, or move source-sync authority.

## Audit Record

Record the audit in the release issue, release closeout, or post-release
tracking issue with this shape:

```markdown
## Post-release audit for v0.10.0

- Release visible: pass/fail, link
- Tag/source commit: pass/fail, SHA
- crates.io packages visible: pass/fail, package list or API evidence
- External install smoke: pass/fail, command evidence
- docs.rs facade status: pass/fail/queued, link
- Public claim checks: pass/fail, command evidence
- Contract-pack receipts: pass/fail, receipt paths
- Verification pack: pass/fail, receipt path
- Package/source checks: pass/fail/not applicable, command evidence
- Open follow-ups:
  - issue link or "none"
```

## Failure Handling

- Text-only release note issue: edit the GitHub release and note the correction
  in the audit record.
- Missing docs.rs build: open a follow-up issue and link the docs.rs build log.
- crates.io publish gap: pause announcements until the missing crate or facade
  path is understood.
- External install or installed CLI regression: open a blocking release issue;
  cut a patch only after the source boundary approves it.
- Contract-pack or metadata-only receipt regression: mark the affected claim
  unproven until a fix lands and proof is regenerated.
- Evidence artifact gap: attach the missing command output or explain why the
  evidence is not applicable for this release.

## Boundaries

This checklist does not prove:

- production security;
- provider compatibility;
- scanner-policy approval;
- downstream verifier correctness;
- future registry state;
- release authority transfer from `EffortlessMetrics/uselesskey` to
  `EffortlessMetrics/uselesskey-swarm`.
