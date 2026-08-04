#[cfg(test)]
use ssh_key::{Certificate, PrivateKey, PublicKey, certificate::CertType};
#[cfg(test)]
use uselesskey_core::Factory;
#[cfg(test)]
use uselesskey_ssh::{
    SshCertFactoryExt, SshCertSpec, SshCertType, SshFactoryExt, SshSpec, SshValidity,
};
#[cfg(test)]
use uselesskey_test_support::{TestResult, ensure, ensure_eq, require_ok};

#[test]
fn ssh_key_fixtures_exercise_private_and_authorized_key_paths() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-ssh-key-validation");

    let ed25519 = fx.ssh_key("deploy-ed25519", SshSpec::ed25519());
    ensure!(ed25519.authorized_key_line().starts_with("ssh-ed25519 "));
    assert_private_and_public_round_trip(&ed25519)?;

    let rsa = fx.ssh_key("deploy-rsa", SshSpec::rsa());
    ensure!(rsa.authorized_key_line().starts_with("ssh-rsa "));
    assert_private_and_public_round_trip(&rsa)?;
    Ok(())
}

#[test]
fn ssh_certificate_fixtures_exercise_principals_and_validity_paths() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-ssh-cert-validation");
    let spec = SshCertSpec {
        principals: vec!["deploy".to_string(), "ci".to_string()],
        validity: SshValidity::new(1_700_000_000, 1_700_000_600),
        cert_type: SshCertType::User,
        critical_options: vec![(
            "force-command".to_string(),
            "/usr/local/bin/deploy".to_string(),
        )],
        extensions: vec![("permit-pty".to_string(), String::new())],
    };

    let fixture = fx.ssh_cert("deploy-user-cert", spec.clone());
    let parsed = require_ok(
        Certificate::from_openssh(fixture.certificate_openssh()),
        "SSH cert parses",
    )?;

    ensure_eq!(parsed.cert_type(), CertType::User);
    ensure_eq!(parsed.valid_principals(), spec.principals.as_slice());
    ensure_eq!(parsed.valid_after(), spec.validity.valid_after);
    ensure_eq!(parsed.valid_before(), spec.validity.valid_before);
    ensure_eq!(
        parsed
            .critical_options()
            .get("force-command")
            .map(String::as_str),
        Some("/usr/local/bin/deploy")
    );
    ensure_eq!(
        parsed.extensions().get("permit-pty").map(String::as_str),
        Some("")
    );
    Ok(())
}

#[test]
fn ssh_fixture_negative_inputs_are_stable_without_committed_payloads() -> TestResult<()> {
    let fx = Factory::deterministic_from_str("external-ssh-negative-paths");
    let key = fx.ssh_key("deploy", SshSpec::ed25519());

    let tampered_authorized_key = key
        .authorized_key_line()
        .replacen("ssh-ed25519", "ssh-rsa", 1);
    ensure!(PublicKey::from_openssh(&tampered_authorized_key).is_err());

    let validity = SshValidity::new(1_700_000_000, 1_700_000_600);
    let user_cert = fx.ssh_cert("deploy-user", SshCertSpec::user(["deploy"], validity));
    let host_cert = fx.ssh_cert(
        "deploy-host",
        SshCertSpec::host(["deploy.internal"], validity),
    );

    let parsed_user = require_ok(
        Certificate::from_openssh(user_cert.certificate_openssh()),
        "user cert parses",
    )?;
    let parsed_host = require_ok(
        Certificate::from_openssh(host_cert.certificate_openssh()),
        "host cert parses",
    )?;

    ensure_eq!(parsed_user.cert_type(), CertType::User);
    ensure_eq!(parsed_host.cert_type(), CertType::Host);
    ensure!(parsed_user.valid_principals() != parsed_host.valid_principals());

    let debug = format!("{key:?}");
    ensure!(!debug.contains("BEGIN OPENSSH PRIVATE KEY"));
    ensure!(!debug.contains("ssh-ed25519 "));
    Ok(())
}

#[cfg(test)]
fn assert_private_and_public_round_trip(key: &uselesskey_ssh::SshKeyPair) -> TestResult<()> {
    let private = require_ok(
        PrivateKey::from_openssh(key.private_key_openssh()),
        "OpenSSH private key parses",
    )?;
    let public = require_ok(
        PublicKey::from_openssh(key.authorized_key_line()),
        "authorized_keys line parses",
    )?;

    let private_public = require_ok(
        private.public_key().to_openssh(),
        "private public key encodes",
    )?;
    let authorized_public = require_ok(public.to_openssh(), "authorized key encodes")?;
    ensure_eq!(private_public, authorized_public);
    Ok(())
}
