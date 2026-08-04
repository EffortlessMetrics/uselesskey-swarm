#[cfg(test)]
use uselesskey_core::Factory;
#[cfg(test)]
use uselesskey_core::negative::CorruptPem;
#[cfg(test)]
use uselesskey_ecdsa::{EcdsaFactoryExt, EcdsaSpec};
#[cfg(test)]
use uselesskey_test_support::{TestResult, ensure, ensure_eq, require_ok};

#[test]
fn ecdsa_fixtures_exercise_es256_pkcs8_and_spki_parse_paths() -> TestResult<()> {
    use p256::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};

    let fx = Factory::deterministic_from_str("external-ecdsa-es256-parse");
    let key = fx.ecdsa("release-signer-es256", EcdsaSpec::es256());

    ensure_eq!(key.label(), "release-signer-es256");
    ensure_eq!(key.spec(), EcdsaSpec::es256());
    ensure!(key.private_key_pkcs8_pem().contains("BEGIN PRIVATE KEY"));
    ensure!(key.public_key_spki_pem().contains("BEGIN PUBLIC KEY"));

    require_ok(
        p256::SecretKey::from_pkcs8_pem(key.private_key_pkcs8_pem()),
        "ES256 private PEM parses",
    )?;
    require_ok(
        p256::PublicKey::from_public_key_pem(key.public_key_spki_pem()),
        "ES256 public PEM parses",
    )?;
    require_ok(
        p256::SecretKey::from_pkcs8_der(key.private_key_pkcs8_der()),
        "ES256 private DER parses",
    )?;
    require_ok(
        p256::PublicKey::from_public_key_der(key.public_key_spki_der()),
        "ES256 public DER parses",
    )?;

    Ok(())
}

#[test]
fn ecdsa_fixtures_exercise_es384_pkcs8_and_spki_parse_paths() -> TestResult<()> {
    use p384::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};

    let fx = Factory::deterministic_from_str("external-ecdsa-es384-parse");
    let key = fx.ecdsa("release-signer-es384", EcdsaSpec::es384());

    ensure_eq!(key.label(), "release-signer-es384");
    ensure_eq!(key.spec(), EcdsaSpec::es384());

    require_ok(
        p384::SecretKey::from_pkcs8_pem(key.private_key_pkcs8_pem()),
        "ES384 private PEM parses",
    )?;
    require_ok(
        p384::PublicKey::from_public_key_pem(key.public_key_spki_pem()),
        "ES384 public PEM parses",
    )?;
    require_ok(
        p384::SecretKey::from_pkcs8_der(key.private_key_pkcs8_der()),
        "ES384 private DER parses",
    )?;
    require_ok(
        p384::PublicKey::from_public_key_der(key.public_key_spki_der()),
        "ES384 public DER parses",
    )?;

    Ok(())
}

#[test]
fn ecdsa_fixture_negative_inputs_are_stable_without_committed_payloads() -> TestResult<()> {
    use p256::pkcs8::{DecodePrivateKey as _, DecodePublicKey as _};

    let fx = Factory::deterministic_from_str("external-ecdsa-negatives");
    let key = fx.ecdsa("policy-signer-es256", EcdsaSpec::es256());

    let corrupt_pem = key.private_key_pkcs8_pem_corrupt(CorruptPem::BadBase64);
    ensure!(corrupt_pem != key.private_key_pkcs8_pem());
    ensure!(p256::SecretKey::from_pkcs8_pem(&corrupt_pem).is_err());

    let truncated_der = key.private_key_pkcs8_der_truncated(16);
    ensure_eq!(truncated_der.len(), 16);
    ensure!(p256::SecretKey::from_pkcs8_der(&truncated_der).is_err());

    let mismatched_der = key.mismatched_public_key_spki_der();
    ensure!(mismatched_der != key.public_key_spki_der());
    require_ok(
        p256::PublicKey::from_public_key_der(&mismatched_der),
        "mismatched ES256 public key parses",
    )?;

    Ok(())
}

#[test]
fn ecdsa_fixture_debug_output_omits_key_material() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-ecdsa-debug");
    let key = fx.ecdsa("debug-signer-es384", EcdsaSpec::es384());
    let debug = format!("{key:?}");

    ensure!(debug.contains("EcdsaKeyPair"));
    ensure!(debug.contains("debug-signer-es384"));
    ensure!(!debug.contains("BEGIN PRIVATE KEY"));
    ensure!(!debug.contains("BEGIN PUBLIC KEY"));
    ensure!(!debug.contains(&format!("{:?}", key.private_key_pkcs8_der())));
    ensure!(!debug.contains(&format!("{:?}", key.public_key_spki_der())));

    Ok(())
}
