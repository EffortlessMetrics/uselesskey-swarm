# Test SSH key and certificate validation

You are testing code that consumes OpenSSH private keys, `authorized_keys`
lines, user certificates, or host certificates. You need deterministic fixture
material that exercises the parser and policy path without committing generated
key payloads.

The `uselesskey-ssh` crate provides Ed25519 and RSA OpenSSH key fixtures plus
OpenSSH user and host certificate fixtures. A copyable downstream-style test
crate lives at
[`../../examples/external/ssh-fixture-validation/`](../../examples/external/ssh-fixture-validation/).

## Add the test dependencies

```toml
[dev-dependencies]
ssh-key = { version = "0.6.7", default-features = false, features = ["std", "ed25519", "rsa"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-ssh = "0.9.1"
```

Run the repo proof path with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Generate an SSH key fixture

```rust
use ssh_key::{PrivateKey, PublicKey};
use uselesskey_core::{Factory, Seed};
use uselesskey_ssh::{SshFactoryExt, SshSpec};

let fx = Factory::deterministic(Seed::from_env_value("ssh-test").unwrap());
let key = fx.ssh_key("deploy", SshSpec::ed25519());

let private = PrivateKey::from_openssh(key.private_key_openssh()).unwrap();
let public = PublicKey::from_openssh(key.authorized_key_line()).unwrap();

assert_eq!(
    private.public_key().to_openssh().unwrap(),
    public.to_openssh().unwrap()
);
```

Use `SshSpec::rsa()` when the code under test needs an `ssh-rsa` key shape.
The generated private key and public line are stable for the same seed, label,
and spec.

## Generate an SSH certificate fixture

```rust
use ssh_key::{Certificate, certificate::CertType};
use uselesskey_core::{Factory, Seed};
use uselesskey_ssh::{SshCertFactoryExt, SshCertSpec, SshValidity};

let fx = Factory::deterministic(Seed::from_env_value("ssh-cert-test").unwrap());
let spec = SshCertSpec::user(
    ["deploy", "ci"],
    SshValidity::new(1_700_000_000, 1_700_000_600),
);

let fixture = fx.ssh_cert("deploy-cert", spec.clone());
let cert = Certificate::from_openssh(fixture.certificate_openssh()).unwrap();

assert_eq!(cert.cert_type(), CertType::User);
assert_eq!(cert.valid_principals(), spec.principals.as_slice());
assert_eq!(cert.valid_after(), spec.validity.valid_after);
assert_eq!(cert.valid_before(), spec.validity.valid_before);
```

Use `SshCertSpec::host(["host.internal"], validity)` to exercise host
certificate policy.

## Test failure paths

Generate adjacent fixtures in the test instead of committing payloads:

- **Wrong key algorithm.** Parse an Ed25519 `authorized_keys` line in a path
  that expects RSA, or generate `SshSpec::rsa()` when the policy expects
  Ed25519.
- **Wrong certificate type.** Feed a host certificate to a user-certificate
  validation path, or the reverse.
- **Wrong principal.** Generate the certificate with a different username or
  hostname and assert that your policy rejects it.
- **Expired or not-yet-valid window.** Set `SshValidity` outside the time your
  verifier accepts.
- **Malformed public key line.** Tamper with the `authorized_keys` line in the
  test before parsing it.

## What this proves

- Your code can parse the OpenSSH private-key and public-key shapes it expects.
- Your code can parse OpenSSH certificates and inspect certificate type,
  principals, critical options, extensions, and validity windows.
- Your policy code rejects wrong algorithm, wrong principal, wrong certificate
  type, malformed public-key line, or invalid validity windows when your test
  asserts those cases.

## What this does not prove

- OpenSSH daemon or client authorization behavior.
- Production SSH key custody, rotation, or CA operation.
- Host authorization, bastion hardening, or infrastructure access policy.
- Provider compatibility with every SSH implementation.
- Release readiness, downstream verifier correctness, or production security.

## Scanner-safety boundary

Private keys and certificate payloads are generated at test time from
`Seed + (domain, label, spec, variant)`. Keep generated files under `target/`
or in memory, do not copy them into docs or source fixtures, and use
`cargo xtask no-blob` to catch accidental committed payloads.

## See also

- [`../../crates/uselesskey-ssh/README.md`](../../crates/uselesskey-ssh/README.md)
  - crate-level SSH fixture overview.
- [`../../examples/external/ssh-fixture-validation/`](../../examples/external/ssh-fixture-validation/)
  - copyable downstream test wiring.
- [`test-tls-chain-validation.md`](test-tls-chain-validation.md) - the TLS
  certificate-chain analogue.
