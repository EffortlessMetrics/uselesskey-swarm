use uselesskey_core::Factory;
use uselesskey_core::negative::CorruptPem;
use uselesskey_ecdsa::{EcdsaFactoryExt, EcdsaSpec};

#[test]
fn ecdsa_fixtures_exercise_es256_pkcs8_and_spki_parse_paths() {
    use p256::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};

    let fx = Factory::deterministic_from_str("external-ecdsa-es256-parse");
    let key = fx.ecdsa("release-signer-es256", EcdsaSpec::es256());

    assert_eq!(key.label(), "release-signer-es256");
    assert_eq!(key.spec(), EcdsaSpec::es256());
    assert!(key.private_key_pkcs8_pem().contains("BEGIN PRIVATE KEY"));
    assert!(key.public_key_spki_pem().contains("BEGIN PUBLIC KEY"));

    p256::SecretKey::from_pkcs8_pem(key.private_key_pkcs8_pem()).expect("ES256 private PEM parses");
    p256::PublicKey::from_public_key_pem(key.public_key_spki_pem())
        .expect("ES256 public PEM parses");
    p256::SecretKey::from_pkcs8_der(key.private_key_pkcs8_der()).expect("ES256 private DER parses");
    p256::PublicKey::from_public_key_der(key.public_key_spki_der())
        .expect("ES256 public DER parses");
}

#[test]
fn ecdsa_fixtures_exercise_es384_pkcs8_and_spki_parse_paths() {
    use p384::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};

    let fx = Factory::deterministic_from_str("external-ecdsa-es384-parse");
    let key = fx.ecdsa("release-signer-es384", EcdsaSpec::es384());

    assert_eq!(key.label(), "release-signer-es384");
    assert_eq!(key.spec(), EcdsaSpec::es384());

    p384::SecretKey::from_pkcs8_pem(key.private_key_pkcs8_pem()).expect("ES384 private PEM parses");
    p384::PublicKey::from_public_key_pem(key.public_key_spki_pem())
        .expect("ES384 public PEM parses");
    p384::SecretKey::from_pkcs8_der(key.private_key_pkcs8_der()).expect("ES384 private DER parses");
    p384::PublicKey::from_public_key_der(key.public_key_spki_der())
        .expect("ES384 public DER parses");
}

#[test]
fn ecdsa_fixture_negative_inputs_are_stable_without_committed_payloads() {
    use p256::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};

    let fx = Factory::deterministic_from_str("external-ecdsa-negatives");
    let key = fx.ecdsa("policy-signer-es256", EcdsaSpec::es256());

    let corrupt_pem = key.private_key_pkcs8_pem_corrupt(CorruptPem::BadBase64);
    assert_ne!(corrupt_pem, key.private_key_pkcs8_pem());
    assert!(p256::SecretKey::from_pkcs8_pem(&corrupt_pem).is_err());

    let truncated_der = key.private_key_pkcs8_der_truncated(16);
    assert_eq!(truncated_der.len(), 16);
    assert!(p256::SecretKey::from_pkcs8_der(&truncated_der).is_err());

    let mismatched_der = key.mismatched_public_key_spki_der();
    assert_ne!(mismatched_der, key.public_key_spki_der());
    p256::PublicKey::from_public_key_der(&mismatched_der)
        .expect("mismatched ES256 public key parses");
}

#[test]
fn ecdsa_fixture_debug_output_omits_key_material() {
    let fx = Factory::deterministic_from_str("external-ecdsa-debug");
    let key = fx.ecdsa("debug-signer-es384", EcdsaSpec::es384());
    let debug = format!("{key:?}");

    assert!(debug.contains("EcdsaKeyPair"));
    assert!(debug.contains("debug-signer-es384"));
    assert!(!debug.contains("BEGIN PRIVATE KEY"));
    assert!(!debug.contains("BEGIN PUBLIC KEY"));
    assert!(!debug.contains(&format!("{:?}", key.private_key_pkcs8_der())));
    assert!(!debug.contains(&format!("{:?}", key.public_key_spki_der())));
}
