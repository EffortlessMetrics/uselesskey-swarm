#[cfg(test)]
use ed25519_dalek::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};
#[cfg(test)]
use ed25519_dalek::{SigningKey, VerifyingKey};
#[cfg(test)]
use uselesskey_core::Factory;
#[cfg(test)]
use uselesskey_core::negative::CorruptPem;
#[cfg(test)]
use uselesskey_ed25519::{Ed25519FactoryExt, Ed25519Spec};
#[cfg(test)]
use uselesskey_test_support::{TestResult, ensure, ensure_eq, require_ok};

#[test]
fn ed25519_fixtures_exercise_pkcs8_and_spki_parse_paths() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-ed25519-parse");
    let key = fx.ed25519("release-signer", Ed25519Spec::new());

    ensure_eq!(key.label(), "release-signer");
    ensure_eq!(key.spec(), Ed25519Spec::new());
    ensure!(key.private_key_pkcs8_pem().contains("BEGIN PRIVATE KEY"));
    ensure!(key.public_key_spki_pem().contains("BEGIN PUBLIC KEY"));

    let private_pem = require_ok(
        SigningKey::from_pkcs8_pem(key.private_key_pkcs8_pem()),
        "private PEM parses",
    )?;
    let public_pem = require_ok(
        VerifyingKey::from_public_key_pem(key.public_key_spki_pem()),
        "public PEM parses",
    )?;
    ensure_eq!(
        private_pem.verifying_key().as_bytes(),
        public_pem.as_bytes()
    );

    let private_der = require_ok(
        SigningKey::from_pkcs8_der(key.private_key_pkcs8_der()),
        "private DER parses",
    )?;
    let public_der = require_ok(
        VerifyingKey::from_public_key_der(key.public_key_spki_der()),
        "public DER parses",
    )?;
    ensure_eq!(
        private_der.verifying_key().as_bytes(),
        public_der.as_bytes()
    );

    Ok(())
}

#[test]
fn ed25519_fixture_negative_inputs_are_stable_without_committed_payloads() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-ed25519-negatives");
    let key = fx.ed25519("policy-signer", Ed25519Spec::new());

    let corrupt_pem = key.private_key_pkcs8_pem_corrupt(CorruptPem::BadBase64);
    ensure!(corrupt_pem != key.private_key_pkcs8_pem());
    ensure!(SigningKey::from_pkcs8_pem(&corrupt_pem).is_err());

    let truncated_der = key.private_key_pkcs8_der_truncated(16);
    ensure_eq!(truncated_der.len(), 16);
    ensure!(SigningKey::from_pkcs8_der(&truncated_der).is_err());

    let mismatched_der = key.mismatched_public_key_spki_der();
    let mismatched_public = require_ok(
        VerifyingKey::from_public_key_der(&mismatched_der),
        "mismatched public key parses",
    )?;
    let original_public = require_ok(
        VerifyingKey::from_public_key_der(key.public_key_spki_der()),
        "public key parses",
    )?;
    ensure!(mismatched_public.as_bytes() != original_public.as_bytes());

    Ok(())
}

#[test]
fn ed25519_fixture_debug_output_omits_key_material() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-ed25519-debug");
    let key = fx.ed25519("debug-signer", Ed25519Spec::new());
    let debug = format!("{key:?}");

    ensure!(debug.contains("Ed25519KeyPair"));
    ensure!(debug.contains("debug-signer"));
    ensure!(!debug.contains("BEGIN PRIVATE KEY"));
    ensure!(!debug.contains("BEGIN PUBLIC KEY"));
    ensure!(!debug.contains(&format!("{:?}", key.private_key_pkcs8_der())));
    ensure!(!debug.contains(&format!("{:?}", key.public_key_spki_der())));

    Ok(())
}
