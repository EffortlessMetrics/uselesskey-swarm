# PKCS#11 Mock Validation Example

Use this example when downstream HSM-adjacent tests need deterministic slot,
token, key-handle, and signature-buffer shapes without depending on a real
PKCS#11 provider.

```toml
[dev-dependencies]
uselesskey-core = { version = "0.9.1", default-features = false }
uselesskey-pkcs11-mock = "0.9.1"
```

The test crate covers:

- deterministic slot and token metadata;
- stable one-based key handles and labels;
- valid sign/verify round trips;
- unknown-handle rejection;
- signature mismatch rejection;
- DER-looking certificate bytes kept inside the test process.

Run it from this repo with:

```bash
cargo xtask external-adoption-smoke --path . --library-examples
```

## Boundary

These fixtures are HSM-shaped parser and adapter inputs. They do not provide a
real cryptoki implementation, C ABI, FIPS validation, production HSM behavior,
provider compatibility, or production signing security.
