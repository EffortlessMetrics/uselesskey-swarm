# Use uselesskey in Downstream CI

Use this recipe when a downstream project wants deterministic fixtures plus a
metadata-only audit receipt during CI.

For GitHub Actions workflow files, see
[use-uselesskey-in-github-actions.md](use-uselesskey-in-github-actions.md).
For copyable workflow and regression snippets, see
[`../../examples/external/ci-recipes/`](../../examples/external/ci-recipes/).

## Copy this

```yaml
steps:
  - run: cargo install uselesskey-cli --version 0.9.1 --locked
  - run: uselesskey bundle --profile webhook --out target/uselesskey-webhook
  - run: uselesskey verify-bundle --path target/uselesskey-webhook
  - run: uselesskey inspect-bundle --path target/uselesskey-webhook
  - run: uselesskey audit-bundle --path target/uselesskey-webhook --out target/uselesskey-webhook-audit --ci --expect-profile webhook --policy strict
```

Switch `webhook` to `tls`, `oidc`, or `scanner-safe` when that is the bundle
profile the job owns. Keep one output directory per profile.

## What you get

CI generates a deterministic bundle under `target/`, verifies the manifest and
profile layout, prints a quick inspect summary, then writes audit receipts:

```text
target/uselesskey-webhook-audit/bundle-audit.md
target/uselesskey-webhook-audit/bundle-audit.json
```

Those files are reviewer metadata. They record paths, counts, profile metadata,
stable failure classes where the profile defines them, and boundaries.

## Positive path

A passing CI run means:

- the installed CLI ran in the downstream project;
- the requested profile generated under the selected output directory;
- `verify-bundle` accepted the local bundle structure;
- `audit-bundle --ci --policy strict` found no meaningful local drift.

## Negative path

With `--policy strict`, `audit-bundle` exits non-zero for stable CI failure
classes such as:

- `missing_manifest`;
- `invalid_manifest`;
- `path_escape`;
- `missing_artifact`;
- `unexpected_artifact`;
- `missing_receipt`;
- `invalid_receipt`;
- `scanner_safe_mismatch`;
- `runtime_material_mismatch`;
- `profile_validation_failed`;
- `unsupported_profile`.

Do not parse English prose from logs. Branch on exit status and, when needed,
the JSON receipt.

## Verify

To run the installed CLI path locally:

```bash
uselesskey bundle --profile webhook --out target/uselesskey-webhook
uselesskey verify-bundle --path target/uselesskey-webhook
uselesskey audit-bundle --path target/uselesskey-webhook --ci --expect-profile webhook --policy strict
```

Repo-local proof for the documented downstream path:

```bash
cargo xtask external-adoption-smoke --path . --ci-recipes --format json
cargo xtask adoption-regression --external
```

## Audit / receipt

Upload only metadata-only audit receipts unless the downstream repository has a
separate reviewed policy for raw generated fixtures.

Keep generated fixture payloads under `target/`. Do not commit generated PEM,
DER, JWT, JWK/JWKS, webhook request, Kubernetes Secret, or Vault payload files
unless a local policy explicitly allows that artifact.

## What this does not prove

- It does not prove provider compatibility.
- It does not prove production security.
- It does not prove every repo public claim.
- It does not prove scanner evasion.
- It does not prove release readiness.
- It does not prove downstream verifier correctness.

Use repo-local proof only when a reviewer asks for public-claim receipts:

```bash
cargo xtask verification-pack --out target/uselesskey-verification
```

For the workflow-to-claim map, see
[`../status/workflow-support.md`](../status/workflow-support.md). For policy
presets and the reviewer checklist, see
[use-downstream-policy-pack.md](use-downstream-policy-pack.md).
