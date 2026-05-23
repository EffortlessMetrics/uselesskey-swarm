use std::io::Cursor;

use pgp::composed::{Deserializable, SignedPublicKey, SignedSecretKey};
use pgp::types::KeyDetails;
use uselesskey_core::Factory;
use uselesskey_core::negative::CorruptPem;
use uselesskey_pgp::{PgpFactoryExt, PgpSpec};

#[test]
fn pgp_armored_fixtures_exercise_private_and_public_parse_paths() {
    let fx = Factory::deterministic_from_str("external-pgp-armored-validation");
    let key = fx.pgp("release signer", PgpSpec::ed25519());

    assert!(key.private_key_armored().contains("BEGIN PGP PRIVATE KEY BLOCK"));
    assert!(key.public_key_armored().contains("BEGIN PGP PUBLIC KEY BLOCK"));
    assert_eq!(key.user_id(), "release signer <release-signer@uselesskey.test>");

    let (secret, _) = SignedSecretKey::from_armor_single(Cursor::new(key.private_key_armored()))
        .expect("armored private key parses");
    let (public, _) = SignedPublicKey::from_armor_single(Cursor::new(key.public_key_armored()))
        .expect("armored public key parses");

    assert_eq!(secret.fingerprint().to_string(), key.fingerprint());
    assert_eq!(public.fingerprint().to_string(), key.fingerprint());
}

#[test]
fn pgp_binary_fixtures_exercise_transferable_key_paths() {
    let fx = Factory::deterministic_from_str("external-pgp-binary-validation");
    let key = fx.pgp("binary signer", PgpSpec::ed25519());

    let secret = SignedSecretKey::from_bytes(Cursor::new(key.private_key_binary()))
        .expect("binary private key parses");
    let public = SignedPublicKey::from_bytes(Cursor::new(key.public_key_binary()))
        .expect("binary public key parses");

    assert_eq!(secret.fingerprint().to_string(), key.fingerprint());
    assert_eq!(public.fingerprint().to_string(), key.fingerprint());
    assert!(!key.private_key_binary().is_empty());
    assert!(!key.public_key_binary().is_empty());
}

#[test]
fn pgp_fixture_negative_inputs_are_stable_without_committed_payloads() {
    let fx = Factory::deterministic_from_str("external-pgp-negative-paths");
    let key = fx.pgp("policy signer", PgpSpec::ed25519());

    let corrupt = key.private_key_armored_corrupt(CorruptPem::BadBase64);
    assert_ne!(corrupt, key.private_key_armored());
    assert!(SignedSecretKey::from_armor_single(Cursor::new(&corrupt)).is_err());

    let truncated = key.private_key_binary_truncated(32);
    assert_eq!(truncated.len(), 32);
    assert!(SignedSecretKey::from_bytes(Cursor::new(&truncated)).is_err());

    let mismatched = key.mismatched_public_key_armored();
    let (mismatched_public, _) =
        SignedPublicKey::from_armor_single(Cursor::new(&mismatched))
            .expect("mismatched public key is still parseable");
    assert_ne!(mismatched_public.fingerprint().to_string(), key.fingerprint());

    let debug = format!("{key:?}");
    assert!(!debug.contains("BEGIN PGP PRIVATE KEY BLOCK"));
    assert!(!debug.contains("BEGIN PGP PUBLIC KEY BLOCK"));
}
