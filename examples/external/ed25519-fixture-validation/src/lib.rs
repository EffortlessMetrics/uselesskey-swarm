use ed25519_dalek::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};
use ed25519_dalek::{SigningKey, VerifyingKey};
use uselesskey_core::Factory;
use uselesskey_core::negative::CorruptPem;
use uselesskey_ed25519::{Ed25519FactoryExt, Ed25519Spec};

#[test]
fn ed25519_fixtures_exercise_pkcs8_and_spki_parse_paths() {
    let fx = Factory::deterministic_from_str("external-ed25519-parse");
    let key = fx.ed25519("release-signer", Ed25519Spec::new());

    assert_eq!(key.label(), "release-signer");
    assert_eq!(key.spec(), Ed25519Spec::new());
    assert!(key.private_key_pkcs8_pem().contains("BEGIN PRIVATE KEY"));
    assert!(key.public_key_spki_pem().contains("BEGIN PUBLIC KEY"));

    let private_pem =
        SigningKey::from_pkcs8_pem(key.private_key_pkcs8_pem()).expect("private PEM parses");
    let public_pem =
        VerifyingKey::from_public_key_pem(key.public_key_spki_pem()).expect("public PEM parses");
    assert_eq!(private_pem.verifying_key().as_bytes(), public_pem.as_bytes());

    let private_der =
        SigningKey::from_pkcs8_der(key.private_key_pkcs8_der()).expect("private DER parses");
    let public_der =
        VerifyingKey::from_public_key_der(key.public_key_spki_der()).expect("public DER parses");
    assert_eq!(private_der.verifying_key().as_bytes(), public_der.as_bytes());
}

#[test]
fn ed25519_fixture_negative_inputs_are_stable_without_committed_payloads() {
    let fx = Factory::deterministic_from_str("external-ed25519-negatives");
    let key = fx.ed25519("policy-signer", Ed25519Spec::new());

    let corrupt_pem = key.private_key_pkcs8_pem_corrupt(CorruptPem::BadBase64);
    assert_ne!(corrupt_pem, key.private_key_pkcs8_pem());
    assert!(SigningKey::from_pkcs8_pem(&corrupt_pem).is_err());

    let truncated_der = key.private_key_pkcs8_der_truncated(16);
    assert_eq!(truncated_der.len(), 16);
    assert!(SigningKey::from_pkcs8_der(&truncated_der).is_err());

    let mismatched_der = key.mismatched_public_key_spki_der();
    let mismatched_public =
        VerifyingKey::from_public_key_der(&mismatched_der).expect("mismatched public key parses");
    let original_public =
        VerifyingKey::from_public_key_der(key.public_key_spki_der()).expect("public key parses");
    assert_ne!(mismatched_public.as_bytes(), original_public.as_bytes());
}

#[test]
fn ed25519_fixture_debug_output_omits_key_material() {
    let fx = Factory::deterministic_from_str("external-ed25519-debug");
    let key = fx.ed25519("debug-signer", Ed25519Spec::new());
    let debug = format!("{key:?}");

    assert!(debug.contains("Ed25519KeyPair"));
    assert!(debug.contains("debug-signer"));
    assert!(!debug.contains("BEGIN PRIVATE KEY"));
    assert!(!debug.contains("BEGIN PUBLIC KEY"));
    assert!(!debug.contains(&format!("{:?}", key.private_key_pkcs8_der())));
    assert!(!debug.contains(&format!("{:?}", key.public_key_spki_der())));
}
