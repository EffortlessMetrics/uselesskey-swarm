use std::fs;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;
use uselesskey_test_support::{TestResult, ensure, ensure_eq, require_ok, require_some};

#[test]
fn webhook_bundle_tampered_request_preserves_the_valid_signed_pair() -> TestResult<()> {
    let temp = require_ok(tempdir(), "tempdir")?;
    let bundle_dir = temp.path().join("webhook");
    let bundle_path = require_some(bundle_dir.to_str(), "utf-8 bundle path")?;

    let mut cmd = require_ok(Command::cargo_bin("uselesskey"), "uselesskey bin")?;
    cmd.args([
        "bundle",
        "--profile",
        "webhook",
        "--seed",
        "webhook-cli-tamper-contract",
        "--label",
        "payment",
        "--out",
        bundle_path,
    ]);
    cmd.assert().success();

    let valid = read_json(&bundle_dir.join("requests/valid.json"))?;
    let tampered = read_json(&bundle_dir.join("requests/negative-tampered-body.json"))?;

    ensure_eq!(valid["expected_result"], "accept");
    ensure_eq!(tampered["expected_result"], "reject");
    ensure_eq!(tampered["rejection_class"], "tampered_body");
    ensure_eq!(valid["verifier_secret"], tampered["verifier_secret"]);
    ensure_eq!(valid["timestamp"], tampered["timestamp"]);
    ensure_eq!(valid["headers"], tampered["headers"]);
    ensure_eq!(valid["signature_profile"], tampered["signature_profile"]);

    let valid_body = require_some(valid["body"].as_str(), "valid body")?;
    let tampered_body = require_some(tampered["body"].as_str(), "tampered body")?;
    ensure_eq!(tampered_body, format!("{valid_body}\n"));

    let signature = require_some(
        valid["headers"]["Stripe-Signature"].as_str(),
        "valid Stripe-Signature",
    )?;
    ensure(
        signature.contains("t=") && signature.contains(",v1="),
        "valid request must carry the signed Stripe header",
    )?;
    ensure_eq!(
        tampered["headers"]["Stripe-Signature"].as_str(),
        Some(signature)
    );

    Ok(())
}

fn read_json(path: &std::path::Path) -> TestResult<Value> {
    let bytes = require_ok(fs::read(path), format!("read {}", path.display()))?;
    require_ok(
        serde_json::from_slice(&bytes),
        format!("parse {}", path.display()),
    )
}
