#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use uselesskey_core::Factory;
#[cfg(test)]
use uselesskey_test_support::{TestResult, ensure, ensure_eq, require_ok};
#[cfg(test)]
use uselesskey_webauthn::{AttestationMode, WebAuthnFactoryExt, WebAuthnSpec};

#[test]
fn webauthn_fixtures_exercise_registration_and_assertion_paths() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-webauthn-ceremony-validation");
    let mut spec = WebAuthnSpec::packed("login.example.com", b"challenge");
    spec.attestation_mode = AttestationMode::SelfAttestation;

    let registration = fx.webauthn_registration("alice", spec.clone());
    let assertion = fx.webauthn_assertion("alice", spec);

    let registration_client = client_data(&registration.client_data_json)?;
    ensure_eq!(registration_client["type"], "webauthn.create");
    ensure_eq!(registration_client["origin"], "https://login.example.com");
    ensure_eq!(registration_client["challenge"], "Y2hhbGxlbmdl");
    ensure_eq!(registration_client["crossOrigin"], false);

    let assertion_client = client_data(&assertion.client_data_json)?;
    ensure_eq!(assertion_client["type"], "webauthn.get");
    ensure_eq!(assertion_client["origin"], "https://login.example.com");
    ensure_eq!(assertion_client["challenge"], "Y2hhbGxlbmdl");

    ensure_eq!(
        &registration.authenticator_data[..32],
        &registration.rp_id_hash
    );
    ensure_eq!(registration.authenticator_data[32] & 0x41, 0x41);
    ensure_eq!(&assertion.authenticator_data[..32], &assertion.rp_id_hash);
    ensure_eq!(assertion.authenticator_data[32] & 0x01, 0x01);
    ensure_eq!(
        assertion.sign_count,
        registration.sign_count.saturating_add(1)
    );
    ensure_eq!(assertion.signature.len(), 32);
    Ok(())
}

#[test]
fn webauthn_negative_inputs_are_stable_without_committed_payloads() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-webauthn-negative-paths");
    let expected = fx.webauthn_registration(
        "alice",
        WebAuthnSpec::packed("login.example.com", b"challenge"),
    );
    let wrong_rp = fx.webauthn_registration(
        "alice",
        WebAuthnSpec::packed("attacker.example", b"challenge"),
    );

    ensure!(wrong_rp.rp_id_hash != expected.rp_id_hash);
    ensure!(wrong_rp.authenticator_data[..32] != expected.authenticator_data[..32]);

    let assertion = fx.webauthn_assertion(
        "alice",
        WebAuthnSpec::packed("login.example.com", b"challenge"),
    );
    let mut tampered_authenticator_data = assertion.authenticator_data.clone();
    tampered_authenticator_data[32] ^= 0x40;

    ensure!(tampered_authenticator_data != assertion.authenticator_data);
    Ok(())
}

#[cfg(test)]
fn client_data(bytes: &[u8]) -> TestResult<Value> {
    require_ok(serde_json::from_slice(bytes), "clientDataJSON parses")
}
