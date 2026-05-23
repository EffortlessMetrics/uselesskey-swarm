# WebAuthn Ceremony Validation Example

Use this example when a downstream validator needs scanner-safe WebAuthn
registration and assertion shapes without committing authenticator payloads.

```toml
[dev-dependencies]
uselesskey-core = "0.9.1"
uselesskey-webauthn = "0.9.1"
```

The test crate covers:

- registration `clientDataJSON` type, origin, and challenge checks;
- assertion `clientDataJSON` type and monotonic sign count;
- RP ID hash placement in authenticator data;
- adjacent negative inputs for wrong RP ID and tampered authenticator data.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

These fixtures are parser and verifier-shape inputs. They do not simulate CTAP,
platform authenticators, FIDO Metadata Service attestation, production passkeys,
or real ECDSA signatures.
