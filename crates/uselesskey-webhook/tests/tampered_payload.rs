use std::collections::BTreeMap;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use uselesskey_core::Factory;
use uselesskey_webhook::{WebhookFactoryExt, WebhookPayloadSpec, WebhookProfile};

type TestResult<T = ()> = Result<T, String>;
type HmacSha256 = Hmac<Sha256>;

fn ensure(condition: bool, message: &str) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

fn verify_hmac(secret: &str, input: &str, digest_hex: &str) -> bool {
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    let Ok(tag) = hex::decode(digest_hex) else {
        return false;
    };
    mac.update(input.as_bytes());
    mac.verify_slice(&tag).is_ok()
}

fn verify_github(secret: &str, payload: &str, headers: &BTreeMap<String, String>) -> bool {
    let Some(digest) = headers
        .get("X-Hub-Signature-256")
        .and_then(|value| value.strip_prefix("sha256="))
    else {
        return false;
    };
    verify_hmac(secret, payload, digest)
}

fn verify_stripe(
    secret: &str,
    payload: &str,
    headers: &BTreeMap<String, String>,
    now: i64,
    tolerance_secs: i64,
) -> bool {
    let Some(signature) = headers.get("Stripe-Signature") else {
        return false;
    };
    let mut timestamp = None;
    let mut digest = None;
    for part in signature.split(',') {
        if let Some(value) = part.strip_prefix("t=") {
            timestamp = value.parse::<i64>().ok();
        } else if let Some(value) = part.strip_prefix("v1=") {
            digest = Some(value);
        }
    }
    let (Some(timestamp), Some(digest)) = (timestamp, digest) else {
        return false;
    };
    if (now - timestamp).abs() > tolerance_secs {
        return false;
    }
    verify_hmac(secret, &format!("{timestamp}.{payload}"), digest)
}

fn verify_slack(
    secret: &str,
    payload: &str,
    headers: &BTreeMap<String, String>,
    now: i64,
    tolerance_secs: i64,
) -> bool {
    let Some(timestamp) = headers
        .get("X-Slack-Request-Timestamp")
        .and_then(|value| value.parse::<i64>().ok())
    else {
        return false;
    };
    let Some(digest) = headers
        .get("X-Slack-Signature")
        .and_then(|value| value.strip_prefix("v0="))
    else {
        return false;
    };
    if (now - timestamp).abs() > tolerance_secs {
        return false;
    }
    verify_hmac(secret, &format!("v0:{timestamp}:{payload}"), digest)
}

fn verify_fixture(
    profile: WebhookProfile,
    secret: &str,
    payload: &str,
    headers: &BTreeMap<String, String>,
    timestamp: i64,
) -> bool {
    match profile {
        WebhookProfile::GitHub => verify_github(secret, payload, headers),
        WebhookProfile::Stripe => verify_stripe(secret, payload, headers, timestamp, 300),
        WebhookProfile::Slack => verify_slack(secret, payload, headers, timestamp, 300),
    }
}

fn assert_tampered_payload_case(profile: WebhookProfile) -> TestResult {
    let fx = Factory::deterministic_from_str("webhook-tampered-public-regression");
    let valid = fx.webhook(profile, "service", WebhookPayloadSpec::Canonical);
    ensure(
        verify_fixture(
            profile,
            &valid.secret,
            &valid.payload,
            &valid.headers,
            valid.timestamp,
        ),
        "valid fixture must verify",
    )?;

    let tampered = valid.near_miss_tampered_payload();
    ensure(tampered.secret == valid.secret, "tamper must preserve secret")?;
    ensure(
        tampered.timestamp == valid.timestamp,
        "tamper must preserve timestamp",
    )?;
    ensure(
        tampered.headers == valid.headers,
        "tamper must preserve signed headers",
    )?;
    ensure(
        tampered.signature_input == valid.signature_input,
        "tamper must preserve the original signed input",
    )?;
    ensure(
        tampered.payload != valid.payload,
        "tamper must change only the delivered payload",
    )?;
    ensure(
        !verify_fixture(
            profile,
            &valid.secret,
            &tampered.payload,
            &tampered.headers,
            tampered.timestamp,
        ),
        "tampered delivered body must fail signature verification",
    )?;
    ensure(
        verify_fixture(
            profile,
            &valid.secret,
            &valid.payload,
            &tampered.headers,
            tampered.timestamp,
        ),
        "restoring only the original body must make the preserved signature valid",
    )?;

    Ok(())
}

#[test]
fn github_tampered_payload_preserves_signed_request_and_fails_on_delivered_body() -> TestResult {
    assert_tampered_payload_case(WebhookProfile::GitHub)
}

#[test]
fn stripe_tampered_payload_preserves_signed_request_and_fails_on_delivered_body() -> TestResult {
    assert_tampered_payload_case(WebhookProfile::Stripe)
}

#[test]
fn slack_tampered_payload_preserves_signed_request_and_fails_on_delivered_body() -> TestResult {
    assert_tampered_payload_case(WebhookProfile::Slack)
}
