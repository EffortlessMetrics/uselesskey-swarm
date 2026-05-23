use ssh_key::{Certificate, PrivateKey, PublicKey, certificate::CertType};
use uselesskey_core::Factory;
use uselesskey_ssh::{
    SshCertFactoryExt, SshCertSpec, SshCertType, SshFactoryExt, SshSpec, SshValidity,
};

#[test]
fn ssh_key_fixtures_exercise_private_and_authorized_key_paths() {
    let fx = Factory::deterministic_from_str("external-ssh-key-validation");

    let ed25519 = fx.ssh_key("deploy-ed25519", SshSpec::ed25519());
    assert!(ed25519.authorized_key_line().starts_with("ssh-ed25519 "));
    assert_private_and_public_round_trip(&ed25519);

    let rsa = fx.ssh_key("deploy-rsa", SshSpec::rsa());
    assert!(rsa.authorized_key_line().starts_with("ssh-rsa "));
    assert_private_and_public_round_trip(&rsa);
}

#[test]
fn ssh_certificate_fixtures_exercise_principals_and_validity_paths() {
    let fx = Factory::deterministic_from_str("external-ssh-cert-validation");
    let spec = SshCertSpec {
        principals: vec!["deploy".to_string(), "ci".to_string()],
        validity: SshValidity::new(1_700_000_000, 1_700_000_600),
        cert_type: SshCertType::User,
        critical_options: vec![("force-command".to_string(), "/usr/local/bin/deploy".to_string())],
        extensions: vec![("permit-pty".to_string(), String::new())],
    };

    let fixture = fx.ssh_cert("deploy-user-cert", spec.clone());
    let parsed =
        Certificate::from_openssh(fixture.certificate_openssh()).expect("SSH cert parses");

    assert_eq!(parsed.cert_type(), CertType::User);
    assert_eq!(parsed.valid_principals(), spec.principals.as_slice());
    assert_eq!(parsed.valid_after(), spec.validity.valid_after);
    assert_eq!(parsed.valid_before(), spec.validity.valid_before);
    assert_eq!(
        parsed
            .critical_options()
            .get("force-command")
            .map(String::as_str),
        Some("/usr/local/bin/deploy")
    );
    assert_eq!(
        parsed.extensions().get("permit-pty").map(String::as_str),
        Some("")
    );
}

#[test]
fn ssh_fixture_negative_inputs_are_stable_without_committed_payloads() {
    let fx = Factory::deterministic_from_str("external-ssh-negative-paths");
    let key = fx.ssh_key("deploy", SshSpec::ed25519());

    let tampered_authorized_key = key
        .authorized_key_line()
        .replacen("ssh-ed25519", "ssh-rsa", 1);
    assert!(PublicKey::from_openssh(&tampered_authorized_key).is_err());

    let validity = SshValidity::new(1_700_000_000, 1_700_000_600);
    let user_cert = fx.ssh_cert("deploy-user", SshCertSpec::user(["deploy"], validity));
    let host_cert = fx.ssh_cert(
        "deploy-host",
        SshCertSpec::host(["deploy.internal"], validity),
    );

    let parsed_user =
        Certificate::from_openssh(user_cert.certificate_openssh()).expect("user cert parses");
    let parsed_host =
        Certificate::from_openssh(host_cert.certificate_openssh()).expect("host cert parses");

    assert_eq!(parsed_user.cert_type(), CertType::User);
    assert_eq!(parsed_host.cert_type(), CertType::Host);
    assert_ne!(parsed_user.valid_principals(), parsed_host.valid_principals());

    let debug = format!("{key:?}");
    assert!(!debug.contains("BEGIN OPENSSH PRIVATE KEY"));
    assert!(!debug.contains("ssh-ed25519 "));
}

fn assert_private_and_public_round_trip(key: &uselesskey_ssh::SshKeyPair) {
    let private =
        PrivateKey::from_openssh(key.private_key_openssh()).expect("OpenSSH private key parses");
    let public =
        PublicKey::from_openssh(key.authorized_key_line()).expect("authorized_keys line parses");

    assert_eq!(
        private
            .public_key()
            .to_openssh()
            .expect("private public key encodes"),
        public.to_openssh().expect("authorized key encodes")
    );
}
