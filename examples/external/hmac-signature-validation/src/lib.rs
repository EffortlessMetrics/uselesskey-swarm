use sha2::{Digest, Sha256, Sha384, Sha512};
use uselesskey_core::Factory;
use uselesskey_hmac::{HmacFactoryExt, HmacSecret, HmacSpec};

#[test]
fn hmac_fixtures_exercise_secret_lengths_and_algorithm_paths() {
    let fx = Factory::deterministic_from_str("external-hmac-shape-validation");

    for (label, spec, alg, byte_len) in [
        ("webhook-hs256", HmacSpec::hs256(), "HS256", 32),
        ("jwt-hs384", HmacSpec::hs384(), "HS384", 48),
        ("session-hs512", HmacSpec::hs512(), "HS512", 64),
    ] {
        let secret = fx.hmac(label, spec);

        assert_eq!(secret.label(), label);
        assert_eq!(secret.spec().alg_name(), alg);
        assert_eq!(secret.secret_bytes().len(), byte_len);
    }
}

#[test]
fn hmac_fixtures_drive_positive_and_negative_signature_policy() {
    let fx = Factory::deterministic_from_str("external-hmac-policy-validation");
    let secret = fx.hmac("webhook-shared-secret", HmacSpec::hs256());
    let message = br#"{"event":"invoice.paid","id":"evt_test"}"#;
    let signature = fixture_signature_for_policy_test(&secret, message);

    assert!(verify_fixture_signature(&secret, message, &signature));
    assert!(!verify_fixture_signature(
        &secret,
        br#"{"event":"invoice.voided","id":"evt_test"}"#,
        &signature
    ));

    let wrong_secret = fx.hmac("other-webhook-secret", HmacSpec::hs256());
    assert!(!verify_fixture_signature(
        &wrong_secret,
        message,
        &signature
    ));

    let wrong_algorithm = fx.hmac("webhook-shared-secret", HmacSpec::hs512());
    assert!(!verify_fixture_signature(
        &wrong_algorithm,
        message,
        &signature
    ));
}

#[test]
fn hmac_fixture_debug_output_omits_secret_material() {
    let fx = Factory::deterministic_from_str("external-hmac-debug-validation");
    let secret = fx.hmac("debug-webhook-secret", HmacSpec::hs384());
    let debug = format!("{secret:?}");

    assert!(debug.contains("HmacSecret"));
    assert!(debug.contains("debug-webhook-secret"));
    assert!(!debug.contains(&format!("{:?}", secret.secret_bytes())));
}

fn verify_fixture_signature(secret: &HmacSecret, message: &[u8], expected: &[u8]) -> bool {
    fixture_signature_for_policy_test(secret, message) == expected
}

fn fixture_signature_for_policy_test(secret: &HmacSecret, message: &[u8]) -> Vec<u8> {
    match secret.spec() {
        HmacSpec::Hs256 => {
            let mut digest = Sha256::new();
            digest.update(secret.spec().alg_name().as_bytes());
            digest.update(secret.secret_bytes());
            digest.update(message);
            digest.finalize().to_vec()
        }
        HmacSpec::Hs384 => {
            let mut digest = Sha384::new();
            digest.update(secret.spec().alg_name().as_bytes());
            digest.update(secret.secret_bytes());
            digest.update(message);
            digest.finalize().to_vec()
        }
        HmacSpec::Hs512 => {
            let mut digest = Sha512::new();
            digest.update(secret.spec().alg_name().as_bytes());
            digest.update(secret.secret_bytes());
            digest.update(message);
            digest.finalize().to_vec()
        }
    }
}
