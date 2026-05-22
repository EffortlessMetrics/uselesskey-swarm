use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::{git_head_sha, target_output};

#[derive(Debug, Deserialize)]
struct ClaimLedger {
    #[serde(default)]
    claim: Vec<ClaimEntry>,
    #[serde(default)]
    claim_proof: Vec<ClaimProofPolicy>,
}

#[derive(Debug, Deserialize)]
struct ClaimEntry {
    id: String,
    title: String,
    status: String,
    boundary: String,
    #[serde(default)]
    artifacts: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClaimProofPolicy {
    claim: String,
    status: String,
    #[serde(default)]
    include_in_all_stable: bool,
    #[serde(default)]
    requires_explicit_version: bool,
    #[serde(default)]
    handlers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ClaimProofReceipt {
    schema_version: u32,
    claim: String,
    title: String,
    claim_status: String,
    status: String,
    generated_at: String,
    git_sha: Option<String>,
    boundary: String,
    handlers: Vec<HandlerReceipt>,
    artifacts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HandlerReceipt {
    handler: String,
    command: Vec<String>,
    status: String,
    artifacts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HandlerSpec {
    id: &'static str,
    argv: Vec<&'static str>,
    artifacts: Vec<&'static str>,
}

const CLAIM_PROOF_POLICY_IMPLEMENTED: &str = "implemented";
const CLAIM_PROOF_POLICY_PLANNED: &str = "planned";
const LOCK_DIR: &str = "target/claim-proof.lock";

pub(crate) fn run(root: &Path, claim: Option<&str>, all_stable: bool) -> Result<()> {
    if claim.is_some() == all_stable {
        bail!("claim-proof: pass exactly one of --claim <id> or --all-stable");
    }

    let _output_lock = acquire_output_lock(root)?;
    let ledger = read_ledger(root)?;
    let selected = match (all_stable, claim) {
        (true, _) => stable_claims_with_policy(&ledger)?,
        (false, Some(claim)) => vec![claim.to_string()],
        (false, None) => bail!("claim-proof: pass exactly one of --claim <id> or --all-stable"),
    };

    let mut failures = Vec::new();
    for claim_id in selected {
        match run_claim(root, &ledger, &claim_id) {
            Ok(receipt) => {
                println!("claim-proof: {} {}", receipt.claim, receipt.status);
            }
            Err(err) => {
                failures.push(format!("{claim_id}: {err:#}"));
            }
        }
    }

    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("claim-proof: {failure}");
        }
        bail!("claim-proof: {} claim(s) failed", failures.len());
    }

    Ok(())
}

fn acquire_output_lock(root: &Path) -> Result<target_output::TargetOutputLock> {
    target_output::acquire_lock(root, LOCK_DIR, "claim-proof")
}

pub(crate) fn stable_claim_ids(root: &Path) -> Result<Vec<String>> {
    let ledger = read_ledger(root)?;
    stable_claims_with_policy(&ledger)
}

pub(crate) fn check_policy(root: &Path) -> Result<()> {
    let ledger = read_ledger(root)?;
    println!(
        "claim-proof-policy: {} policies ok",
        ledger.claim_proof.len()
    );
    Ok(())
}

fn run_claim(root: &Path, ledger: &ClaimLedger, claim_id: &str) -> Result<ClaimProofReceipt> {
    validate_claim_id(claim_id)?;
    let claim = ledger
        .claim
        .iter()
        .find(|claim| claim.id == claim_id)
        .with_context(|| format!("unknown claim `{claim_id}`"))?;
    let policy = policy_for_claim(ledger, claim_id)?;
    if policy.requires_explicit_version {
        bail!("claim `{claim_id}` requires an explicit version and is not supported yet");
    }
    require_implemented_policy(policy, "claim")?;
    if policy.handlers.is_empty() {
        bail!("claim `{claim_id}` has no claim-proof handlers");
    }

    let mut receipt = ClaimProofReceipt {
        schema_version: 1,
        claim: claim.id.clone(),
        title: claim.title.clone(),
        claim_status: claim.status.clone(),
        status: "running".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        git_sha: git_head_sha().ok(),
        boundary: claim.boundary.clone(),
        handlers: Vec::new(),
        artifacts: claim.artifacts.clone(),
    };

    for handler in &policy.handlers {
        match run_handler(root, handler) {
            Ok(handler_receipt) => receipt.handlers.push(handler_receipt),
            Err(err) => {
                receipt.handlers.push(HandlerReceipt {
                    handler: handler.clone(),
                    command: handler_spec(handler)
                        .map(command_strings)
                        .unwrap_or_default(),
                    status: "failed".to_string(),
                    artifacts: handler_spec(handler)
                        .map(|spec| {
                            spec.artifacts
                                .into_iter()
                                .map(str::to_string)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                });
                receipt.status = "failed".to_string();
                write_receipt(root, &receipt)?;
                return Err(err).with_context(|| format!("handler `{handler}` failed"));
            }
        }
    }

    receipt.status = "pass".to_string();
    write_receipt(root, &receipt)?;
    Ok(receipt)
}

fn run_handler(root: &Path, handler: &str) -> Result<HandlerReceipt> {
    let spec = handler_spec(handler)?;
    let Some((program, args)) = spec.argv.split_first() else {
        bail!("handler `{handler}` has no command");
    };

    let mut command = Command::new(program);
    command.args(args).current_dir(root);
    let status = command
        .status()
        .with_context(|| format!("run handler `{handler}`"))?;
    if !status.success() {
        bail!("handler `{handler}` exited with {status}");
    }

    Ok(HandlerReceipt {
        handler: spec.id.to_string(),
        command: command_strings(spec.clone()),
        status: "ok".to_string(),
        artifacts: spec
            .artifacts
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>(),
    })
}

fn read_ledger(root: &Path) -> Result<ClaimLedger> {
    let path = root.join("policy/claim-ledger.toml");
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let ledger = toml::from_str(&text).context("parse policy/claim-ledger.toml")?;
    validate_claim_proof_policies(&ledger)?;
    Ok(ledger)
}

fn validate_claim_proof_policies(ledger: &ClaimLedger) -> Result<()> {
    let claims = ledger
        .claim
        .iter()
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();

    for policy in &ledger.claim_proof {
        validate_claim_id(&policy.claim)
            .with_context(|| format!("claim-proof policy `{}`", policy.claim))?;
        if !seen.insert(policy.claim.as_str()) {
            bail!("duplicate claim-proof policy for `{}`", policy.claim);
        }
        if !claims.contains(policy.claim.as_str()) {
            bail!(
                "claim-proof policy `{}` points to an unknown claim",
                policy.claim
            );
        }
        match policy.status.as_str() {
            CLAIM_PROOF_POLICY_IMPLEMENTED | CLAIM_PROOF_POLICY_PLANNED => {}
            other => bail!(
                "claim-proof policy `{}` has unknown status `{other}`",
                policy.claim
            ),
        }
        if policy.include_in_all_stable && policy.status != CLAIM_PROOF_POLICY_IMPLEMENTED {
            bail!(
                "claim-proof policy `{}` sets include_in_all_stable with status `{}`, expected `{}`",
                policy.claim,
                policy.status,
                CLAIM_PROOF_POLICY_IMPLEMENTED
            );
        }
        if policy.status == CLAIM_PROOF_POLICY_IMPLEMENTED && policy.requires_explicit_version {
            bail!(
                "implemented claim-proof policy `{}` requires an explicit version",
                policy.claim
            );
        }
        if policy.status == CLAIM_PROOF_POLICY_IMPLEMENTED && policy.handlers.is_empty() {
            bail!(
                "implemented claim-proof policy `{}` has no handlers",
                policy.claim
            );
        }
        let mut handlers_seen = BTreeSet::new();
        for handler in &policy.handlers {
            if !handlers_seen.insert(handler.as_str()) {
                bail!(
                    "claim-proof policy `{}` repeats handler `{handler}`",
                    policy.claim
                );
            }
            validate_handler_id(handler)
                .with_context(|| format!("claim-proof policy `{}`", policy.claim))?;
        }
    }

    let policies = ledger
        .claim_proof
        .iter()
        .map(|policy| (policy.claim.as_str(), policy))
        .collect::<BTreeMap<_, _>>();
    for claim in ledger.claim.iter().filter(|claim| claim.status == "stable") {
        let Some(policy) = policies.get(claim.id.as_str()) else {
            bail!("stable claim `{}` has no claim-proof policy", claim.id);
        };
        if policy.status != CLAIM_PROOF_POLICY_IMPLEMENTED {
            bail!(
                "stable claim `{}` has claim-proof policy status `{}`, expected `{}`",
                claim.id,
                policy.status,
                CLAIM_PROOF_POLICY_IMPLEMENTED
            );
        }
        if !policy.include_in_all_stable {
            bail!(
                "stable claim `{}` has claim-proof policy excluded from --all-stable",
                claim.id
            );
        }
    }

    Ok(())
}

fn stable_claims_with_policy(ledger: &ClaimLedger) -> Result<Vec<String>> {
    let policies = ledger
        .claim_proof
        .iter()
        .map(|policy| (policy.claim.as_str(), policy))
        .collect::<BTreeMap<_, _>>();
    let mut selected = Vec::new();

    for claim in ledger.claim.iter().filter(|claim| claim.status == "stable") {
        let Some(policy) = policies.get(claim.id.as_str()) else {
            bail!("stable claim `{}` has no claim-proof policy", claim.id);
        };
        if policy.include_in_all_stable {
            require_implemented_policy(policy, "stable claim")?;
            selected.push(claim.id.clone());
        }
    }

    if selected.is_empty() {
        bail!("claim-proof: no stable claims selected");
    }

    Ok(selected)
}

fn policy_for_claim<'a>(ledger: &'a ClaimLedger, claim_id: &str) -> Result<&'a ClaimProofPolicy> {
    ledger
        .claim_proof
        .iter()
        .find(|policy| policy.claim == claim_id)
        .with_context(|| format!("claim `{claim_id}` has no claim-proof policy"))
}

fn require_implemented_policy(policy: &ClaimProofPolicy, label: &str) -> Result<()> {
    if policy.status != CLAIM_PROOF_POLICY_IMPLEMENTED {
        bail!(
            "{label} `{}` has claim-proof policy status `{}`, expected `{}`",
            policy.claim,
            policy.status,
            CLAIM_PROOF_POLICY_IMPLEMENTED
        );
    }
    Ok(())
}

fn validate_handler_id(handler: &str) -> Result<()> {
    if handler == "cratesio_smoke_version" {
        return Ok(());
    }
    handler_spec(handler).map(|_| ())
}

fn handler_spec(handler: &str) -> Result<HandlerSpec> {
    let spec = match handler {
        "scanner_safe_reference_check" => HandlerSpec {
            id: "scanner_safe_reference_check",
            argv: vec!["cargo", "xtask", "scanner-safe-reference", "--check"],
            artifacts: vec![],
        },
        "no_blob" => HandlerSpec {
            id: "no_blob",
            argv: vec!["cargo", "xtask", "no-blob"],
            artifacts: vec![],
        },
        "badges_check" => HandlerSpec {
            id: "badges_check",
            argv: vec!["cargo", "xtask", "badges", "--check"],
            artifacts: vec!["badges/ripr-plus.json", "badges/scanner-safe.json"],
        },
        "test_efficiency_report" => HandlerSpec {
            id: "test_efficiency_report",
            argv: vec!["cargo", "xtask", "test-efficiency-report"],
            artifacts: vec![
                "target/ripr/reports/test-efficiency.json",
                "target/ripr/reports/test-efficiency.md",
            ],
        },
        "bundle_proof_tls" => HandlerSpec {
            id: "bundle_proof_tls",
            argv: vec![
                "cargo",
                "xtask",
                "bundle-proof",
                "--profile",
                "tls",
                "--out",
                "target/release-evidence/tls",
            ],
            artifacts: vec![
                "target/release-evidence/tls/tls-contract-pack-proof.json",
                "target/release-evidence/tls/tls-contract-pack-proof.md",
            ],
        },
        "bundle_proof_oidc" => HandlerSpec {
            id: "bundle_proof_oidc",
            argv: vec![
                "cargo",
                "xtask",
                "bundle-proof",
                "--profile",
                "oidc",
                "--out",
                "target/release-evidence/oidc",
            ],
            artifacts: vec![
                "target/release-evidence/oidc/oidc-contract-pack-proof.json",
                "target/release-evidence/oidc/oidc-contract-pack-proof.md",
            ],
        },
        "bundle_proof_webhook" => HandlerSpec {
            id: "bundle_proof_webhook",
            argv: vec![
                "cargo",
                "xtask",
                "bundle-proof",
                "--profile",
                "webhook",
                "--out",
                "target/release-evidence/webhook",
            ],
            artifacts: vec![
                "target/release-evidence/webhook/webhook-contract-pack-proof.json",
                "target/release-evidence/webhook/webhook-contract-pack-proof.md",
            ],
        },
        "public_surface" => HandlerSpec {
            id: "public_surface",
            argv: vec!["cargo", "xtask", "public-surface"],
            artifacts: vec![],
        },
        "publish_check" => HandlerSpec {
            id: "publish_check",
            argv: vec!["cargo", "xtask", "publish-check"],
            artifacts: vec![],
        },
        "publish_preflight" => HandlerSpec {
            id: "publish_preflight",
            argv: vec!["cargo", "xtask", "publish-preflight"],
            artifacts: vec!["target/xtask/receipt.json"],
        },
        "cratesio_smoke_version" => {
            bail!("handler `cratesio_smoke_version` requires an explicit version")
        }
        other => bail!("unknown claim-proof handler `{other}`"),
    };

    Ok(spec)
}

fn command_strings(spec: HandlerSpec) -> Vec<String> {
    spec.argv.into_iter().map(str::to_string).collect()
}

fn write_receipt(root: &Path, receipt: &ClaimProofReceipt) -> Result<()> {
    let out_dir = claim_out_dir(root, &receipt.claim);
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    write_json_pretty(&out_dir.join("receipt.json"), receipt)?;
    fs::write(out_dir.join("receipt.md"), render_receipt_markdown(receipt))
        .with_context(|| format!("write {}", out_dir.join("receipt.md").display()))?;
    Ok(())
}

fn claim_out_dir(root: &Path, claim: &str) -> PathBuf {
    root.join("target/claim-proof").join(claim)
}

fn render_receipt_markdown(receipt: &ClaimProofReceipt) -> String {
    let mut md = String::new();
    md.push_str("# Claim Proof Receipt\n\n");
    md.push_str(&format!("- Claim: `{}`\n", receipt.claim));
    md.push_str(&format!("- Title: {}\n", receipt.title));
    md.push_str(&format!("- Claim status: `{}`\n", receipt.claim_status));
    md.push_str(&format!("- Status: `{}`\n", receipt.status));
    md.push_str(&format!("- Generated at: `{}`\n", receipt.generated_at));
    if let Some(git_sha) = &receipt.git_sha {
        md.push_str(&format!("- Git SHA: `{git_sha}`\n"));
    }

    md.push_str("\n## Handlers\n\n");
    md.push_str("| Handler | Status | Command |\n");
    md.push_str("| --- | --- | --- |\n");
    for handler in &receipt.handlers {
        md.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            handler.handler,
            handler.status,
            handler.command.join(" ")
        ));
    }

