#[cfg(test)]
use std::fmt::{Debug, Display};

#[cfg(test)]
use reqwest::StatusCode;
#[cfg(test)]
use reqwest::header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH};
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use uselesskey_core::Factory;
#[cfg(test)]
use uselesskey_rsa::RsaSpec;
#[cfg(test)]
use uselesskey_test_server::{
    CachePolicySpec, IssuerUrlMode, JwksPhase, JwksRotation, JwksSpec, OidcServerSpec,
    OidcTestServer,
};

#[cfg(test)]
type TestResult<T = ()> = Result<T, String>;

#[cfg(test)]
fn ensure(condition: bool, context: &str) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(context.to_string())
    }
}

#[cfg(test)]
fn ensure_eq<T>(left: T, right: T, context: &str) -> TestResult
where
    T: Debug + PartialEq,
{
    if left == right {
        Ok(())
    } else {
        Err(format!("{context}: left={left:?}, right={right:?}"))
    }
}

#[cfg(test)]
fn require_ok<T, E>(result: Result<T, E>, context: &str) -> TestResult<T>
where
    E: Display,
{
    result.map_err(|error| format!("{context}: {error}"))
}

#[cfg(test)]
fn require_some<T>(value: Option<T>, context: &str) -> TestResult<T> {
    value.ok_or_else(|| context.to_string())
}

#[cfg(test)]
#[tokio::test]
async fn oidc_test_server_serves_discovery_and_jwks() -> TestResult<()> {
    let server = require_ok(
        OidcTestServer::start(
            Factory::deterministic_from_str("external-oidc-test-server"),
            OidcServerSpec::localhost_static(JwksSpec::single_rsa("issuer", RsaSpec::rs256())),
        )
        .await,
        "server starts",
    )?;

    let client = http_client()?;
    let discovery: Value = require_ok(
        require_ok(
            client.get(server.discovery_url()).send().await,
            "discovery response",
        )?
        .json()
        .await,
        "discovery json",
    )?;
    let base_url = server.base_url();
    let jwks_url = server.jwks_url();
    ensure_eq(
        discovery.get("issuer").and_then(Value::as_str),
        Some(base_url),
        "discovery issuer",
    )?;
    ensure_eq(
        discovery.get("jwks_uri").and_then(Value::as_str),
        Some(jwks_url.as_str()),
        "discovery jwks_uri",
    )?;

    let jwks: Value = require_ok(
        require_ok(client.get(jwks_url).send().await, "jwks response")?
            .json()
            .await,
        "jwks json",
    )?;
    let keys = require_some(jwks.get("keys").and_then(Value::as_array), "keys array")?;
    let first_key = require_some(keys.first(), "first JWKS key")?;
    ensure_eq(keys.len(), 1, "JWKS key count")?;
    ensure_eq(
        first_key.get("kty").and_then(Value::as_str),
        Some("RSA"),
        "JWKS kty",
    )?;
    ensure_eq(
        first_key.get("alg").and_then(Value::as_str),
        Some("RS256"),
        "JWKS alg",
    )?;

    server.shutdown().await;
    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn oidc_test_server_switches_deterministic_jwks_phases() -> TestResult<()> {
    let server = require_ok(
        OidcTestServer::start(
            Factory::deterministic_from_str("external-oidc-rotation"),
            rotated_spec(),
        )
        .await,
        "server starts",
    )?;

    let client = http_client()?;
    let primary = jwks_kid(&client, server.jwks_url()).await?;
    ensure_eq(
        server.active_phase_name(),
        "primary",
        "initial active phase",
    )?;

    require_ok(server.with_phase("rotated"), "switch phase")?;
    let rotated = jwks_kid(&client, server.jwks_url()).await?;

    ensure_eq(
        server.active_phase_name(),
        "rotated",
        "rotated active phase",
    )?;
    ensure(primary != rotated, "rotation must change the active kid")?;
    ensure(
        server.with_phase("missing").is_err(),
        "unknown phase must be rejected",
    )?;

    server.shutdown().await;
    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn oidc_test_server_exercises_cache_headers_and_route_flags() -> TestResult<()> {
    let mut spec = rotated_spec();
    spec.cache_headers = Some(CachePolicySpec {
        max_age_seconds: 30,
        emit_etag: true,
    });

    let server = require_ok(
        OidcTestServer::start(Factory::deterministic_from_str("external-oidc-cache"), spec).await,
        "server starts",
    )?;
    let client = http_client()?;

    let first = require_ok(client.get(server.jwks_url()).send().await, "jwks response")?;
    ensure_eq(first.status(), StatusCode::OK, "first JWKS status")?;
    ensure_eq(
        first
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|header| header.to_str().ok()),
        Some("public, max-age=30"),
        "cache-control header",
    )?;
    let etag = require_some(
        first.headers().get(ETAG).and_then(|header| header.to_str().ok()),
        "etag header",
    )?
    .to_owned();

    let not_modified = require_ok(
        client
            .get(server.jwks_url())
            .header(IF_NONE_MATCH, etag)
            .send()
            .await,
        "cached jwks response",
    )?;
    ensure_eq(
        not_modified.status(),
        StatusCode::NOT_MODIFIED,
        "conditional JWKS status",
    )?;

    server.shutdown().await;

    let disabled = require_ok(
        OidcTestServer::start(
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
        .await,
        "disabled discovery server starts",
    )?;

    let response = require_ok(
        client.get(disabled.discovery_url()).send().await,
        "disabled discovery response",
    )?;
    ensure_eq(
        response.status(),
        StatusCode::NOT_FOUND,
        "disabled discovery status",
    )?;

    disabled.shutdown().await;
    Ok(())
}

#[cfg(test)]
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

#[cfg(test)]
async fn jwks_kid(client: &reqwest::Client, url: String) -> TestResult<String> {
    let jwks: Value = require_ok(
        require_ok(client.get(url).send().await, "jwks response")?
            .json()
            .await,
        "jwks json",
    )?;

    let keys = require_some(jwks.get("keys").and_then(Value::as_array), "keys array")?;
    let first_key = require_some(keys.first(), "first JWKS key")?;
    Ok(require_some(first_key.get("kid").and_then(Value::as_str), "kid string")?.to_owned())
}

#[cfg(test)]
fn http_client() -> TestResult<reqwest::Client> {
    require_ok(reqwest::Client::builder().no_proxy().build(), "http client")
}
