# Share an Installed Bundle Audit

Use this when you generated a bundle with the installed CLI and need a
metadata-only packet for a security or platform reviewer.

The installed audit path explains the local bundle and binds to the advisory
`metadata-only-audit-packets` claim. A downstream receipt is still local
evidence; it is not standalone proof for broader repo public claims, release
evidence, provider compatibility proof, or production security assurance.

## Generate a Bundle

```bash
cargo install uselesskey-cli --version 0.9.1 --locked
uselesskey doctor --format json
uselesskey bundle --profile webhook --out target/uselesskey-webhook
uselesskey verify-bundle target/uselesskey-webhook
uselesskey inspect-bundle target/uselesskey-webhook
```

Keep generated payloads under `target/` or another ignored build directory.

## Audit the Bundle

```bash
uselesskey audit-bundle target/uselesskey-webhook --out target/uselesskey-webhook-audit
```

Attach these files:

```text
target/uselesskey-webhook-audit/bundle-audit.md
target/uselesskey-webhook-audit/bundle-audit.json
```

Do not attach generated request payloads, PEM/DER material, token files,
Kubernetes Secret YAML, Vault JSON, or other raw fixture payloads unless your
review process explicitly asks for them.

## Reviewer Checklist

Attach the metadata-only receipts, then record:

- the generated profile, such as `webhook`, `tls`, `oidc`, or `scanner-safe`;
- the command used to verify the bundle;
- the command used to audit the bundle;
- whether `bundle-audit.json` reports `status: "pass"`;
- whether every `checks[]` entry reports `status: "pass"`;
- whether `bundle-audit.md` reports `- Status: pass` and the expected
  `- Profile: <profile>`;
- whether `bundle-audit.md` includes the metadata-only `Boundaries` section
  and a production-security item under `Does Not Prove`;
- whether generated fixture payloads stayed under `target/` or another ignored
  output directory;
- the explicit "does not prove" boundary below.

For CI jobs, prefer the strict policy preset:

```bash
uselesskey audit-bundle \
  target/uselesskey-webhook \
  --ci \
  --expect-profile webhook \
  --policy strict \
  --out target/uselesskey-webhook-audit
```

With `--ci --out`, the audit directory is written for passing audits and stable
policy failures, so a reviewer can inspect the same metadata-only packet when
CI rejects a bundle.

For the full downstream policy preset list, see
[use-downstream-policy-pack.md](use-downstream-policy-pack.md).

## JSON for CI

```bash
uselesskey audit-bundle target/uselesskey-webhook --format json
```

The JSON receipt reports:

- bundle profile and manifest version;
- artifact paths, kinds, formats, and scanner-safe labels;
- runtime-material counts;
- expected receipt files;
- missing or unexpected files;
- profile validation status;
- explicit boundaries.

## What This Proves

`audit-bundle` proves local bundle consistency:

- the manifest parses;
- generated files are contained by the bundle path;
- listed artifacts and receipts exist;
- the bundle verifies against deterministic generation;
- scanner-safe and runtime-material counts match the audit-surface receipt.

## What This Does Not Prove

`audit-bundle` does not prove:

- broader repo public claims by itself;
- release readiness;
- provider compatibility;
- production key management or production security;
- scanner evasion;
- downstream verifier correctness.

For broader repo public-claim proof, use a repo checkout:

```bash
cargo xtask verification-pack --out target/uselesskey-verification
```

`verification-pack` runs claim-proof for the included claims and copies the
receipts into the pack. Run `cargo xtask claim-proof --claim <id>` separately
only when a reviewer needs standalone claim-proof receipts.
