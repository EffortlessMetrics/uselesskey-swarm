use std::io::{Read, Write};
use std::sync::Arc;

use rustls::crypto::CryptoProvider;
use uselesskey::{ChainSpec, Factory, X509FactoryExt};
use uselesskey_rustls::{RustlsClientConfigExt, RustlsServerConfigExt};

type TestResult<T = ()> = Result<T, String>;

const DOMAIN: &str = "valid.tls.uselesskey.test";
const MAX_HANDSHAKE_ITERATIONS: usize = 10;

fn ring_provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn ensure(condition: bool, context: &str) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(context.to_string())
    }
}

fn make_connections(
    server_chain: &uselesskey::X509Chain,
    trusted_chain: &uselesskey::X509Chain,
    domain: &str,
) -> TestResult<(rustls::ServerConnection, rustls::ClientConnection)> {
    let provider = ring_provider();
    let server_config = Arc::new(server_chain.server_config_rustls_with_provider(provider.clone()));
    let client_config = Arc::new(trusted_chain.client_config_rustls_with_provider(provider));
    let server_name: rustls::pki_types::ServerName<'_> = domain
        .try_into()
        .map_err(|error| format!("server name: {error}"))?;
    let server = rustls::ServerConnection::new(server_config)
        .map_err(|error| format!("server connection: {error}"))?;
    let client = rustls::ClientConnection::new(client_config, server_name.to_owned())
        .map_err(|error| format!("client connection: {error}"))?;
    Ok((server, client))
}

fn drive_handshake(
    server: &mut rustls::ServerConnection,
    client: &mut rustls::ClientConnection,
) -> Result<(), rustls::Error> {
    let mut buf = Vec::new();
    for _ in 0..MAX_HANDSHAKE_ITERATIONS {
        let mut progress = false;

        buf.clear();
        if client.wants_write() {
            client
                .write_tls(&mut buf)
                .map_err(|error| rustls::Error::General(format!("client write: {error}")))?;
            if !buf.is_empty() {
                server
                    .read_tls(&mut &buf[..])
                    .map_err(|error| rustls::Error::General(format!("server read: {error}")))?;
                server.process_new_packets()?;
                progress = true;
            }
        }

        buf.clear();
        if server.wants_write() {
            server
                .write_tls(&mut buf)
                .map_err(|error| rustls::Error::General(format!("server write: {error}")))?;
            if !buf.is_empty() {
                client
                    .read_tls(&mut &buf[..])
                    .map_err(|error| rustls::Error::General(format!("client read: {error}")))?;
                client.process_new_packets()?;
                progress = true;
            }
        }

        if !progress || (!client.is_handshaking() && !server.is_handshaking()) {
            break;
        }
    }
    if client.is_handshaking() || server.is_handshaking() {
        return Err(rustls::Error::General(
            "handshake did not complete within the bounded loop".to_string(),
        ));
    }
    Ok(())
}

fn read_available(reader: &mut rustls::Reader<'_>) -> TestResult<Vec<u8>> {
    let mut received = Vec::new();
    loop {
        let mut chunk = [0_u8; 4096];
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => received.extend_from_slice(&chunk[..count]),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(error) => return Err(format!("plaintext read: {error}")),
        }
    }
    Ok(received)
}

fn transfer_client_request(
    server: &mut rustls::ServerConnection,
    client: &mut rustls::ClientConnection,
    request: &[u8],
) -> TestResult<Vec<u8>> {
    client
        .writer()
        .write_all(request)
        .map_err(|error| format!("client plaintext write: {error}"))?;

    let mut tls = Vec::new();
    while client.wants_write() {
        client
            .write_tls(&mut tls)
            .map_err(|error| format!("client TLS write: {error}"))?;
    }
    server
        .read_tls(&mut &tls[..])
        .map_err(|error| format!("server TLS read: {error}"))?;
    server
        .process_new_packets()
        .map_err(|error| format!("server packets: {error}"))?;

    read_available(&mut server.reader())
}

#[test]
fn tls_chain_fixture_completes_verified_handshake_and_data_transfer() -> TestResult {
    let fx = Factory::deterministic_from_str("external-tls-chain-validation");
    let chain = fx.x509_chain("service", ChainSpec::new(DOMAIN));

    ensure(
        chain.chain_pem().contains("BEGIN CERTIFICATE"),
        "chain PEM must contain a certificate",
    )?;
    ensure(
        chain.root_cert_pem().contains("BEGIN CERTIFICATE"),
        "root PEM must contain a certificate",
    )?;
    ensure(
        chain
            .leaf_private_key_pkcs8_pem()
            .contains("BEGIN PRIVATE KEY"),
        "leaf identity must contain a PKCS#8 private key",
    )?;

    let (mut server, mut client) = make_connections(&chain, &chain, DOMAIN)?;
    drive_handshake(&mut server, &mut client)
        .map_err(|error| format!("valid TLS handshake failed: {error}"))?;
    let received = transfer_client_request(&mut server, &mut client, b"fixture request")?;
    ensure(received == b"fixture request", "server must receive client data")?;
    Ok(())
}

#[test]
fn expired_leaf_is_rejected_by_the_same_trust_configuration() -> TestResult {
    let fx = Factory::deterministic_from_str("external-tls-expired");
    let valid = fx.x509_chain("service", ChainSpec::new(DOMAIN));
    let expired = valid.expired_leaf();

    let (mut valid_server, mut valid_client) = make_connections(&valid, &valid, DOMAIN)?;
    drive_handshake(&mut valid_server, &mut valid_client)
        .map_err(|error| format!("valid control failed: {error}"))?;

    let (mut expired_server, mut client) = make_connections(&expired, &valid, DOMAIN)?;
    ensure(
        drive_handshake(&mut expired_server, &mut client).is_err(),
        "expired leaf must fail certificate validation",
    )?;
    Ok(())
}

#[test]
fn unknown_ca_is_rejected_by_the_same_hostname_and_provider() -> TestResult {
    let fx = Factory::deterministic_from_str("external-tls-unknown-ca");
    let server_chain = fx.x509_chain("server", ChainSpec::new(DOMAIN));
    let trusted_chain = fx.x509_chain("other-ca", ChainSpec::new(DOMAIN));

    let (mut valid_server, mut valid_client) =
        make_connections(&server_chain, &server_chain, DOMAIN)?;
    drive_handshake(&mut valid_server, &mut valid_client)
        .map_err(|error| format!("valid control failed: {error}"))?;

    let (mut server, mut wrong_trust_client) =
        make_connections(&server_chain, &trusted_chain, DOMAIN)?;
    ensure(
        drive_handshake(&mut server, &mut wrong_trust_client).is_err(),
        "client trusting another generated CA must reject the server chain",
    )?;
    Ok(())
}
