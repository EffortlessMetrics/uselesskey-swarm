#[cfg(test)]
use std::io::Cursor;

#[cfg(test)]
use pgp::composed::{Deserializable, SignedPublicKey, SignedSecretKey};
#[cfg(test)]
use pgp::types::KeyDetails;
#[cfg(test)]
use uselesskey_core::Factory;
#[cfg(test)]
use uselesskey_core::negative::CorruptPem;
#[cfg(test)]
use uselesskey_pgp::{PgpFactoryExt, PgpSpec};
#[cfg(test)]
use uselesskey_test_support::{TestResult, ensure, ensure_eq, require_ok};

#[test]
fn pgp_armored_fixtures_exercise_private_and_public_parse_paths() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-pgp-armored-validation");
    let key = fx.pgp("release signer", PgpSpec::ed25519());

    ensure!(
        key.private_key_armored()
            .contains("BEGIN PGP PRIVATE KEY BLOCK")
    );
    ensure!(
        key.public_key_armored()
            .contains("BEGIN PGP PUBLIC KEY BLOCK")
    );
    ensure_eq!(
        key.user_id(),
        "release signer <release-signer@uselesskey.test>"
    );

    let (secret, _) = require_ok(
        SignedSecretKey::from_armor_single(Cursor::new(key.private_key_armored())),
        "armored private key parses",
    )?;
    let (public, _) = require_ok(
        SignedPublicKey::from_armor_single(Cursor::new(key.public_key_armored())),
        "armored public key parses",
    )?;

    ensure_eq!(secret.fingerprint().to_string(), key.fingerprint());
    ensure_eq!(public.fingerprint().to_string(), key.fingerprint());
    Ok(())
}

#[test]
fn pgp_binary_fixtures_exercise_transferable_key_paths() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-pgp-binary-validation");
    let key = fx.pgp("binary signer", PgpSpec::ed25519());

    let secret = require_ok(
        SignedSecretKey::from_bytes(Cursor::new(key.private_key_binary())),
        "binary private key parses",
    )?;
    let public = require_ok(
        SignedPublicKey::from_bytes(Cursor::new(key.public_key_binary())),
        "binary public key parses",
    )?;

    ensure_eq!(secret.fingerprint().to_string(), key.fingerprint());
    ensure_eq!(public.fingerprint().to_string(), key.fingerprint());
    ensure!(!key.private_key_binary().is_empty());
    ensure!(!key.public_key_binary().is_empty());
    Ok(())
}

#[test]
fn pgp_fixture_negative_inputs_are_stable_without_committed_payloads() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-pgp-negative-paths");
    let key = fx.pgp("policy signer", PgpSpec::ed25519());

    let corrupt = key.private_key_armored_corrupt(CorruptPem::BadBase64);
    ensure!(corrupt != key.private_key_armored());
    ensure!(SignedSecretKey::from_armor_single(Cursor::new(&corrupt)).is_err());

    let truncated = key.private_key_binary_truncated(32);
    ensure_eq!(truncated.len(), 32);
    ensure!(SignedSecretKey::from_bytes(Cursor::new(&truncated)).is_err());

    let mismatched = key.mismatched_public_key_armored();
    let (mismatched_public, _) = require_ok(
        SignedPublicKey::from_armor_single(Cursor::new(&mismatched)),
        "mismatched public key parses",
    )?;
    ensure!(mismatched_public.fingerprint().to_string() != key.fingerprint());

    let debug = format!("{key:?}");
    ensure!(!debug.contains("BEGIN PGP PRIVATE KEY BLOCK"));
    ensure!(!debug.contains("BEGIN PGP PUBLIC KEY BLOCK"));
    Ok(())
}
