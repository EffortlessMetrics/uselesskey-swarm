use serde_json::Value;
use uselesskey_core::Factory;
use uselesskey_webauthn::{AttestationMode, WebAuthnFactoryExt, WebAuthnSpec};

#[test]
fn webauthn_fixtures_exercise_registration_and_assertion_paths() {
    let fx = Factory::deterministic_from_str("external-webauthn-ceremony-validation");
    let mut spec = WebAuthnSpec::packed("login.example.com", b"challenge");
    spec.attestation_mode = AttestationMode::SelfAttestation;

    let registration = fx.webauthn_registration("alice", spec.clone());
    let assertion = fx.webauthn_assertion("alice", spec);

    let registration_client = client_data(&registration.client_data_json);
    assert_eq!(registration_client["type"], "webauthn.create");
    assert_eq!(registration_client["origin"], "https://login.example.com");
    assert_eq!(registration_client["challenge"], "Y2hhbGxlbmdl");
    assert_eq!(registration_client["crossOrigin"], false);

    let assertion_client = client_data(&assertion.client_data_json);
    assert_eq!(assertion_client["type"], "webauthn.get");
    assert_eq!(assertion_client["origin"], "https://login.example.com");
    assert_eq!(assertion_client["challenge"], "Y2hhbGxlbmdl");

    assert_eq!(&registration.authenticator_data[..32], &registration.rp_id_hash);
    assert_eq!(registration.authenticator_data[32] & 0x41, 0x41);
    assert_eq!(&assertion.authenticator_data[..32], &assertion.rp_id_hash);
    assert_eq!(assertion.authenticator_data[32] & 0x01, 0x01);
    assert_eq!(assertion.sign_count, registration.sign_count.saturating_add(1));
    assert_eq!(assertion.signature.len(), 32);
}

#[test]
fn webauthn_negative_inputs_are_stable_without_committed_payloads() {
    let fx = Factory::deterministic_from_str("external-webauthn-negative-paths");
    let expected = fx.webauthn_registration(
        "alice",
        WebAuthnSpec::packed("login.example.com", b"challenge"),
    );
    let wrong_rp = fx.webauthn_registration(
        "alice",
        WebAuthnSpec::packed("attacker.example", b"challenge"),
    );

    assert_ne!(wrong_rp.rp_id_hash, expected.rp_id_hash);
    assert_ne!(
        &wrong_rp.authenticator_data[..32],
        &expected.authenticator_data[..32]
    );

    let assertion = fx.webauthn_assertion(
        "alice",
        WebAuthnSpec::packed("login.example.com", b"challenge"),
    );
    let mut tampered_authenticator_data = assertion.authenticator_data.clone();
    tampered_authenticator_data[32] ^= 0x40;

    assert_ne!(tampered_authenticator_data, assertion.authenticator_data);
}

fn client_data(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("clientDataJSON parses")
}
