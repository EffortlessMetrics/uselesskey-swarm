# SSH Fixture Validation Example

Use this example when downstream deployment, bastion, or SSH-CA adjacent tests
need deterministic OpenSSH private keys, `authorized_keys` lines, and
OpenSSH certificate shapes without committing generated key material.

```toml
[dev-dependencies]
ssh-key = { version = "0.6.7", default-features = false, features = ["std", "ed25519", "rsa"] }
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-ssh = "0.9.1"
```

The test crate covers:

- Ed25519 and RSA OpenSSH private-key parsing;
- `authorized_keys` public-key line parsing;
- deterministic user-certificate principals and validity windows;
- host-certificate vs user-certificate rejection hooks;
- tampered public-key line rejection;
- debug output that omits generated key material.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

These fixtures are parser and policy inputs for tests. They do not prove
OpenSSH daemon or client policy, SSH CA operations, host authorization,
production key custody, provider compatibility, release readiness, downstream
verifier correctness, or production security.
