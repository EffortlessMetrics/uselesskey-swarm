use reqwest::StatusCode;
use reqwest::header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH};
use serde_json::Value;
use uselesskey_core::Factory;
use uselesskey_rsa::RsaSpec;
use uselesskey_test_server::{
    CachePolicySpec, IssuerUrlMode, JwksPhase, JwksRotation, JwksSpec, OidcServerSpec,
    OidcTestServer,
};

#[tokio::test]
async fn oidc_test_server_serves_discovery_and_jwks() {
    let server = OidcTestServer::start(
        Factory::deterministic_from_str("external-oidc-test-server"),
        OidcServerSpec::localhost_static(JwksSpec::single_rsa("issuer", RsaSpec::rs256())),
    )
    .await
    .expect("server starts");

    let client = http_client();
    let discovery: Value = client
        .get(server.discovery_url())
        .send()
        .await
        .expect("discovery response")
        .json()
        .await
        .expect("discovery json");
    let jwks_url = server.jwks_url();
    assert_eq!(discovery["issuer"].as_str(), Some(server.base_url()));
    assert_eq!(discovery["jwks_uri"].as_str(), Some(jwks_url.as_str()));

    let jwks: Value = client
        .get(jwks_url)
        .send()
        .await
        .expect("jwks response")
        .json()
        .await
        .expect("jwks json");
    let keys = jwks["keys"].as_array().expect("keys array");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0]["kty"].as_str(), Some("RSA"));
    assert_eq!(keys[0]["alg"].as_str(), Some("RS256"));

    server.shutdown().await;
}

#[tokio::test]
async fn oidc_test_server_switches_deterministic_jwks_phases() {
    let server = OidcTestServer::start(
        Factory::deterministic_from_str("external-oidc-rotation"),
        rotated_spec(),
    )
    .await
    .expect("server starts");

    let client = http_client();
    let primary = jwks_kid(&client, server.jwks_url()).await;
    assert_eq!(server.active_phase_name(), "primary");

    server.with_phase("rotated").expect("switch phase");
    let rotated = jwks_kid(&client, server.jwks_url()).await;

    assert_eq!(server.active_phase_name(), "rotated");
    assert_ne!(primary, rotated);
    assert!(server.with_phase("missing").is_err());

    server.shutdown().await;
}

#[tokio::test]
async fn oidc_test_server_exercises_cache_headers_and_route_flags() {
    let mut spec = rotated_spec();
    spec.cache_headers = Some(CachePolicySpec {
        max_age_seconds: 30,
        emit_etag: true,
    });

    let server =
        OidcTestServer::start(Factory::deterministic_from_str("external-oidc-cache"), spec)
            .await
            .expect("server starts");
    let client = http_client();

    let first = client
        .get(server.jwks_url())
        .send()
        .await
        .expect("jwks response");
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(
        first.headers().get(CACHE_CONTROL).and_then(|h| h.to_str().ok()),
        Some("public, max-age=30")
    );
    let etag = first
        .headers()
        .get(ETAG)
        .and_then(|h| h.to_str().ok())
        .expect("etag header")
        .to_owned();

    let not_modified = client
        .get(server.jwks_url())
        .header(IF_NONE_MATCH, etag)
        .send()
        .await
        .expect("cached jwks response");
    assert_eq!(not_modified.status(), StatusCode::NOT_MODIFIED);

    server.shutdown().await;

    let disabled = OidcTestServer::start(
        Factory::deterministic_from_str("external-oidc-disabled"),
        OidcServerSpec {
            issuer_url_mode: IssuerUrlMode::RandomPortLocalhost,
            jwks_rotation: JwksRotation::Static(JwksSpec::single_rsa(
                "disabled",
                RsaSpec::rs256(),
            )),
            cache_headers: None,
            serve_discovery: false,
            serve_jwks: true,
        },
    )
    .await
    .expect("disabled discovery server starts");

    let response = client
        .get(disabled.discovery_url())
        .send()
        .await
        .expect("disabled discovery response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    disabled.shutdown().await;
}

fn rotated_spec() -> OidcServerSpec {
    OidcServerSpec {
        issuer_url_mode: IssuerUrlMode::RandomPortLocalhost,
        jwks_rotation: JwksRotation::Sequence(vec![
            JwksPhase::new("primary", JwksSpec::single_rsa("issuer", RsaSpec::rs256())),
            JwksPhase::new(
                "rotated",
                JwksSpec::single_rsa("issuer-rotated", RsaSpec::rs256()),
            ),
        ]),
        cache_headers: None,
        serve_discovery: true,
        serve_jwks: true,
    }
}

async fn jwks_kid(client: &reqwest::Client, url: String) -> String {
    let jwks: Value = client
        .get(url)
        .send()
        .await
        .expect("jwks response")
        .json()
        .await
        .expect("jwks json");

    jwks["keys"][0]["kid"]
        .as_str()
        .expect("kid string")
        .to_owned()
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("http client")
}