    md.push_str("\n## Boundary\n\n");
    md.push_str(&receipt.boundary);
    md.push('\n');

    md
}

fn write_json_pretty(path: &Path, value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json + "\n").with_context(|| format!("write {}", path.display()))
}

fn validate_claim_id(claim_id: &str) -> Result<()> {
    if claim_id.is_empty()
        || !claim_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("invalid claim id `{claim_id}`");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_claim_selection_uses_include_in_all_stable() -> Result<()> {
        let ledger = minimal_ledger();
        let selected = stable_claims_with_policy(&ledger)?;

        assert_eq!(
            selected,
            vec![
                "scanner-safe-fixtures".to_string(),
                "tls-contract-pack".to_string()
            ]
        );
        Ok(())
    }

    #[test]
    fn stable_claim_selection_rejects_missing_policy() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim.push(ClaimEntry {
            id: "generated-badge-endpoints".to_string(),
            title: "Generated badges".to_string(),
            status: "stable".to_string(),
            boundary: "Boundary.".to_string(),
            artifacts: Vec::new(),
        });

        let err = match stable_claims_with_policy(&ledger) {
            Ok(selected) => bail!("unexpected stable claim selection: {selected:?}"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("stable claim `generated-badge-endpoints` has no claim-proof policy"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn stable_claim_selection_rejects_planned_policy() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].status = "planned".to_string();

        let err = match stable_claims_with_policy(&ledger) {
            Ok(selected) => bail!("unexpected stable claim selection: {selected:?}"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains(
                "stable claim `scanner-safe-fixtures` has claim-proof policy status `planned`, expected `implemented`"
            ),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn run_claim_rejects_planned_policy_before_handlers() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].status = "planned".to_string();

        let err = match run_claim(Path::new("."), &ledger, "scanner-safe-fixtures") {
            Ok(receipt) => bail!("unexpected claim proof receipt: {receipt:?}"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains(
                "claim `scanner-safe-fixtures` has claim-proof policy status `planned`, expected `implemented`"
            ),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_accepts_minimal_ledger() -> Result<()> {
        validate_claim_proof_policies(&minimal_ledger())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_duplicate_policy() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof.push(ClaimProofPolicy {
            claim: "scanner-safe-fixtures".to_string(),
            status: "implemented".to_string(),
            include_in_all_stable: false,
            requires_explicit_version: false,
            handlers: vec!["badges_check".to_string()],
        });

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("duplicate claim-proof policy for `scanner-safe-fixtures`"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_unknown_status() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].status = "ready".to_string();

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("claim-proof policy `scanner-safe-fixtures` has unknown status `ready`"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_unknown_claim() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].claim = "future-contract-pack".to_string();

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("claim-proof policy `future-contract-pack` points to an unknown claim"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_stable_claim_without_policy() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim.push(ClaimEntry {
            id: "generated-badge-endpoints".to_string(),
            title: "Generated badges".to_string(),
            status: "stable".to_string(),
            boundary: "Boundary.".to_string(),
            artifacts: Vec::new(),
        });

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("stable claim `generated-badge-endpoints` has no claim-proof policy"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_stable_claim_planned_policy() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].status = "planned".to_string();
        ledger.claim_proof[0].include_in_all_stable = false;

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains(
                "stable claim `scanner-safe-fixtures` has claim-proof policy status `planned`, expected `implemented`"
            ),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_stable_claim_excluded_from_all_stable() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].include_in_all_stable = false;

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains(
                "stable claim `scanner-safe-fixtures` has claim-proof policy excluded from --all-stable"
            ),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_planned_all_stable() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].status = "planned".to_string();

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains(
                "claim-proof policy `scanner-safe-fixtures` sets include_in_all_stable with status `planned`, expected `implemented`"
            ),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_implemented_explicit_version_policy() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].requires_explicit_version = true;

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains(
                "implemented claim-proof policy `scanner-safe-fixtures` requires an explicit version"
            ),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_implemented_policy_without_handlers() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].handlers.clear();

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("implemented claim-proof policy `scanner-safe-fixtures` has no handlers"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_duplicate_handler_id() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].handlers = vec!["no_blob".to_string(), "no_blob".to_string()];

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("claim-proof policy `scanner-safe-fixtures` repeats handler `no_blob`"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_policy_validation_rejects_unknown_handler_id() -> Result<()> {
        let mut ledger = minimal_ledger();
        ledger.claim_proof[0].handlers = vec!["cargo xtask no-blob".to_string()];

        let err = match validate_claim_proof_policies(&ledger) {
            Ok(()) => bail!("unexpected valid claim-proof policy ledger"),
            Err(err) => err,
        };

        let error_chain = format!("{err:#}");
        assert!(
            error_chain.contains(
                "claim-proof policy `scanner-safe-fixtures`: unknown claim-proof handler `cargo xtask no-blob`"
            ),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn handler_specs_construct_argv_without_shell() -> Result<()> {
        let spec = handler_spec("scanner_safe_reference_check")?;

        assert_eq!(
            spec.argv,
            vec!["cargo", "xtask", "scanner-safe-reference", "--check"]
        );
        assert!(!spec.argv.iter().any(|part| part.contains("&&")));
        Ok(())
    }

    #[test]
    fn webhook_handler_runs_bundle_proof_without_shell() -> Result<()> {
        let spec = handler_spec("bundle_proof_webhook")?;

        assert_eq!(
            spec.argv,
            vec![
                "cargo",
                "xtask",
                "bundle-proof",
                "--profile",
                "webhook",
                "--out",
                "target/release-evidence/webhook",
            ]
        );
        assert!(
            spec.artifacts
                .contains(&"target/release-evidence/webhook/webhook-contract-pack-proof.json")
        );
        assert!(!spec.argv.iter().any(|part| part.contains("&&")));
        Ok(())
    }

    #[test]
    fn unknown_handler_is_rejected() -> Result<()> {
        let err = match handler_spec("cargo xtask no-blob") {
            Ok(spec) => bail!("unexpected handler spec: {spec:?}"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("unknown claim-proof handler `cargo xtask no-blob`"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn explicit_version_handler_is_not_implicit() -> Result<()> {
        let err = match handler_spec("cratesio_smoke_version") {
            Ok(spec) => bail!("unexpected handler spec: {spec:?}"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains("requires an explicit version"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn claim_proof_output_lock_is_target_local() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let _lock = acquire_output_lock(dir.path())?;

        assert!(dir.path().join(LOCK_DIR).is_dir());
        Ok(())
    }

    #[test]
    fn receipt_markdown_includes_boundary() -> Result<()> {
        let receipt = ClaimProofReceipt {
            schema_version: 1,
            claim: "scanner-safe-fixtures".to_string(),
            title: "Scanner-safe fixtures".to_string(),
            claim_status: "stable".to_string(),
            status: "pass".to_string(),
            generated_at: "2026-05-13T00:00:00Z".to_string(),
            git_sha: Some("abc123".to_string()),
            boundary: "Not production key management.".to_string(),
            handlers: vec![HandlerReceipt {
                handler: "no_blob".to_string(),
                command: vec![
                    "cargo".to_string(),
                    "xtask".to_string(),
                    "no-blob".to_string(),
                ],
                status: "ok".to_string(),
                artifacts: Vec::new(),
            }],
            artifacts: Vec::new(),
        };

        let markdown = render_receipt_markdown(&receipt);

        assert!(markdown.contains("scanner-safe-fixtures"));
        assert!(markdown.contains("Not production key management."));
        Ok(())
    }

    #[test]
    fn invalid_claim_ids_are_rejected() -> Result<()> {
        for claim_id in ["", "../bad", "BadClaim", "bad_claim"] {
            assert!(
                validate_claim_id(claim_id).is_err(),
                "{claim_id} should be invalid"
            );
        }
        Ok(())
    }

    fn minimal_ledger() -> ClaimLedger {
        ClaimLedger {
            claim: vec![
                ClaimEntry {
                    id: "scanner-safe-fixtures".to_string(),
                    title: "Scanner-safe fixtures".to_string(),
                    status: "stable".to_string(),
                    boundary: "Boundary.".to_string(),
                    artifacts: Vec::new(),
                },
                ClaimEntry {
                    id: "tls-contract-pack".to_string(),
                    title: "TLS contract pack".to_string(),
                    status: "stable".to_string(),
                    boundary: "Boundary.".to_string(),
                    artifacts: Vec::new(),
                },
                ClaimEntry {
                    id: "external-cratesio-install-smoke".to_string(),
                    title: "Crates.io smoke".to_string(),
                    status: "release-proof".to_string(),
                    boundary: "Boundary.".to_string(),
                    artifacts: Vec::new(),
                },
            ],
            claim_proof: vec![
                ClaimProofPolicy {
                    claim: "scanner-safe-fixtures".to_string(),
                    status: "implemented".to_string(),
                    include_in_all_stable: true,
                    requires_explicit_version: false,
                    handlers: vec!["no_blob".to_string()],
                },
                ClaimProofPolicy {
                    claim: "tls-contract-pack".to_string(),
                    status: "implemented".to_string(),
                    include_in_all_stable: true,
                    requires_explicit_version: false,
                    handlers: vec!["bundle_proof_tls".to_string()],
                },
                ClaimProofPolicy {
                    claim: "external-cratesio-install-smoke".to_string(),
                    status: "planned".to_string(),
                    include_in_all_stable: false,
                    requires_explicit_version: true,
                    handlers: vec!["cratesio_smoke_version".to_string()],
                },
            ],
        }
    }
}
