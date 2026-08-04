#[cfg(test)]
use uselesskey_core::Factory;
#[cfg(test)]
use uselesskey_pkcs11_mock::{KeyHandle, Pkcs11MockFactoryExt, Pkcs11MockSpec};
#[cfg(test)]
use uselesskey_test_support::{TestResult, ensure, ensure_eq, require_some};

#[test]
fn pkcs11_mock_exercises_slot_key_and_signature_paths() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-pkcs11-mock-validation");
    let mut spec = Pkcs11MockSpec::basic("HSM-APP");
    spec.key_labels = vec!["signing-key".to_string(), "rotation-key".to_string()];

    let provider = fx.pkcs11_mock("tenant-a", spec);
    let slot = provider.slot_info();
    ensure_eq!(slot.token_label, "HSM-APP");
    ensure_eq!(slot.manufacturer_id, "uselesskey");
    ensure_eq!(slot.model, "UK-PKCS11-MOCK");
    ensure_eq!(slot.serial_number.len(), 16);

    let handles = provider.key_handles();
    ensure_eq!(handles, vec![KeyHandle(1), KeyHandle(2)]);
    ensure_eq!(provider.key_label(KeyHandle(1)), Some("signing-key"));
    ensure_eq!(provider.key_label(KeyHandle(2)), Some("rotation-key"));

    let message = b"statement to sign";
    let signature = require_some(provider.sign(KeyHandle(1), message), "known handle signs")?;
    ensure!(provider.verify(KeyHandle(1), message, &signature));
    ensure!(!provider.verify(KeyHandle(1), b"tampered message", &signature));

    let certificate = require_some(
        provider.certificate_der(KeyHandle(1)),
        "known handle has certificate bytes",
    )?;
    ensure_eq!(certificate.get(..2), Some(&[0x30, 0x82][..]));
    Ok(())
}

#[test]
fn pkcs11_mock_negative_paths_are_stable_without_committed_payloads() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-pkcs11-negative-paths");
    let provider = fx.pkcs11_mock("tenant-a", Pkcs11MockSpec::basic("HSM-APP"));
    let valid = *require_some(provider.key_handles().first(), "first key handle")?;
    let signature = require_some(provider.sign(valid, b"payload"), "known handle signs")?;

    ensure!(provider.sign(KeyHandle(999), b"payload").is_none());
    ensure!(!provider.verify(KeyHandle(999), b"payload", &signature));
    ensure!(provider.key_label(KeyHandle(999)).is_none());
    ensure!(provider.certificate_der(KeyHandle(999)).is_none());
    ensure_eq!(provider.next_sign_count(), 1);
    ensure_eq!(provider.next_sign_count(), 2);
    Ok(())
}
