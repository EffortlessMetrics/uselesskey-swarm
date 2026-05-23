use uselesskey_core::Factory;
use uselesskey_pkcs11_mock::{KeyHandle, Pkcs11MockFactoryExt, Pkcs11MockSpec};

#[test]
fn pkcs11_mock_exercises_slot_key_and_signature_paths() {
    let fx = Factory::deterministic_from_str("external-pkcs11-mock-validation");
    let mut spec = Pkcs11MockSpec::basic("HSM-APP");
    spec.key_labels = vec!["signing-key".to_string(), "rotation-key".to_string()];

    let provider = fx.pkcs11_mock("tenant-a", spec);
    let slot = provider.slot_info();
    assert_eq!(slot.token_label, "HSM-APP");
    assert_eq!(slot.manufacturer_id, "uselesskey");
    assert_eq!(slot.model, "UK-PKCS11-MOCK");
    assert_eq!(slot.serial_number.len(), 16);

    let handles = provider.key_handles();
    assert_eq!(handles, vec![KeyHandle(1), KeyHandle(2)]);
    assert_eq!(provider.key_label(KeyHandle(1)), Some("signing-key"));
    assert_eq!(provider.key_label(KeyHandle(2)), Some("rotation-key"));

    let message = b"statement to sign";
    let signature = provider
        .sign(KeyHandle(1), message)
        .expect("known handle signs");
    assert!(provider.verify(KeyHandle(1), message, &signature));
    assert!(!provider.verify(KeyHandle(1), b"tampered message", &signature));

    let certificate = provider
        .certificate_der(KeyHandle(1))
        .expect("known handle has certificate bytes");
    assert_eq!(&certificate[..2], &[0x30, 0x82]);
}

#[test]
fn pkcs11_mock_negative_paths_are_stable_without_committed_payloads() {
    let fx = Factory::deterministic_from_str("external-pkcs11-negative-paths");
    let provider = fx.pkcs11_mock("tenant-a", Pkcs11MockSpec::basic("HSM-APP"));
    let valid = provider.key_handles()[0];
    let signature = provider.sign(valid, b"payload").expect("known handle signs");

    assert!(provider.sign(KeyHandle(999), b"payload").is_none());
    assert!(!provider.verify(KeyHandle(999), b"payload", &signature));
    assert!(provider.key_label(KeyHandle(999)).is_none());
    assert!(provider.certificate_der(KeyHandle(999)).is_none());
    assert_eq!(provider.next_sign_count(), 1);
    assert_eq!(provider.next_sign_count(), 2);
}
