use std::collections::BTreeMap;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use uselesskey::{Factory, WebhookFactoryExt, WebhookPayloadSpec, WebhookProfile};

type TestResult<T = ()> = Result<T, String>;
type HmacSha256 = Hmac<Sha256>;

fn ensure(condition: bool, context: &str) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(context.to_string())
    }
}

fn verify_hmac(secret: &str, input: &str, digest_hex: &str) -> bool {
    let Ok(mut mac) = <HmacSha256 as KeyInit>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    let Ok(tag) = hex::decode(digest_hex) else {
        return false;
    };
    mac.update(input.as_bytes());
    mac.verify_slice(&tag).is_ok()
}

fn verify_github(secret: &str, body: &str, headers: &BTreeMap<String, String>) -> bool {
    let Some(digest) = headers
        .get("X-Hub-Signature-256")
        .and_then(|value| value.strip_prefix("sha256="))
    else {
        return false;
    };
    verify_hmac(secret, body, digest)
}

fn verify_stripe(
    secret: &str,
    body: &str,
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
    verify_hmac(secret, &format!("{timestamp}.{body}"), digest)
}

fn verify_slack(
    secret: &str,
    body: &str,
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
    verify_hmac(secret, &format!("v0:{timestamp}:{body}"), digest)
}

fn verify_request(
    profile: WebhookProfile,
    secret: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
    now: i64,
) -> bool {
    match profile {
        WebhookProfile::GitHub => verify_github(secret, body, headers),
        WebhookProfile::Stripe => verify_stripe(secret, body, headers, now, 300),
        WebhookProfile::Slack => verify_slack(secret, body, headers, now, 300),
    }
}

fn exercise_provider(profile: WebhookProfile) -> TestResult {
    let fx = Factory::deterministic_from_str("external-webhook-verifier");
    let fixture = fx.webhook(profile, "payment", WebhookPayloadSpec::Canonical);
    let now = fixture.timestamp;

    ensure(
        verify_request(
            profile,
            &fixture.secret,
            &fixture.payload,
            &fixture.headers,
            now,
        ),
        "valid provider fixture must verify",
    )?;
    ensure(
        !format!("{fixture:?}").contains(&fixture.secret),
        "fixture Debug must redact the secret",
    )?;

    let wrong_secret = fixture.near_miss_wrong_secret();
    ensure(
        !verify_request(
            profile,
            &fixture.secret,
            &wrong_secret.payload,
            &wrong_secret.headers,
            now,
        ),
        "wrong-secret request must fail against the configured verifier secret",
    )?;

    let tampered = fixture.near_miss_tampered_payload();
    ensure(
        !verify_request(
            profile,
            &fixture.secret,
            &tampered.payload,
            &tampered.headers,
            now,
        ),
        "tampered delivered body must fail signature verification",
    )?;
    ensure(
        verify_request(
            profile,
            &fixture.secret,
            &fixture.payload,
            &tampered.headers,
            now,
        ),
        "restoring only the signed body must make the preserved signature valid",
    )?;

    let stale = fixture.near_miss_stale_timestamp(300);
    match profile {
        WebhookProfile::GitHub => ensure(
            verify_request(
                profile,
                &fixture.secret,
                &stale.payload,
                &stale.headers,
                now,
            ),
            "GitHub HMAC verification has no fixture timestamp freshness policy",
        )?,
        WebhookProfile::Stripe | WebhookProfile::Slack => ensure(
            !verify_request(
                profile,
                &fixture.secret,
                &stale.payload,
                &stale.headers,
                now,
            ),
            "stale signed timestamp must fail the configured freshness window",
        )?,
    }

    let signature_header = match profile {
        WebhookProfile::GitHub => "X-Hub-Signature-256",
        WebhookProfile::Stripe => "Stripe-Signature",
        WebhookProfile::Slack => "X-Slack-Signature",
    };
    let mut missing = fixture.headers.clone();
    missing.remove(signature_header);
    ensure(
        !verify_request(profile, &fixture.secret, &fixture.payload, &missing, now),
        "missing signature header must fail",
    )?;

    let mut malformed = fixture.headers.clone();
    malformed.insert(signature_header.to_string(), "not-a-signature".to_string());
    ensure(
        !verify_request(
            profile,
            &fixture.secret,
            &fixture.payload,
            &malformed,
            now,
        ),
        "malformed signature header must fail",
    )?;

    Ok(())
}

#[test]
fn github_fixture_exercises_real_hmac_accept_and_reject_paths() -> TestResult {
    exercise_provider(WebhookProfile::GitHub)
}

#[test]
fn stripe_fixture_exercises_real_hmac_accept_and_reject_paths() -> TestResult {
    exercise_provider(WebhookProfile::Stripe)
}

#[test]
fn slack_fixture_exercises_real_hmac_accept_and_reject_paths() -> TestResult {
    exercise_provider(WebhookProfile::Slack)
}

#[test]
fn raw_payload_bytes_are_verified_without_json_reserialization() -> TestResult {
    let fx = Factory::deterministic_from_str("external-webhook-raw-body");
    let raw = "{\"event\": \"fixture\", \"spacing\": true}\n";
    let fixture = fx.webhook_stripe("raw", WebhookPayloadSpec::Raw(raw.to_string()));

    ensure(
        verify_request(
            WebhookProfile::Stripe,
            &fixture.secret,
            raw,
            &fixture.headers,
            fixture.timestamp,
        ),
        "exact delivered raw bytes must verify",
    )?;
    ensure(
        !verify_request(
            WebhookProfile::Stripe,
            &fixture.secret,
            raw.trim(),
            &fixture.headers,
            fixture.timestamp,
        ),
        "normalizing whitespace before verification must change the signed bytes",
    )?;
    Ok(())
}

#[test]
fn signed_malformed_json_is_a_parse_failure_after_hmac_acceptance() -> TestResult {
    let fx = Factory::deterministic_from_str("external-webhook-malformed-json");
    let malformed_json = "{\"event\":\"fixture\"";
    let fixture = fx.webhook_github(
        "malformed-json",
        WebhookPayloadSpec::Raw(malformed_json.to_string()),
    );

    ensure(
        verify_request(
            WebhookProfile::GitHub,
            &fixture.secret,
            &fixture.payload,
            &fixture.headers,
            fixture.timestamp,
        ),
        "the raw malformed body is still correctly signed",
    )?;
    ensure(
        serde_json::from_str::<serde_json::Value>(&fixture.payload).is_err(),
        "application JSON parsing must fail after signature acceptance",
    )?;
    Ok(())
}
