use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{git_head_sha, target_output};

const OUT_DIR: &str = "target/external-adoption-smoke";
const LOCK_DIR: &str = "target/external-adoption-smoke.lock";
const WORK_DIR: &str = "target/external-adoption-smoke/work";
const LOG_DIR: &str = "target/external-adoption-smoke/logs";
const REPORT_JSON: &str = "target/external-adoption-smoke/report.json";
const REPORT_MD: &str = "target/external-adoption-smoke/report.md";

const CLI_PROFILES: &[&str] = &["scanner-safe", "tls", "oidc", "webhook"];
const CI_RECIPE_PROFILES: &[&str] = &["scanner-safe", "oidc", "webhook", "tls"];
const RUST_TEST_FIXTURES_EXAMPLE: ExternalExample = ExternalExample {
    name: "rust-test-fixtures",
    source_dir: "examples/external/rust-test-fixtures",
};
const WEBHOOK_VERIFIER_EXAMPLE: ExternalExample = ExternalExample {
    name: "webhook-verifier",
    source_dir: "examples/external/webhook-verifier",
};
const OIDC_JWKS_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "oidc-jwks-validation",
    source_dir: "examples/external/oidc-jwks-validation",
};
const OIDC_TEST_SERVER_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "oidc-test-server-validation",
    source_dir: "examples/external/oidc-test-server-validation",
};
const TLS_CHAIN_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "tls-chain-validation",
    source_dir: "examples/external/tls-chain-validation",
};
const WEBAUTHN_CEREMONY_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "webauthn-ceremony-validation",
    source_dir: "examples/external/webauthn-ceremony-validation",
};
const PKCS11_MOCK_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "pkcs11-mock-validation",
    source_dir: "examples/external/pkcs11-mock-validation",
};
const SSH_FIXTURE_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "ssh-fixture-validation",
    source_dir: "examples/external/ssh-fixture-validation",
};
const PGP_FIXTURE_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "pgp-fixture-validation",
    source_dir: "examples/external/pgp-fixture-validation",
};
const HMAC_SIGNATURE_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "hmac-signature-validation",
    source_dir: "examples/external/hmac-signature-validation",
};
const JSONWEBTOKEN_ADAPTER_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "jsonwebtoken-adapter-validation",
    source_dir: "examples/external/jsonwebtoken-adapter-validation",
};
const ENTROPY_BYTE_FIXTURES_EXAMPLE: ExternalExample = ExternalExample {
    name: "entropy-byte-fixtures",
    source_dir: "examples/external/entropy-byte-fixtures",
};
const ECDSA_FIXTURE_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "ecdsa-fixture-validation",
    source_dir: "examples/external/ecdsa-fixture-validation",
};
const ED25519_FIXTURE_VALIDATION_EXAMPLE: ExternalExample = ExternalExample {
    name: "ed25519-fixture-validation",
    source_dir: "examples/external/ed25519-fixture-validation",
};
const DOWNSTREAM_CI_BUNDLE_AUDIT_EXAMPLE: ExternalExample = ExternalExample {
    name: "downstream-ci-bundle-audit",
    source_dir: "examples/external/downstream-ci-bundle-audit",
};
const LIBRARY_EXAMPLES: &[ExternalExample] = &[
    RUST_TEST_FIXTURES_EXAMPLE,
    WEBHOOK_VERIFIER_EXAMPLE,
    OIDC_JWKS_VALIDATION_EXAMPLE,
    OIDC_TEST_SERVER_VALIDATION_EXAMPLE,
    TLS_CHAIN_VALIDATION_EXAMPLE,
    WEBAUTHN_CEREMONY_VALIDATION_EXAMPLE,
    PKCS11_MOCK_VALIDATION_EXAMPLE,
    SSH_FIXTURE_VALIDATION_EXAMPLE,
    PGP_FIXTURE_VALIDATION_EXAMPLE,
    HMAC_SIGNATURE_VALIDATION_EXAMPLE,
    JSONWEBTOKEN_ADAPTER_VALIDATION_EXAMPLE,
    ENTROPY_BYTE_FIXTURES_EXAMPLE,
    ECDSA_FIXTURE_VALIDATION_EXAMPLE,
    ED25519_FIXTURE_VALIDATION_EXAMPLE,
];
const CI_RECIPE_EXAMPLES: &[ExternalExample] = &[
    RUST_TEST_FIXTURES_EXAMPLE,
    WEBHOOK_VERIFIER_EXAMPLE,
    OIDC_JWKS_VALIDATION_EXAMPLE,
    TLS_CHAIN_VALIDATION_EXAMPLE,
    DOWNSTREAM_CI_BUNDLE_AUDIT_EXAMPLE,
];
const EXTERNAL_EXAMPLES: &[ExternalExample] = &[
    RUST_TEST_FIXTURES_EXAMPLE,
    WEBHOOK_VERIFIER_EXAMPLE,
    OIDC_JWKS_VALIDATION_EXAMPLE,
    OIDC_TEST_SERVER_VALIDATION_EXAMPLE,
    TLS_CHAIN_VALIDATION_EXAMPLE,
    WEBAUTHN_CEREMONY_VALIDATION_EXAMPLE,
    PKCS11_MOCK_VALIDATION_EXAMPLE,
    SSH_FIXTURE_VALIDATION_EXAMPLE,
    PGP_FIXTURE_VALIDATION_EXAMPLE,
    HMAC_SIGNATURE_VALIDATION_EXAMPLE,
    JSONWEBTOKEN_ADAPTER_VALIDATION_EXAMPLE,
    ENTROPY_BYTE_FIXTURES_EXAMPLE,
    ECDSA_FIXTURE_VALIDATION_EXAMPLE,
    ED25519_FIXTURE_VALIDATION_EXAMPLE,
    DOWNSTREAM_CI_BUNDLE_AUDIT_EXAMPLE,
];
const BOUNDARIES: &[&str] = &[
    "external-adoption-smoke proves clean-project user paths; it does not publish or approve a release",
    "published-version mode is audit/reference evidence, not a release trigger",
    "installed-style CLI smoke does not claim provider compatibility",
    "repo-local claim-proof and verification-pack remain separate proof surfaces",
    "generated fixture payloads and temp projects stay under target/external-adoption-smoke/",
    "child Cargo build caches may use CARGO_TARGET_DIR and are not fixture payloads",
];

#[derive(Clone, Copy, Debug)]
struct ExternalExample {
    name: &'static str,
    source_dir: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug)]
pub struct RunOptions {
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub ci_recipes: bool,
    pub library_examples: bool,
    pub format: OutputFormat,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
enum SmokeMode {
    Path,
    Version,
}

#[derive(Debug)]
struct SmokeSource {
    mode: SmokeMode,
    label: String,
    facade_dep: FacadeDependency,
    cli_source: CliSource,
}

#[derive(Debug)]
enum FacadeDependency {
    Path(PathBuf),
    Version(String),
}

#[derive(Debug)]
enum CliSource {
    LocalPath(PathBuf),
    Version(String),
}

#[derive(Debug, serde::Serialize)]
struct ExternalAdoptionSmokeReceipt {
    schema_version: u32,
    status: String,
    generated_at: String,
    git_sha: Option<String>,
    mode: SmokeMode,
    source: String,
    work_root: String,
    ci_recipes: bool,
    library_examples: bool,
    projects: Vec<ExternalAdoptionProject>,
    steps: Vec<ExternalAdoptionStep>,
    artifacts: Vec<String>,
    boundaries: Vec<&'static str>,
}

#[derive(Debug, serde::Serialize)]
struct ExternalAdoptionProject {
    name: String,
    path: String,
    status: String,
    outputs: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ExternalAdoptionStep {
    name: String,
    command: Vec<String>,
    cwd: String,
    status: String,
    duration_ms: u64,
    stdout: String,
    stderr: String,
    details: Option<String>,
    artifacts: Vec<String>,
}

pub fn run(root: &Path, options: RunOptions) -> Result<()> {
    validate_run_options(&options)?;
    let source = resolve_source(&options)?;
    let _output_lock = target_output::acquire_lock(root, LOCK_DIR, "external-adoption-smoke")?;
    let out_dir = root.join(OUT_DIR);
    reset_target_output(root, &out_dir)?;
    let work_dir = root.join(WORK_DIR);
    let log_dir = root.join(LOG_DIR);
    fs::create_dir_all(&work_dir)
        .with_context(|| format!("failed to create {}", work_dir.display()))?;
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create {}", log_dir.display()))?;

    let mut receipt = ExternalAdoptionSmokeReceipt {
        schema_version: 1,
        status: "running".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        git_sha: git_head_sha().ok(),
        mode: source.mode.clone(),
        source: source.label.clone(),
        work_root: relative_artifact(root, &work_dir),
        ci_recipes: options.ci_recipes,
        library_examples: options.library_examples,
        projects: Vec::new(),
        steps: Vec::new(),
        artifacts: vec![
            REPORT_JSON.to_string(),
            REPORT_MD.to_string(),
            WORK_DIR.to_string(),
            LOG_DIR.to_string(),
        ],
        boundaries: BOUNDARIES.to_vec(),
    };

    let result = run_matrix(
        root,
        &source,
        &work_dir,
        &log_dir,
        options.ci_recipes,
        options.library_examples,
        &mut receipt,
    );
    receipt.status = if result.is_ok() { "pass" } else { "failed" }.to_string();
    write_receipts(&out_dir, &receipt)?;
    match options.format {
        OutputFormat::Human => print_human(&receipt),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&receipt)?),
    }

    result
}

fn run_matrix(
    root: &Path,
    source: &SmokeSource,
    work_dir: &Path,
    log_dir: &Path,
    ci_recipes: bool,
    library_examples: bool,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<()> {
    if library_examples {
        run_external_examples(root, source, LIBRARY_EXAMPLES, work_dir, log_dir, receipt)?;
        return Ok(());
    }

    let cli_bin = prepare_cli(root, source, work_dir, log_dir, receipt)?;
    if ci_recipes {
        run_external_examples(root, source, CI_RECIPE_EXAMPLES, work_dir, log_dir, receipt)?;
        run_ci_recipes(&cli_bin, work_dir, log_dir, receipt)?;
        return Ok(());
    }

    run_external_examples(root, source, EXTERNAL_EXAMPLES, work_dir, log_dir, receipt)?;
    run_cli_discovery(&cli_bin, work_dir, log_dir, receipt)?;
    for profile in CLI_PROFILES {
        run_cli_profile(&cli_bin, profile, work_dir, log_dir, receipt)?;
    }
    Ok(())
}

fn prepare_cli(
    root: &Path,
    source: &SmokeSource,
    work_dir: &Path,
    log_dir: &Path,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<PathBuf> {
    match &source.cli_source {
        CliSource::LocalPath(cli_dir) => {
            let inherited_target_dir = std::env::var_os("CARGO_TARGET_DIR");
            let target_dir = smoke_cargo_target_dir(
                root,
                work_dir,
                "local-cli",
                inherited_target_dir.as_deref(),
            );
            fs::create_dir_all(&target_dir)
                .with_context(|| format!("failed to create {}", target_dir.display()))?;
            let target_artifact = relative_artifact_from_path(&target_dir);
            let mut cmd = Command::new("cargo");
            cmd.args(["build", "--quiet", "--bin", "uselesskey", "--manifest-path"])
                .arg(cli_dir.join("Cargo.toml"))
                .env("CARGO_TARGET_DIR", &target_dir);
            run_command_step(
                receipt,
                "build-local-cli",
                cmd,
                root,
                log_dir,
                &[target_artifact.as_str()],
            )?;
            let bin = target_dir.join("debug").join(cli_binary_name());
            if !bin.is_file() {
                bail!("expected local CLI binary at {}", bin.display());
            }
            Ok(bin)
        }
        CliSource::Version(version) => {
            let cli_root = work_dir.join("cli-root");
            let mut cmd = Command::new("cargo");
            cmd.args(["install", "uselesskey-cli", "--version", version, "--root"])
                .arg(&cli_root)
                .arg("--locked");
            run_command_step(
                receipt,
                "install-published-cli",
                cmd,
                root,
                log_dir,
                &["target/external-adoption-smoke/work/cli-root/"],
            )?;
            let bin = cli_root.join("bin").join(cli_binary_name());
            if !bin.is_file() {
                bail!("expected installed CLI binary at {}", bin.display());
            }
            Ok(bin)
        }
    }
}

fn run_external_examples(
    root: &Path,
    source: &SmokeSource,
    examples: &[ExternalExample],
    work_dir: &Path,
    log_dir: &Path,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<()> {
    for example in examples {
        run_external_example(root, source, example, work_dir, log_dir, receipt)?;
    }
    Ok(())
}

fn run_external_example(
    root: &Path,
    source: &SmokeSource,
    example: &ExternalExample,
    work_dir: &Path,
    log_dir: &Path,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<()> {
    let source_dir = root.join(example.source_dir);
    let project_dir = work_dir.join(example.name);
    copy_example_project(&source_dir, &project_dir)
        .with_context(|| format!("failed to copy {}", source_dir.display()))?;
    patch_example_dependencies(&project_dir, source)
        .with_context(|| format!("failed to patch {}", project_dir.display()))?;

    let project_artifact = relative_artifact_from_path(&project_dir);
    let inherited_target_dir = std::env::var_os("CARGO_TARGET_DIR");
    let target_dir = external_examples_target_dir(root, work_dir, inherited_target_dir.as_deref());
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;
    let target_artifact = relative_artifact_from_path(&target_dir);

    let mut cmd = Command::new("cargo");
    cmd.args(["test", "--quiet"])
        .current_dir(&project_dir)
        .env("CARGO_TARGET_DIR", &target_dir);
    run_command_step(
        receipt,
        &format!("external-example-{}", example.name),
        cmd,
        &project_dir,
        log_dir,
        &[project_artifact.as_str(), target_artifact.as_str()],
    )?;
    record_project(receipt, example.name, &project_dir, &[target_artifact]);
    Ok(())
}

fn external_examples_target_dir(
    root: &Path,
    work_dir: &Path,
    inherited_target_dir: Option<&OsStr>,
) -> PathBuf {
    smoke_cargo_target_dir(root, work_dir, "external-examples", inherited_target_dir)
}

fn smoke_cargo_target_dir(
    root: &Path,
    work_dir: &Path,
    child: &str,
    inherited_target_dir: Option<&OsStr>,
) -> PathBuf {
    inherited_target_dir
        .filter(|value| !value.is_empty())
        .map(|value| {
            normalize_target_dir(root, PathBuf::from(value))
                .join("external-adoption-smoke")
                .join(child)
        })
        .unwrap_or_else(|| work_dir.join("cargo-target").join(child))
}

fn normalize_target_dir(root: &Path, target_dir: PathBuf) -> PathBuf {
    if target_dir.is_absolute() {
        target_dir
    } else {
        root.join(target_dir)
    }
}

fn run_cli_discovery(
    cli_bin: &Path,
    work_dir: &Path,
    log_dir: &Path,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<()> {
    let project_dir = work_dir.join("cli-discovery");
    fs::create_dir_all(project_dir.join("target"))
        .with_context(|| format!("failed to create {}", project_dir.display()))?;

    let mut doctor = Command::new(cli_bin);
    doctor
        .args(["doctor", "--format", "json"])
        .current_dir(&project_dir);
    let doctor_stdout = run_command_step(
        receipt,
        "cli-doctor-json",
        doctor,
        &project_dir,
        log_dir,
        &[],
    )?;
    verify_doctor_json(&doctor_stdout)?;

    let mut profiles = Command::new(cli_bin);
    profiles.arg("profiles").current_dir(&project_dir);
    run_command_step(
        receipt,
        "cli-profiles",
        profiles,
        &project_dir,
        log_dir,
        &[],
    )?;

    let mut explain = Command::new(cli_bin);
    explain
        .args(["profile", "webhook", "--explain"])
        .current_dir(&project_dir);
    run_command_step(
        receipt,
        "cli-profile-webhook-explain",
        explain,
        &project_dir,
        log_dir,
        &[],
    )?;

    Ok(())
}

fn run_cli_profile(
    cli_bin: &Path,
    profile: &str,
    work_dir: &Path,
    log_dir: &Path,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<()> {
    let project_name = format!("{profile}-cli");
    let project_dir = work_dir.join(&project_name);
    let target_dir = project_dir.join("target");
    let bundle_dir = target_dir.join(format!("uselesskey-{profile}"));
    let inspect_out = target_dir.join(format!("inspect-{profile}.txt"));
    let audit_dir = target_dir.join(format!("audit-{profile}"));
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;

    let bundle_artifact = relative_artifact_from_path(&bundle_dir);
    let inspect_artifact = relative_artifact_from_path(&inspect_out);
    let audit_artifact = relative_artifact_from_path(&audit_dir);

    let mut bundle = Command::new(cli_bin);
    bundle
        .args(["bundle", "--profile", profile, "--out"])
        .arg(&bundle_dir)
        .current_dir(&project_dir);
    run_command_step(
        receipt,
        &format!("cli-bundle-{profile}"),
        bundle,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str()],
    )?;

    let mut verify = Command::new(cli_bin);
    verify
        .arg("verify-bundle")
        .arg(&bundle_dir)
        .current_dir(&project_dir);
    run_command_step(
        receipt,
        &format!("cli-verify-{profile}"),
        verify,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str()],
    )?;

    let mut inspect = Command::new(cli_bin);
    inspect
        .arg("inspect-bundle")
        .arg(&bundle_dir)
        .args(["--out"])
        .arg(&inspect_out)
        .current_dir(&project_dir);
    run_command_step(
        receipt,
        &format!("cli-inspect-{profile}"),
        inspect,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str(), inspect_artifact.as_str()],
    )?;

    let mut audit = Command::new(cli_bin);
    audit
        .arg("audit-bundle")
        .arg(&bundle_dir)
        .args(["--out"])
        .arg(&audit_dir)
        .arg("--ci")
        .args(["--expect-profile", profile])
        .args(["--policy", "strict"])
        .current_dir(&project_dir);
    let audit_stdout = run_command_step(
        receipt,
        &format!("cli-audit-{profile}"),
        audit,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str(), audit_artifact.as_str()],
    )?;
    verify_ci_audit_json(&audit_stdout, profile, "CLI release audit")?;
    verify_ci_audit_receipt(&audit_dir, profile)?;

    verify_bundle_shape(&bundle_dir, profile)?;
    record_project(
        receipt,
        &project_name,
        &project_dir,
        &[bundle_artifact, inspect_artifact, audit_artifact],
    );

    Ok(())
}

fn run_ci_recipes(
    cli_bin: &Path,
    work_dir: &Path,
    log_dir: &Path,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<()> {
    for profile in CI_RECIPE_PROFILES {
        run_ci_recipe_profile(cli_bin, profile, work_dir, log_dir, receipt)?;
    }
    Ok(())
}

fn run_ci_recipe_profile(
    cli_bin: &Path,
    profile: &str,
    work_dir: &Path,
    log_dir: &Path,
    receipt: &mut ExternalAdoptionSmokeReceipt,
) -> Result<()> {
    let project_name = format!("ci-recipe-{profile}");
    let project_dir = work_dir.join(&project_name);
    let target_dir = project_dir.join("target");
    let bundle_dir = target_dir.join(format!("uselesskey-{profile}"));
    let audit_dir = target_dir.join(format!("uselesskey-{profile}-audit"));
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;

    let bundle_artifact = relative_artifact_from_path(&bundle_dir);
    let audit_artifact = relative_artifact_from_path(&audit_dir);

    let mut doctor = Command::new(cli_bin);
    doctor
        .args(["doctor", "--format", "json"])
        .current_dir(&project_dir);
    let doctor_stdout = run_command_step(
        receipt,
        &format!("ci-recipe-doctor-{profile}"),
        doctor,
        &project_dir,
        log_dir,
        &[],
    )?;
    verify_doctor_json(&doctor_stdout)?;

    let mut bundle = Command::new(cli_bin);
    bundle
        .args(["bundle", "--profile", profile, "--out"])
        .arg(&bundle_dir)
        .current_dir(&project_dir);
    run_command_step(
        receipt,
        &format!("ci-recipe-bundle-{profile}"),
        bundle,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str()],
    )?;

    let mut verify = Command::new(cli_bin);
    verify
        .arg("verify-bundle")
        .arg(&bundle_dir)
        .current_dir(&project_dir);
    run_command_step(
        receipt,
        &format!("ci-recipe-verify-{profile}"),
        verify,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str()],
    )?;

    let mut inspect = Command::new(cli_bin);
    inspect
        .arg("inspect-bundle")
        .arg(&bundle_dir)
        .current_dir(&project_dir);
    let inspect_stdout = run_command_step(
        receipt,
        &format!("ci-recipe-inspect-{profile}"),
        inspect,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str()],
    )?;
    verify_ci_inspect_summary(&inspect_stdout, profile)?;

    let mut audit = Command::new(cli_bin);
    audit
        .arg("audit-bundle")
        .arg(&bundle_dir)
        .args(["--out"])
        .arg(&audit_dir)
        .args(["--expect-profile", profile])
        .args(["--policy", "strict"])
        .arg("--ci")
        .current_dir(&project_dir);
    let audit_stdout = run_command_step(
        receipt,
        &format!("ci-recipe-audit-{profile}"),
        audit,
        &project_dir,
        log_dir,
        &[bundle_artifact.as_str(), audit_artifact.as_str()],
    )?;

    verify_ci_audit_json(&audit_stdout, profile, "CI recipe audit stdout")?;
    verify_ci_audit_receipt(&audit_dir, profile)?;
    record_project(
        receipt,
        &project_name,
        &project_dir,
        &[bundle_artifact, audit_artifact],
    );
    Ok(())
}

fn verify_ci_inspect_summary(inspect_path: &Path, expected_profile: &str) -> Result<()> {
    let summary = fs::read_to_string(inspect_path)
        .with_context(|| format!("read {}", inspect_path.display()))?;
    for expected in [
        &format!("Bundle profile: {expected_profile}"),
        "Summary type: quick human bundle summary",
        "Verification: ok",
        "Generated files:",
        "Artifact posture:",
    ] {
        if !summary.contains(expected) {
            bail!(
                "CI recipe inspect summary missing `{expected}` for {}",
                inspect_path.display()
            );
        }
    }
    for forbidden in [
        "-----BEGIN PRIVATE KEY-----",
        "-----BEGIN RSA PRIVATE KEY-----",
        "uk_test_",
    ] {
        if summary.contains(forbidden) {
            bail!(
                "CI recipe inspect summary contains forbidden payload marker `{forbidden}` for {}",
                inspect_path.display()
            );
        }
    }
    Ok(())
}

fn verify_ci_audit_receipt(audit_dir: &Path, expected_profile: &str) -> Result<()> {
    let receipt_path = audit_dir.join("bundle-audit.json");
    verify_ci_audit_json(&receipt_path, expected_profile, "CI recipe audit")?;

    let markdown_path = audit_dir.join("bundle-audit.md");
    if !markdown_path.is_file() {
        bail!(
            "CI recipe audit markdown receipt missing for {expected_profile}: {}",
            markdown_path.display()
        );
    }
    verify_ci_audit_markdown(&markdown_path, expected_profile)?;

    Ok(())
}

fn verify_ci_audit_markdown(markdown_path: &Path, expected_profile: &str) -> Result<()> {
    let markdown = fs::read_to_string(markdown_path)
        .with_context(|| format!("read {}", markdown_path.display()))?;
    for expected in [
        "# uselesskey Bundle Audit",
        "- Status: pass",
        &format!("- Profile: {expected_profile}"),
        "- Receipt type: durable metadata-only reviewer/CI receipt",
        "- Payload posture: raw generated fixture payloads are not copied into this receipt",
    ] {
        if !markdown.contains(expected) {
            bail!(
                "CI recipe audit markdown receipt missing `{expected}` for {}",
                markdown_path.display()
            );
        }
    }
    require_markdown_section_contains(
        &markdown,
        markdown_path,
        "Checks",
        "| Check | Status | Failure class | Detail |",
    )?;
    require_markdown_section_contains(
        &markdown,
        markdown_path,
        "Checks",
        "profile_validation_failed",
    )?;
    require_markdown_section_omits(&markdown, markdown_path, "Checks", "| fail |")?;
    require_markdown_section_contains(
        &markdown,
        markdown_path,
        "Boundaries",
        "- audit receipts contain metadata only and do not copy generated fixture payloads",
    )?;
    require_markdown_section_contains(&markdown, markdown_path, "Does Not Prove", "production")?;
    Ok(())
}

fn require_markdown_section_contains(
    markdown: &str,
    markdown_path: &Path,
    heading: &str,
    expected: &str,
) -> Result<()> {
    let marker = format!("## {heading}");
    let (_, after_marker) = markdown.split_once(&marker).with_context(|| {
        format!(
            "CI recipe audit markdown receipt missing `{marker}` for {}",
            markdown_path.display()
        )
    })?;
    let section = after_marker.split("\n## ").next().unwrap_or(after_marker);
    if !section.contains(expected) {
        bail!(
            "CI recipe audit markdown section `{heading}` missing `{expected}` for {}",
            markdown_path.display()
        );
    }
    Ok(())
}

fn require_markdown_section_omits(
    markdown: &str,
    markdown_path: &Path,
    heading: &str,
    forbidden: &str,
) -> Result<()> {
    let marker = format!("## {heading}");
    let (_, after_marker) = markdown.split_once(&marker).with_context(|| {
        format!(
            "CI recipe audit markdown receipt missing `{marker}` for {}",
            markdown_path.display()
        )
    })?;
    let section = after_marker.split("\n## ").next().unwrap_or(after_marker);
    if section.contains(forbidden) {
        bail!(
            "CI recipe audit markdown section `{heading}` contains forbidden `{forbidden}` for {}",
            markdown_path.display()
        );
    }
    Ok(())
}

fn verify_ci_audit_json(audit_path: &Path, expected_profile: &str, label: &str) -> Result<()> {
    let audit = read_json(audit_path)?;
    if audit["status"].as_str() != Some("pass") {
        bail!(
            "{label} status mismatch for {}: {:?}",
            audit_path.display(),
            audit["status"]
        );
    }
    if audit["profile"].as_str() != Some(expected_profile) {
        bail!(
            "{label} profile mismatch for {}: expected {expected_profile}, got {:?}",
            audit_path.display(),
            audit["profile"]
        );
    }
    let checks = audit["checks"].as_array().with_context(|| {
        format!(
            "{label} checks is not an array for {}",
            audit_path.display()
        )
    })?;
    if checks.is_empty() {
        bail!("{label} checks are empty for {}", audit_path.display());
    }
    for check in checks {
        if check["status"].as_str() != Some("pass") {
            bail!(
                "{label} check did not pass for {}: {:?}",
                audit_path.display(),
                check
            );
        }
        if check["failure_class"].as_str().is_none_or(str::is_empty) {
            bail!(
                "{label} check missing failure_class for {}: {:?}",
                audit_path.display(),
                check
            );
        }
    }
    require_audit_string_array_with(
        &audit,
        audit_path,
        label,
        "boundaries",
        &["metadata only", "generated fixture payloads"],
    )?;
    require_audit_string_array_with(&audit, audit_path, label, "does_not_prove", &["production"])?;
    Ok(())
}

fn require_audit_string_array_with(
    audit: &Value,
    audit_path: &Path,
    label: &str,
    field: &str,
    required_substrings: &[&str],
) -> Result<()> {
    let values = audit[field].as_array().with_context(|| {
        format!(
            "{label} {field} is not an array for {}",
            audit_path.display()
        )
    })?;
    if values.is_empty() {
        bail!("{label} {field} is empty for {}", audit_path.display());
    }

    let mut strings = Vec::with_capacity(values.len());
    for value in values {
        let text = value.as_str().with_context(|| {
            format!(
                "{label} {field} contains a non-string entry for {}: {:?}",
                audit_path.display(),
                value
            )
        })?;
        strings.push(text);
    }

    for expected in required_substrings {
        if !strings.iter().any(|text| text.contains(expected)) {
            bail!(
                "{label} {field} missing entry containing `{expected}` for {}: {:?}",
                audit_path.display(),
                values
            );
        }
    }

    Ok(())
}

fn verify_doctor_json(stdout_path: &Path) -> Result<()> {
    let doctor = read_json(stdout_path)?;
    if doctor["status"].as_str() != Some("pass") {
        bail!(
            "doctor status mismatch for {}: {:?}",
            stdout_path.display(),
            doctor["status"]
        );
    }
    let checks = doctor["checks"]
        .as_array()
        .context("doctor checks is not an array")?;
    if checks.is_empty() {
        bail!("doctor checks are empty for {}", stdout_path.display());
    }
    for check in checks {
        if check["status"].as_str() != Some("pass") {
            bail!(
                "doctor check did not pass for {}: {:?}",
                stdout_path.display(),
                check
            );
        }
    }
    let profiles = doctor["known_profiles"]
        .as_array()
        .context("doctor known_profiles is not an array")?;
    for required_profile in CLI_PROFILES {
        if !profiles
            .iter()
            .any(|profile| profile.as_str() == Some(*required_profile))
        {
            bail!(
                "doctor known_profiles missing {required_profile} in {}",
                stdout_path.display()
            );
        }
    }
    Ok(())
}

fn copy_example_project(source_dir: &Path, project_dir: &Path) -> Result<()> {
    if project_dir.exists() {
        fs::remove_dir_all(project_dir)
            .with_context(|| format!("failed to remove {}", project_dir.display()))?;
    }
    copy_dir_recursive(source_dir, project_dir)
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", source.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            if entry.file_name() == "target" {
                continue;
            }
            copy_dir_recursive(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    entry.path().display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn patch_example_dependencies(project_dir: &Path, source: &SmokeSource) -> Result<()> {
    let manifest_path = project_dir.join("Cargo.toml");
    let mut manifest = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    match &source.facade_dep {
        FacadeDependency::Path(path) => {
            manifest = patch_dependency_path(&manifest, "uselesskey", path);
            let Some(crates_dir) = path.parent() else {
                bail!("failed to find crates dir from {}", path.display());
            };
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-rustls",
                &crates_dir.join("uselesskey-rustls"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-core",
                &crates_dir.join("uselesskey-core"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-webauthn",
                &crates_dir.join("uselesskey-webauthn"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-pkcs11-mock",
                &crates_dir.join("uselesskey-pkcs11-mock"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-ssh",
                &crates_dir.join("uselesskey-ssh"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-pgp",
                &crates_dir.join("uselesskey-pgp"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-hmac",
                &crates_dir.join("uselesskey-hmac"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-entropy",
                &crates_dir.join("uselesskey-entropy"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-ed25519",
                &crates_dir.join("uselesskey-ed25519"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-ecdsa",
                &crates_dir.join("uselesskey-ecdsa"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-rsa",
                &crates_dir.join("uselesskey-rsa"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-jsonwebtoken",
                &crates_dir.join("uselesskey-jsonwebtoken"),
            );
            manifest = patch_dependency_path(
                &manifest,
                "uselesskey-test-server",
                &crates_dir.join("uselesskey-test-server"),
            );
        }
        FacadeDependency::Version(version) => {
            manifest = patch_dependency_version(&manifest, "uselesskey", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-rustls", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-core", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-webauthn", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-pkcs11-mock", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-ssh", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-pgp", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-hmac", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-entropy", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-ed25519", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-ecdsa", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-rsa", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-jsonwebtoken", version);
            manifest = patch_dependency_version(&manifest, "uselesskey-test-server", version);
        }
    }

    fs::write(&manifest_path, manifest)
        .with_context(|| format!("failed to write {}", manifest_path.display()))
}

fn patch_dependency_path(manifest: &str, crate_name: &str, path: &Path) -> String {
    replace_dependency_line(manifest, crate_name, |line| {
        let Some((_, rhs)) = line.split_once('=') else {
            return line.to_string();
        };
        let rhs = rhs.trim();
        let path_part = format!("path = \"{}\"", toml_escape(&path.display().to_string()));
        if rhs.starts_with('"') {
            return format!("{crate_name} = {{ {path_part} }}");
        }
        if !(rhs.starts_with('{') && rhs.ends_with('}')) {
            return line.to_string();
        }

        let inner = rhs.trim_start_matches('{').trim_end_matches('}').trim();
        let mut parts = vec![path_part];
        parts.extend(
            inner
                .split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty() && !part.starts_with("version"))
                .map(ToString::to_string),
        );

        format!("{crate_name} = {{ {} }}", parts.join(", "))
    })
}

fn patch_dependency_version(manifest: &str, crate_name: &str, version: &str) -> String {
    replace_dependency_line(manifest, crate_name, |line| {
        let Some((_, rhs)) = line.split_once('=') else {
            return line.to_string();
        };
        let rhs = rhs.trim();
        if rhs.starts_with('"') {
            return format!("{crate_name} = \"{}\"", toml_escape(version));
        }
        if !(rhs.starts_with('{') && rhs.ends_with('}')) {
            return line.to_string();
        }

        let mut saw_version = false;
        let mut parts: Vec<String> = rhs
            .trim_start_matches('{')
            .trim_end_matches('}')
            .trim()
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(|part| {
                if part.starts_with("version") {
                    saw_version = true;
                    format!("version = \"{}\"", toml_escape(version))
                } else {
                    part.to_string()
                }
            })
            .collect();
        if !saw_version {
            parts.insert(0, format!("version = \"{}\"", toml_escape(version)));
        }

        format!("{crate_name} = {{ {} }}", parts.join(", "))
    })
}

fn replace_dependency_line(
    manifest: &str,
    crate_name: &str,
    replace: impl Fn(&str) -> String,
) -> String {
    let prefix = format!("{crate_name} =");
    let mut output = manifest
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(&prefix) {
                replace(line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    output.push('\n');
    output
}

fn verify_bundle_shape(bundle_dir: &Path, expected_profile: &str) -> Result<()> {
    let manifest_path = bundle_dir.join("manifest.json");
    let manifest: Value = read_json(&manifest_path)?;
    if manifest["profile"].as_str() != Some(expected_profile) {
        bail!(
            "bundle profile mismatch for {}: expected {expected_profile}, got {:?}",
            bundle_dir.display(),
            manifest["profile"]
        );
    }
    let artifacts = manifest["artifacts"]
        .as_array()
        .context("manifest artifacts is not an array")?;
    if artifacts.is_empty() {
        bail!("bundle {} has no artifacts", bundle_dir.display());
    }
    for required in [
        "receipts/materialization.json",
        "receipts/audit-surface.json",
        "receipts/bundle-verification.json",
        "receipts/scanner-safety.json",
        "receipts/negative-coverage.json",
    ] {
        if !bundle_dir.join(required).is_file() {
            bail!(
                "bundle {} is missing expected file {required}",
                bundle_dir.display()
            );
        }
    }
    Ok(())
}

fn resolve_source(options: &RunOptions) -> Result<SmokeSource> {
    match (&options.path, &options.version) {
        (Some(path), None) => {
            let source_root = resolve_existing_path(path)?;
            let facade_dir = if source_root.join("crates/uselesskey/Cargo.toml").exists() {
                source_root.join("crates/uselesskey")
            } else {
                source_root.clone()
            };
            let cli_dir = if source_root
                .join("crates/uselesskey-cli/Cargo.toml")
                .exists()
            {
                source_root.join("crates/uselesskey-cli")
            } else {
                source_root.clone()
            };
            if !facade_dir.join("Cargo.toml").is_file() {
                bail!(
                    "external-adoption-smoke --path could not find uselesskey facade Cargo.toml under {}",
                    source_root.display()
                );
            }
            if !cli_dir.join("Cargo.toml").is_file() {
                bail!(
                    "external-adoption-smoke --path could not find uselesskey-cli Cargo.toml under {}",
                    source_root.display()
                );
            }
            Ok(SmokeSource {
                mode: SmokeMode::Path,
                label: source_root.display().to_string(),
                facade_dep: FacadeDependency::Path(facade_dir),
                cli_source: CliSource::LocalPath(cli_dir),
            })
        }
        (None, Some(version)) if !version.trim().is_empty() => Ok(SmokeSource {
            mode: SmokeMode::Version,
            label: version.clone(),
            facade_dep: FacadeDependency::Version(version.clone()),
            cli_source: CliSource::Version(version.clone()),
        }),
        (None, Some(_)) => bail!("external-adoption-smoke --version must not be empty"),
        (None, None) => {
            bail!("external-adoption-smoke requires exactly one of --path or --version")
        }
        (Some(_), Some(_)) => {
            bail!("external-adoption-smoke: --path and --version are mutually exclusive")
        }
    }
}

fn validate_run_options(options: &RunOptions) -> Result<()> {
    if options.ci_recipes && options.library_examples {
        bail!(
            "external-adoption-smoke: --library-examples and --ci-recipes are mutually exclusive"
        );
    }
    Ok(())
}

fn resolve_existing_path(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("failed to read current dir")?
            .join(path)
    };
    if !absolute.exists() {
        bail!("path does not exist: {}", absolute.display());
    }
    Ok(absolute)
}

fn run_command_step(
    receipt: &mut ExternalAdoptionSmokeReceipt,
    name: &str,
    mut cmd: Command,
    cwd: &Path,
    log_dir: &Path,
    artifacts: &[&str],
) -> Result<PathBuf> {
    eprintln!("==> {name}");
    cmd.stdin(Stdio::null());
    let command = command_to_vec(&cmd);
    let stdout_path = log_dir.join(format!("{name}.stdout.txt"));
    let stderr_path = log_dir.join(format!("{name}.stderr.txt"));
    let start = Instant::now();
    let output = cmd
        .output()
        .with_context(|| format!("failed to spawn {name}"))?;
    let duration_ms = start.elapsed().as_millis() as u64;
    fs::write(&stdout_path, &output.stdout)
        .with_context(|| format!("failed to write {}", stdout_path.display()))?;
    fs::write(&stderr_path, &output.stderr)
        .with_context(|| format!("failed to write {}", stderr_path.display()))?;

    let stdout = relative_artifact_from_path(&stdout_path);
    let stderr = relative_artifact_from_path(&stderr_path);
    if output.status.success() {
        eprintln!("==> {name} [ok]");
        receipt.steps.push(ExternalAdoptionStep {
            name: name.to_string(),
            command,
            cwd: cwd.display().to_string(),
            status: "ok".to_string(),
            duration_ms,
            stdout,
            stderr,
            details: None,
            artifacts: artifacts
                .iter()
                .map(|artifact| (*artifact).to_string())
                .collect(),
        });
        Ok(stdout_path)
    } else {
        let details = format!("command failed with {}", output.status);
        eprintln!("==> {name} [FAILED]");
        eprintln!("    {details}");
        receipt.steps.push(ExternalAdoptionStep {
            name: name.to_string(),
            command,
            cwd: cwd.display().to_string(),
            status: "failed".to_string(),
            duration_ms,
            stdout,
            stderr,
            details: Some(details.clone()),
            artifacts: artifacts
                .iter()
                .map(|artifact| (*artifact).to_string())
                .collect(),
        });
        bail!("{name}: {details}")
    }
}

fn command_to_vec(cmd: &Command) -> Vec<String> {
    let mut command = Vec::new();
    command.push(cmd.get_program().to_string_lossy().into_owned());
    command.extend(cmd.get_args().map(|arg| arg.to_string_lossy().into_owned()));
    command
}

fn record_project(
    receipt: &mut ExternalAdoptionSmokeReceipt,
    name: &str,
    path: &Path,
    outputs: &[String],
) {
    receipt.projects.push(ExternalAdoptionProject {
        name: name.to_string(),
        path: relative_artifact_from_path(path),
        status: "ok".to_string(),
        outputs: outputs.to_vec(),
    });
}

fn reset_target_output(root: &Path, out_root: &Path) -> Result<()> {
    ensure_target_child(root, out_root)?;
    if out_root.exists() {
        fs::remove_dir_all(out_root)
            .with_context(|| format!("failed to remove {}", out_root.display()))?;
    }
    fs::create_dir_all(out_root).with_context(|| format!("failed to create {}", out_root.display()))
}

fn ensure_target_child(root: &Path, path: &Path) -> Result<()> {
    let absolute = absolute_path(root, path);
    let target_root = absolute_path(root, &root.join("target"));
    if !absolute.starts_with(&target_root) {
        bail!(
            "external-adoption-smoke refuses to write outside target/: {}",
            path.display()
        );
    }
    Ok(())
}

fn absolute_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn write_receipts(out_dir: &Path, receipt: &ExternalAdoptionSmokeReceipt) -> Result<()> {
    fs::write(
        out_dir.join("report.json"),
        serde_json::to_string_pretty(receipt).context("failed to serialize adoption smoke")?,
    )
    .with_context(|| format!("failed to write {}", out_dir.join("report.json").display()))?;
    fs::write(out_dir.join("report.md"), render_markdown(receipt))
        .with_context(|| format!("failed to write {}", out_dir.join("report.md").display()))
}

fn render_markdown(receipt: &ExternalAdoptionSmokeReceipt) -> String {
    let mut md = String::new();
    md.push_str("# External Adoption Smoke Receipt\n\n");
    md.push_str(&format!("Status: `{}`\n\n", receipt.status));
    md.push_str(&format!("Mode: `{:?}`\n\n", receipt.mode));
    md.push_str(&format!("Source: `{}`\n\n", receipt.source));
    md.push_str(&format!(
        "Library examples only: `{}`\n\n",
        receipt.library_examples
    ));
    md.push_str(&format!("CI recipes: `{}`\n\n", receipt.ci_recipes));
    if let Some(git_sha) = &receipt.git_sha {
        md.push_str(&format!("Git SHA: `{git_sha}`\n\n"));
    }

    md.push_str("## Projects\n\n");
    md.push_str("| Project | Status | Path | Outputs |\n");
    md.push_str("| --- | --- | --- | --- |\n");
    for project in &receipt.projects {
        md.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` |\n",
            project.name,
            project.status,
            project.path,
            project.outputs.join("<br>")
        ));
    }

    md.push_str("\n## Steps\n\n");
    md.push_str("| Step | Status | Command | Logs | Artifacts | Details |\n");
    md.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for step in &receipt.steps {
        let details = step.details.as_deref().unwrap_or("");
        let artifacts = if step.artifacts.is_empty() {
            String::new()
        } else {
            step.artifacts.join("<br>")
        };
        md.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` / `{}` | `{}` | {} |\n",
            step.name,
            step.status,
            step.command.join(" "),
            step.stdout,
            step.stderr,
            artifacts,
            details
        ));
    }

    md.push_str("\n## Boundaries\n\n");
    for boundary in &receipt.boundaries {
        md.push_str(&format!("- {boundary}\n"));
    }

    md
}

fn print_human(receipt: &ExternalAdoptionSmokeReceipt) {
    println!(
        "external-adoption-smoke: {} (projects={}, steps={})",
        receipt.status,
        receipt.projects.len(),
        receipt.steps.len()
    );
    println!("external-adoption-smoke: wrote {REPORT_JSON} and {REPORT_MD}");
}

fn read_json(path: &Path) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn cli_binary_name() -> &'static str {
    if cfg!(windows) {
        "uselesskey.exe"
    } else {
        "uselesskey"
    }
}

fn relative_artifact(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn relative_artifact_from_path(path: &Path) -> String {
    if let Ok(cwd) = std::env::current_dir()
        && let Ok(stripped) = path.strip_prefix(&cwd)
    {
        return stripped.to_string_lossy().replace('\\', "/");
    }
    path.to_string_lossy().replace('\\', "/")
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_adoption_profiles_are_bounded() {
        assert_eq!(CLI_PROFILES, ["scanner-safe", "tls", "oidc", "webhook"]);
    }

    #[test]
    fn external_adoption_ci_recipe_profiles_are_bounded() {
        assert_eq!(
            CI_RECIPE_PROFILES,
            ["scanner-safe", "oidc", "webhook", "tls"]
        );
    }

    #[test]
    fn external_adoption_ci_recipe_profiles_match_actions_matrix() {
        let example = include_str!(
            "../../examples/external/ci-recipes/github-actions-bundle-verify-audit.yml.example"
        );

        assert_eq!(
            actions_matrix_profiles(example),
            CI_RECIPE_PROFILES.to_vec()
        );
    }

    #[test]
    fn external_adoption_ci_recipe_actions_include_inspect_step() {
        let example = include_str!(
            "../../examples/external/ci-recipes/github-actions-bundle-verify-audit.yml.example"
        );

        assert!(
            example.contains("uselesskey inspect-bundle"),
            "GitHub Actions recipe should preserve the bundle -> verify -> inspect -> audit path"
        );
    }

    #[test]
    fn external_adoption_ci_recipe_actions_include_doctor_preflight() {
        let example = include_str!(
            "../../examples/external/ci-recipes/github-actions-bundle-verify-audit.yml.example"
        );

        assert!(
            example.contains("uselesskey doctor --format json"),
            "GitHub Actions recipe should run installed CLI doctor before generating fixtures"
        );
    }

    #[test]
    fn external_adoption_top_level_docs_include_audit_receipt_upload_step() {
        let readme = include_str!("../../README.md");
        let start_here = include_str!("../../docs/how-to/start-here.md");
        let downstream_ci = include_str!("../../docs/how-to/use-uselesskey-in-downstream-ci.md");

        for (name, doc) in [
            ("README.md", readme),
            ("docs/how-to/start-here.md", start_here),
            (
                "docs/how-to/use-uselesskey-in-downstream-ci.md",
                downstream_ci,
            ),
        ] {
            for expected in [
                "actions/upload-artifact@v7",
                "if: always()",
                "target/uselesskey-webhook-audit/bundle-audit.json",
                "target/uselesskey-webhook-audit/bundle-audit.md",
                "if-no-files-found: error",
            ] {
                assert!(doc.contains(expected), "{name} missing `{expected}`");
            }
        }

        assert!(
            readme.contains("| upload metadata-only receipts |"),
            "README Start Here table should route users to audit receipt upload"
        );
        assert!(
            start_here.contains("use-uselesskey-in-github-actions.md#upload-audit-receipts"),
            "start-here should link directly to the GitHub Actions upload section"
        );
        assert!(
            downstream_ci.contains("Upload metadata-only audit receipts"),
            "downstream CI copy block should include an upload step"
        );
    }

    #[test]
    fn external_adoption_ci_recipe_profiles_are_supported_cli_profiles() {
        for profile in CI_RECIPE_PROFILES {
            assert!(
                CLI_PROFILES.contains(profile),
                "ci recipe profile `{profile}` must be a supported installed CLI smoke profile"
            );
        }
    }

    #[test]
    fn external_adoption_ci_recipe_examples_are_bounded() {
        let names: Vec<&str> = CI_RECIPE_EXAMPLES
            .iter()
            .map(|example| example.name)
            .collect();
        assert_eq!(
            names,
            [
                "rust-test-fixtures",
                "webhook-verifier",
                "oidc-jwks-validation",
                "tls-chain-validation",
                "downstream-ci-bundle-audit",
            ]
        );
    }

    #[test]
    fn external_adoption_ci_recipes_readme_lists_recipe_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask lives under the workspace root");
        let recipes_dir = root.join("examples/external/ci-recipes");
        let recipe_files = fs::read_dir(&recipes_dir)
            .expect("ci-recipes directory should exist")
            .map(|entry| {
                entry
                    .expect("ci-recipes entry should be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name != "README.md")
            .collect::<std::collections::BTreeSet<_>>();
        let doc = include_str!("../../examples/external/ci-recipes/README.md");
        let readme_links = doc
            .lines()
            .filter(|line| line.starts_with('|') && line.contains("]("))
            .filter_map(|line| {
                let href = line.split("](").nth(1)?.split(')').next()?.trim();
                if href.contains('/') {
                    None
                } else {
                    Some(href.to_string())
                }
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            readme_links, recipe_files,
            "ci-recipes README should link every committed recipe file and no missing recipe files"
        );
    }

    #[test]
    fn external_adoption_library_examples_are_bounded() {
        let names: Vec<&str> = LIBRARY_EXAMPLES
            .iter()
            .map(|example| example.name)
            .collect();
        assert_eq!(
            names,
            [
                "rust-test-fixtures",
                "webhook-verifier",
                "oidc-jwks-validation",
                "oidc-test-server-validation",
                "tls-chain-validation",
                "webauthn-ceremony-validation",
                "pkcs11-mock-validation",
                "ssh-fixture-validation",
                "pgp-fixture-validation",
                "hmac-signature-validation",
                "jsonwebtoken-adapter-validation",
                "entropy-byte-fixtures",
                "ecdsa-fixture-validation",
                "ed25519-fixture-validation",
            ]
        );
    }

    #[test]
    fn external_adoption_rejects_conflicting_direct_modes() {
        let err = run(
            Path::new("."),
            RunOptions {
                path: Some(PathBuf::from(".")),
                version: None,
                ci_recipes: true,
                library_examples: true,
                format: OutputFormat::Human,
            },
        )
        .expect_err("direct callers must not combine CI recipes with library examples");

        assert!(
            err.to_string()
                .contains("--library-examples and --ci-recipes are mutually exclusive"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn external_adoption_examples_are_bounded() {
        let names: Vec<&str> = EXTERNAL_EXAMPLES
            .iter()
            .map(|example| example.name)
            .collect();
        assert_eq!(
            names,
            [
                "rust-test-fixtures",
                "webhook-verifier",
                "oidc-jwks-validation",
                "oidc-test-server-validation",
                "tls-chain-validation",
                "webauthn-ceremony-validation",
                "pkcs11-mock-validation",
                "ssh-fixture-validation",
                "pgp-fixture-validation",
                "hmac-signature-validation",
                "jsonwebtoken-adapter-validation",
                "entropy-byte-fixtures",
                "ecdsa-fixture-validation",
                "ed25519-fixture-validation",
                "downstream-ci-bundle-audit",
            ]
        );
    }

    #[test]
    fn external_adoption_examples_readme_lists_runnable_examples() {
        let doc = include_str!("../../examples/external/README.md");
        let library_proof = "cargo xtask external-adoption-smoke --path . --library-examples";

        for example in LIBRARY_EXAMPLES {
            let link = format!(
                "]({}/)",
                example
                    .source_dir
                    .strip_prefix("examples/external/")
                    .expect("external example source is under examples/external")
            );
            let row = doc
                .lines()
                .find(|line| line.contains(&link))
                .unwrap_or_else(|| {
                    panic!(
                        "external examples README missing runnable example `{}`",
                        example.name
                    )
                });
            assert!(
                row.contains(library_proof),
                "external examples README row for `{}` should use library-example proof",
                example.name
            );
        }

        let downstream_row = doc
            .lines()
            .find(|line| line.contains("](downstream-ci-bundle-audit/)"))
            .expect("external examples README lists downstream CI bundle audit example");
        assert!(
            downstream_row.contains("cargo xtask external-adoption-smoke --path ."),
            "downstream CI bundle audit row should use the installed CLI path proof"
        );
        assert!(
            !downstream_row.contains("--library-examples"),
            "downstream CI bundle audit row should not use library-example proof"
        );

        let ci_recipes_row = doc
            .lines()
            .find(|line| line.contains("](ci-recipes/)"))
            .expect("external examples README lists CI recipes");
        assert!(
            ci_recipes_row.contains(
                "cargo xtask external-adoption-smoke --path . --ci-recipes --format json"
            ),
            "CI recipes row should use the CI recipe proof mode"
        );
    }

    #[test]
    fn external_adoption_examples_readme_lists_example_directories() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask lives under the workspace root");
        let examples_dir = root.join("examples/external");
        let example_dirs = fs::read_dir(&examples_dir)
            .expect("external examples directory should exist")
            .filter_map(|entry| {
                let entry = entry.expect("external example entry should be readable");
                if entry.path().is_dir() {
                    Some(entry.file_name().to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        let doc = include_str!("../../examples/external/README.md");
        let readme_links = doc
            .lines()
            .filter(|line| line.starts_with('|') && line.contains("]("))
            .filter_map(|line| {
                let href = line.split("](").nth(1)?.split(')').next()?.trim();
                let name = href.strip_suffix('/')?;
                if name.contains('/') {
                    None
                } else {
                    Some(name.to_string())
                }
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            readme_links, example_dirs,
            "external examples README should link every committed example directory and no stale example directories"
        );
    }

    #[test]
    fn external_adoption_workflow_support_lists_smoked_examples() {
        let doc = include_str!("../../docs/status/workflow-support.md");

        for example in EXTERNAL_EXAMPLES {
            let readme = format!("`{}/README.md`", example.source_dir);
            assert!(
                doc.contains(&readme),
                "workflow support matrix missing smoked example `{}`",
                example.name
            );
        }

        assert!(
            doc.contains("`examples/external/ci-recipes/README.md`"),
            "workflow support matrix should link the downstream CI recipe pack"
        );
        assert!(
            doc.contains("cargo xtask external-adoption-smoke --path . --ci-recipes --format json"),
            "workflow support matrix should name the CI recipe proof mode"
        );
    }

    #[test]
    fn external_adoption_workflow_support_doc_paths_exist() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask lives under the workspace root");
        let doc = include_str!("../../docs/status/workflow-support.md");

        for path in doc
            .split('`')
            .skip(1)
            .step_by(2)
            .filter(|path| path.starts_with("docs/") || path.starts_with("examples/external/"))
        {
            assert!(
                root.join(path).is_file(),
                "workflow support matrix references missing doc path `{path}`"
            );
        }
    }

    #[test]
    fn external_adoption_release_facade_record_lists_library_examples() {
        let doc = include_str!("../../docs/release/v0.10.0-facade-release-smoke.md");

        for example in LIBRARY_EXAMPLES {
            assert!(
                doc.contains(&format!("`{}`", example.source_dir)),
                "facade release smoke record missing library example `{}`",
                example.name
            );
        }

        assert!(
            doc.contains("cargo xtask external-adoption-smoke --path . --library-examples"),
            "facade release smoke record should name the library-example proof command"
        );
        assert!(
            doc.contains("external_adoption_library_examples_are_bounded"),
            "facade release smoke record should name the regression test that owns the matrix"
        );
    }

    #[test]
    fn external_adoption_downstream_bundle_audit_workflow_is_current() {
        let example = include_str!(
            "../../examples/external/downstream-ci-bundle-audit/.github/workflows/uselesskey-audit.yml.example"
        );

        for expected in [
            "uselesskey doctor --format json",
            "uselesskey verify-bundle target/uselesskey-webhook",
            "uselesskey inspect-bundle target/uselesskey-webhook",
            "--expect-profile webhook",
            "--policy strict",
        ] {
            assert!(
                example.contains(expected),
                "downstream bundle audit workflow missing `{expected}`"
            );
        }

        assert!(
            !example.contains("--path target/uselesskey-webhook"),
            "downstream bundle audit workflow should use the current positional bundle path form"
        );
    }

    #[test]
    fn external_adoption_installed_audit_handoff_doc_is_current() {
        let doc = include_str!("../../docs/how-to/share-installed-bundle-audit.md");

        for expected in [
            "uselesskey doctor --format json",
            "uselesskey verify-bundle target/uselesskey-webhook",
            "uselesskey inspect-bundle target/uselesskey-webhook",
            "--expect-profile webhook",
            "--policy strict",
        ] {
            assert!(
                doc.contains(expected),
                "installed audit handoff doc missing `{expected}`"
            );
        }
    }

    #[test]
    fn external_examples_share_target_dir() {
        let root = Path::new("repo");
        let work_dir = Path::new("repo/target/external-adoption-smoke/work");

        assert_eq!(
            external_examples_target_dir(root, work_dir, None),
            PathBuf::from(
                "repo/target/external-adoption-smoke/work/cargo-target/external-examples"
            )
        );
    }

    #[test]
    fn external_examples_target_dir_uses_inherited_cargo_target_dir() {
        let root = Path::new("repo");
        let work_dir = Path::new("repo/target/external-adoption-smoke/work");
        let inherited = std::env::temp_dir().join("uselesskey-external-target");

        assert_eq!(
            external_examples_target_dir(root, work_dir, Some(inherited.as_os_str())),
            inherited
                .join("external-adoption-smoke")
                .join("external-examples")
        );
    }

    #[test]
    fn smoke_target_dir_resolves_relative_inherited_cargo_target_dir_from_root() {
        let root = Path::new("repo");
        let work_dir = Path::new("repo/target/external-adoption-smoke/work");

        assert_eq!(
            smoke_cargo_target_dir(root, work_dir, "local-cli", Some(OsStr::new("cargo-cache"))),
            PathBuf::from("repo")
                .join("cargo-cache")
                .join("external-adoption-smoke")
                .join("local-cli")
        );
    }

    #[test]
    fn smoke_target_dir_ignores_empty_inherited_cargo_target_dir() {
        let root = Path::new("repo");
        let work_dir = Path::new("repo/target/external-adoption-smoke/work");

        assert_eq!(
            smoke_cargo_target_dir(root, work_dir, "local-cli", Some(OsStr::new(""))),
            PathBuf::from("repo/target/external-adoption-smoke/work/cargo-target/local-cli")
        );
    }

    #[test]
    fn external_adoption_dependency_patch_switches_versions_to_paths() {
        let manifest = r#"[dependencies]
uselesskey = { version = "0.9.1", default-features = false, features = ["rsa"] }
uselesskey-rustls = { version = "0.9.1", features = ["tls-config", "rustls-ring"] }
uselesskey-core = "0.9.1"
uselesskey-webauthn = "0.9.1"
uselesskey-pkcs11-mock = "0.9.1"
uselesskey-ssh = "0.9.1"
uselesskey-pgp = "0.9.1"
uselesskey-hmac = "0.9.1"
uselesskey-entropy = "0.9.1"
uselesskey-ed25519 = "0.9.1"
uselesskey-ecdsa = "0.9.1"
uselesskey-rsa = "0.9.1"
uselesskey-jsonwebtoken = { version = "0.9.1", features = ["rsa", "hmac"] }
uselesskey-test-server = "0.9.1"
"#;

        let crates_dir = Path::new(r#"C:\Code\Rust\uselesskey\crates"#);
        let mut patched = manifest.to_string();
        for crate_name in [
            "uselesskey",
            "uselesskey-rustls",
            "uselesskey-core",
            "uselesskey-webauthn",
            "uselesskey-pkcs11-mock",
            "uselesskey-ssh",
            "uselesskey-pgp",
            "uselesskey-hmac",
            "uselesskey-entropy",
            "uselesskey-ed25519",
            "uselesskey-ecdsa",
            "uselesskey-rsa",
            "uselesskey-jsonwebtoken",
            "uselesskey-test-server",
        ] {
            patched = patch_dependency_path(&patched, crate_name, &crates_dir.join(crate_name));
        }

        let expected_path =
            |crate_name: &str| toml_escape(&crates_dir.join(crate_name).display().to_string());
        assert!(patched.contains(&format!(
            r#"uselesskey = {{ path = "{}", default-features = false, features = ["rsa"] }}"#,
            expected_path("uselesskey")
        )));
        assert!(patched.contains(&format!(
            r#"uselesskey-rustls = {{ path = "{}", features = ["tls-config", "rustls-ring"] }}"#,
            expected_path("uselesskey-rustls")
        )));
        for crate_name in [
            "uselesskey-core",
            "uselesskey-webauthn",
            "uselesskey-pkcs11-mock",
            "uselesskey-ssh",
            "uselesskey-pgp",
            "uselesskey-hmac",
            "uselesskey-entropy",
            "uselesskey-ed25519",
            "uselesskey-ecdsa",
            "uselesskey-rsa",
            "uselesskey-test-server",
        ] {
            assert!(patched.contains(&format!(
                r#"{crate_name} = {{ path = "{}" }}"#,
                expected_path(crate_name)
            )));
        }
        assert!(patched.contains(&format!(
            r#"uselesskey-jsonwebtoken = {{ path = "{}", features = ["rsa", "hmac"] }}"#,
            expected_path("uselesskey-jsonwebtoken")
        )));
        assert!(!patched.contains("version = \"0.9.1\""));
    }

    #[test]
    fn external_adoption_rejects_non_target_output() -> Result<()> {
        let root = std::env::temp_dir().join("uselesskey-external-adoption-test");
        let outside = root
            .parent()
            .context("temp-dir test root has no parent")?
            .join("outside-external-adoption");
        let err = match ensure_target_child(&root, &outside) {
            Ok(()) => bail!("non-target output was accepted"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("outside target"));
        Ok(())
    }

    #[test]
    fn external_adoption_accepts_target_output() -> Result<()> {
        let root = std::env::temp_dir().join("uselesskey-external-adoption-test");

        ensure_target_child(&root, &root.join("target/external-adoption-smoke"))?;
        Ok(())
    }

    #[test]
    fn external_adoption_toml_escape_handles_windows_paths() {
        assert_eq!(
            toml_escape(r#"C:\Code\Rust\uselesskey\crates\uselesskey"#),
            r#"C:\\Code\\Rust\\uselesskey\\crates\\uselesskey"#
        );
    }

    #[test]
    fn external_adoption_markdown_lists_cli_loop_and_boundaries() {
        let receipt = ExternalAdoptionSmokeReceipt {
            schema_version: 1,
            status: "pass".to_string(),
            generated_at: "2026-05-17T00:00:00Z".to_string(),
            git_sha: Some("abc123".to_string()),
            mode: SmokeMode::Path,
            source: ".".to_string(),
            work_root: WORK_DIR.to_string(),
            ci_recipes: true,
            library_examples: false,
            projects: vec![ExternalAdoptionProject {
                name: "webhook-cli".to_string(),
                path: "target/external-adoption-smoke/work/webhook-cli".to_string(),
                status: "ok".to_string(),
                outputs: vec![
                    "target/external-adoption-smoke/work/webhook-cli/target/uselesskey-webhook"
                        .to_string(),
                    "target/external-adoption-smoke/work/webhook-cli/target/inspect-webhook.txt"
                        .to_string(),
                    "target/external-adoption-smoke/work/webhook-cli/target/audit-webhook"
                        .to_string(),
                ],
            }],
            steps: vec![
                ExternalAdoptionStep {
                    name: "cli-bundle-webhook".to_string(),
                    command: vec![
                        "uselesskey".to_string(),
                        "bundle".to_string(),
                        "--profile".to_string(),
                        "webhook".to_string(),
                    ],
                    cwd: ".".to_string(),
                    status: "ok".to_string(),
                    duration_ms: 1,
                    stdout: "target/external-adoption-smoke/logs/stdout.txt".to_string(),
                    stderr: "target/external-adoption-smoke/logs/stderr.txt".to_string(),
                    details: None,
                    artifacts: Vec::new(),
                },
                ExternalAdoptionStep {
                    name: "cli-verify-webhook".to_string(),
                    command: vec![
                        "uselesskey".to_string(),
                        "verify-bundle".to_string(),
                        "target/uselesskey-webhook".to_string(),
                    ],
                    cwd: ".".to_string(),
                    status: "ok".to_string(),
                    duration_ms: 1,
                    stdout: "target/external-adoption-smoke/logs/verify-stdout.txt".to_string(),
                    stderr: "target/external-adoption-smoke/logs/verify-stderr.txt".to_string(),
                    details: None,
                    artifacts: vec![
                        "target/external-adoption-smoke/work/webhook-cli/target/uselesskey-webhook"
                            .to_string(),
                    ],
                },
                ExternalAdoptionStep {
                    name: "cli-inspect-webhook".to_string(),
                    command: vec![
                        "uselesskey".to_string(),
                        "inspect-bundle".to_string(),
                        "target/uselesskey-webhook".to_string(),
                        "--out".to_string(),
                        "target/inspect-webhook.txt".to_string(),
                    ],
                    cwd: ".".to_string(),
                    status: "ok".to_string(),
                    duration_ms: 1,
                    stdout: "target/external-adoption-smoke/logs/inspect-stdout.txt".to_string(),
                    stderr: "target/external-adoption-smoke/logs/inspect-stderr.txt".to_string(),
                    details: None,
                    artifacts: vec![
                        "target/external-adoption-smoke/work/webhook-cli/target/inspect-webhook.txt"
                            .to_string(),
                    ],
                },
                ExternalAdoptionStep {
                    name: "cli-audit-webhook".to_string(),
                    command: vec![
                        "uselesskey".to_string(),
                        "audit-bundle".to_string(),
                        "--path".to_string(),
                        "target/uselesskey-webhook".to_string(),
                    ],
                    cwd: ".".to_string(),
                    status: "ok".to_string(),
                    duration_ms: 1,
                    stdout: "target/external-adoption-smoke/logs/audit-stdout.txt".to_string(),
                    stderr: "target/external-adoption-smoke/logs/audit-stderr.txt".to_string(),
                    details: None,
                    artifacts: vec![
                        "target/external-adoption-smoke/work/webhook-cli/target/audit-webhook"
                            .to_string(),
                    ],
                },
            ],
            artifacts: vec![REPORT_JSON.to_string(), REPORT_MD.to_string()],
            boundaries: BOUNDARIES.to_vec(),
        };

        let markdown = render_markdown(&receipt);
        assert!(markdown.contains("External Adoption Smoke Receipt"));
        assert!(markdown.contains("installed-style CLI smoke does not claim provider"));
        assert!(markdown.contains("child Cargo build caches may use CARGO_TARGET_DIR"));
        let bundle_pos = markdown.find("cli-bundle-webhook").expect("bundle step");
        let verify_pos = markdown.find("cli-verify-webhook").expect("verify step");
        let inspect_pos = markdown.find("cli-inspect-webhook").expect("inspect step");
        let audit_pos = markdown.find("cli-audit-webhook").expect("audit step");
        assert!(bundle_pos < verify_pos);
        assert!(verify_pos < inspect_pos);
        assert!(inspect_pos < audit_pos);
        assert!(markdown.contains("| Step | Status | Command | Logs | Artifacts | Details |"));
        let inspect_row = markdown
            .lines()
            .find(|line| line.contains("cli-inspect-webhook"))
            .expect("inspect step row");
        let audit_row = markdown
            .lines()
            .find(|line| line.contains("cli-audit-webhook"))
            .expect("audit step row");
        assert!(
            inspect_row.contains(
                "`target/external-adoption-smoke/work/webhook-cli/target/inspect-webhook.txt`"
            ),
            "{markdown}"
        );
        assert!(
            audit_row
                .contains("target/external-adoption-smoke/work/webhook-cli/target/audit-webhook"),
            "{markdown}"
        );
    }

    #[test]
    fn external_adoption_accepts_ci_audit_receipt_pair() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        write_ci_audit_markdown(dir.path(), "oidc")?;

        verify_ci_audit_receipt(dir.path(), "oidc")
    }

    #[test]
    fn external_adoption_rejects_ci_audit_without_markdown_receipt() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit receipt without markdown was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("markdown receipt missing"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_placeholder_markdown_receipt() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        fs::write(dir.path().join("bundle-audit.md"), "# Bundle Audit\n")?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit placeholder markdown was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("# uselesskey Bundle Audit"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_wrong_profile_markdown_receipt() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        write_ci_audit_markdown(dir.path(), "webhook")?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit markdown with wrong profile was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("- Profile: oidc"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_failed_status_markdown_receipt() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        fs::write(
            dir.path().join("bundle-audit.md"),
            concat!(
                "# uselesskey Bundle Audit\n\n",
                "- Status: fail\n",
                "- Bundle: target/uselesskey-oidc\n",
                "- Profile: oidc\n",
                "- Receipt type: durable metadata-only reviewer/CI receipt\n",
                "- Payload posture: raw generated fixture payloads are not copied into this receipt\n",
                "\n## Boundaries\n\n",
                "- audit receipts contain metadata only and do not copy generated fixture payloads\n",
                "\n## Does Not Prove\n\n",
                "- production signing-key custody\n",
            ),
        )?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit markdown with fail status was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("- Status: pass"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_markdown_without_checks_section() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        fs::write(
            dir.path().join("bundle-audit.md"),
            concat!(
                "# uselesskey Bundle Audit\n\n",
                "- Status: pass\n",
                "- Bundle: target/uselesskey-oidc\n",
                "- Profile: oidc\n",
                "- Receipt type: durable metadata-only reviewer/CI receipt\n",
                "- Payload posture: raw generated fixture payloads are not copied into this receipt\n",
                "\n## Boundaries\n\n",
                "- audit receipts contain metadata only and do not copy generated fixture payloads\n",
                "\n## Does Not Prove\n\n",
                "- production signing-key custody\n",
            ),
        )?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit markdown without checks section was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("## Checks"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_markdown_failed_check_row() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        fs::write(
            dir.path().join("bundle-audit.md"),
            concat!(
                "# uselesskey Bundle Audit\n\n",
                "- Status: pass\n",
                "- Bundle: target/uselesskey-oidc\n",
                "- Profile: oidc\n",
                "- Receipt type: durable metadata-only reviewer/CI receipt\n",
                "- Payload posture: raw generated fixture payloads are not copied into this receipt\n",
                "\n## Checks\n\n",
                "| Check | Status | Failure class | Detail |\n",
                "|---|---|---|---|\n",
                "| profile-validation | fail | profile_validation_failed | simulated stale failure |\n",
                "\n## Boundaries\n\n",
                "- audit receipts contain metadata only and do not copy generated fixture payloads\n",
                "\n## Does Not Prove\n\n",
                "- production signing-key custody\n",
            ),
        )?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit markdown with failed check row was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("forbidden `| fail |`"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_markdown_without_metadata_boundary() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        fs::write(
            dir.path().join("bundle-audit.md"),
            concat!(
                "# uselesskey Bundle Audit\n\n",
                "- Status: pass\n",
                "- Bundle: target/uselesskey-oidc\n",
                "- Profile: oidc\n",
                "- Receipt type: durable metadata-only reviewer/CI receipt\n",
                "- Payload posture: raw generated fixture payloads are not copied into this receipt\n",
                "\n## Checks\n\n",
                "| Check | Status | Failure class | Detail |\n",
                "|---|---|---|---|\n",
                "| profile-validation | pass | profile_validation_failed | profile-specific generated files match the manifest |\n",
                "\n## Boundaries\n\n",
                "- audit-bundle proves local bundle consistency only\n",
                "\n## Does Not Prove\n\n",
                "- production signing-key custody\n",
            ),
        )?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit markdown without metadata boundary was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains(
            "- audit receipts contain metadata only and do not copy generated fixture payloads"
        ));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_markdown_without_production_non_claim() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ci_audit_json(dir.path(), "oidc")?;
        fs::write(
            dir.path().join("bundle-audit.md"),
            concat!(
                "# uselesskey Bundle Audit\n\n",
                "- Status: pass\n",
                "- Bundle: target/uselesskey-oidc\n",
                "- Profile: oidc\n",
                "- Receipt type: durable metadata-only reviewer/CI receipt\n",
                "- Payload posture: raw generated fixture payloads are not copied into this receipt\n",
                "\n## Checks\n\n",
                "| Check | Status | Failure class | Detail |\n",
                "|---|---|---|---|\n",
                "| profile-validation | pass | profile_validation_failed | profile-specific generated files match the manifest |\n",
                "\n## Boundaries\n\n",
                "- audit receipts contain metadata only and do not copy generated fixture payloads\n",
                "- audit-bundle is not production security proof\n",
                "\n## Does Not Prove\n\n",
                "- downstream validator correctness\n",
            ),
        )?;

        let err = match verify_ci_audit_receipt(dir.path(), "oidc") {
            Ok(()) => bail!("CI audit markdown without production non-claim was accepted"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("section `Does Not Prove` missing `production`")
        );
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_without_checks() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(
            dir.path().join("bundle-audit.json"),
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "profile": "oidc",
            }))?,
        )?;

        let err = match verify_ci_audit_json(
            &dir.path().join("bundle-audit.json"),
            "oidc",
            "CI recipe audit",
        ) {
            Ok(()) => bail!("CI audit without checks was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("checks is not an array"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_failed_check() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(
            dir.path().join("bundle-audit.json"),
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "profile": "oidc",
                "checks": [{
                    "name": "bundle-audit",
                    "status": "fail",
                    "failure_class": "profile_validation_failed",
                    "detail": "simulated failure",
                }],
            }))?,
        )?;

        let err = match verify_ci_audit_json(
            &dir.path().join("bundle-audit.json"),
            "oidc",
            "CI recipe audit",
        ) {
            Ok(()) => bail!("CI audit with a failed check was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("check did not pass"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_check_without_failure_class() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(
            dir.path().join("bundle-audit.json"),
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "profile": "oidc",
                "checks": [{
                    "name": "bundle-audit",
                    "status": "pass",
                    "detail": "simulated pass",
                }],
            }))?,
        )?;

        let err = match verify_ci_audit_json(
            &dir.path().join("bundle-audit.json"),
            "oidc",
            "CI recipe audit",
        ) {
            Ok(()) => bail!("CI audit check without failure_class was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("missing failure_class"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_without_metadata_boundaries() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(
            dir.path().join("bundle-audit.json"),
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "profile": "oidc",
                "checks": [{
                    "name": "bundle-audit",
                    "status": "pass",
                    "failure_class": "profile_validation_failed",
                    "detail": "simulated pass",
                }],
                "does_not_prove": ["production signing-key custody"],
            }))?,
        )?;

        let err = match verify_ci_audit_json(
            &dir.path().join("bundle-audit.json"),
            "oidc",
            "CI recipe audit",
        ) {
            Ok(()) => bail!("CI audit without boundaries was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("boundaries is not an array"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_boundary_without_payload_posture() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(
            dir.path().join("bundle-audit.json"),
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "profile": "oidc",
                "checks": [{
                    "name": "bundle-audit",
                    "status": "pass",
                    "failure_class": "profile_validation_failed",
                    "detail": "simulated pass",
                }],
                "boundaries": ["audit-bundle proves local bundle consistency only"],
                "does_not_prove": ["production signing-key custody"],
            }))?,
        )?;

        let err = match verify_ci_audit_json(
            &dir.path().join("bundle-audit.json"),
            "oidc",
            "CI recipe audit",
        ) {
            Ok(()) => bail!("CI audit without metadata boundary was accepted"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("boundaries missing entry containing `metadata only`")
        );
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_audit_without_production_non_claim() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(
            dir.path().join("bundle-audit.json"),
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "profile": "oidc",
                "checks": [{
                    "name": "bundle-audit",
                    "status": "pass",
                    "failure_class": "profile_validation_failed",
                    "detail": "simulated pass",
                }],
                "boundaries": [
                    "audit receipts contain metadata only and do not copy generated fixture payloads"
                ],
                "does_not_prove": ["downstream validator correctness"],
            }))?,
        )?;

        let err = match verify_ci_audit_json(
            &dir.path().join("bundle-audit.json"),
            "oidc",
            "CI recipe audit",
        ) {
            Ok(()) => bail!("CI audit without production non-claim was accepted"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("does_not_prove missing entry containing `production`")
        );
        Ok(())
    }

    #[test]
    fn external_adoption_accepts_ci_inspect_summary() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("inspect.txt");
        write_ci_inspect_summary(&path, "oidc")?;

        verify_ci_inspect_summary(&path, "oidc")
    }

    #[test]
    fn external_adoption_rejects_ci_inspect_wrong_profile() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("inspect.txt");
        write_ci_inspect_summary(&path, "webhook")?;

        let err = match verify_ci_inspect_summary(&path, "oidc") {
            Ok(()) => bail!("CI inspect summary with wrong profile was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("Bundle profile: oidc"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_inspect_without_verification() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("inspect.txt");
        fs::write(
            &path,
            concat!(
                "Bundle profile: oidc\n",
                "Summary type: quick human bundle summary\n",
                "Generated files:\n",
                "Artifact posture:\n",
            ),
        )?;

        let err = match verify_ci_inspect_summary(&path, "oidc") {
            Ok(()) => bail!("CI inspect summary without verification was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("Verification: ok"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_ci_inspect_with_secret_marker() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("inspect.txt");
        fs::write(
            &path,
            concat!(
                "Bundle profile: oidc\n",
                "Summary type: quick human bundle summary\n",
                "Verification: ok\n",
                "Generated files:\n",
                "Artifact posture:\n",
                "-----BEGIN PRIVATE KEY-----\n",
            ),
        )?;

        let err = match verify_ci_inspect_summary(&path, "oidc") {
            Ok(()) => bail!("CI inspect summary with secret marker was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("forbidden payload marker"));
        Ok(())
    }

    fn write_ci_audit_json(dir: &Path, profile: &str) -> Result<()> {
        fs::write(
            dir.join("bundle-audit.json"),
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "profile": profile,
                "checks": [{
                    "name": "bundle-audit",
                    "status": "pass",
                    "failure_class": "profile_validation_failed",
                    "detail": "simulated pass",
                }],
                "boundaries": [
                    "audit receipts contain metadata only and do not copy generated fixture payloads"
                ],
                "does_not_prove": [
                    "production signing-key custody"
                ],
            }))?,
        )?;
        Ok(())
    }

    fn write_ci_inspect_summary(path: &Path, profile: &str) -> Result<()> {
        fs::write(
            path,
            format!(
                concat!(
                    "Bundle profile: {}\n",
                    "Summary type: quick human bundle summary\n",
                    "Verification: ok\n",
                    "Generated files:\n",
                    "Artifact posture:\n",
                    "- jwks/valid.json scanner_safe=yes runtime_material=no\n",
                ),
                profile
            ),
        )?;
        Ok(())
    }

    fn write_ci_audit_markdown(dir: &Path, profile: &str) -> Result<()> {
        fs::write(
            dir.join("bundle-audit.md"),
            format!(
                concat!(
                    "# uselesskey Bundle Audit\n\n",
                    "- Status: pass\n",
                    "- Bundle: target/uselesskey-{}\n",
                    "- Profile: {}\n",
                    "- Receipt type: durable metadata-only reviewer/CI receipt\n",
                    "- Payload posture: raw generated fixture payloads are not copied into this receipt\n",
                    "\n## Checks\n\n",
                    "| Check | Status | Failure class | Detail |\n",
                    "|---|---|---|---|\n",
                    "| profile-validation | pass | profile_validation_failed | profile-specific generated files match the manifest |\n",
                    "\n## Boundaries\n\n",
                    "- audit receipts contain metadata only and do not copy generated fixture payloads\n",
                    "\n## Does Not Prove\n\n",
                    "- production signing-key custody\n",
                ),
                profile, profile
            ),
        )?;
        Ok(())
    }

    fn actions_matrix_profiles(example: &str) -> Vec<&str> {
        let line = example
            .lines()
            .find(|line| line.trim_start().starts_with("profile: ["))
            .expect("actions recipe has profile matrix");
        let (_, profiles) = line
            .split_once('[')
            .expect("matrix line has opening bracket");
        let (profiles, _) = profiles
            .split_once(']')
            .expect("matrix line has closing bracket");

        profiles.split(',').map(str::trim).collect()
    }

    #[test]
    fn external_adoption_verifies_doctor_json() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("doctor.json");
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "checks": [
                    {"name": "cli-version", "status": "pass"},
                    {"name": "known-profiles", "status": "pass"}
                ],
                "known_profiles": ["scanner-safe", "tls", "oidc", "webhook", "runtime"]
            }))?,
        )?;

        verify_doctor_json(&path)
    }

    #[test]
    fn external_adoption_rejects_failing_doctor_json() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("doctor.json");
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "status": "fail",
                "checks": [
                    {"name": "cli-version", "status": "pass"}
                ],
                "known_profiles": ["oidc"]
            }))?,
        )?;

        let err = match verify_doctor_json(&path) {
            Ok(()) => bail!("failing doctor json was accepted"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("doctor status mismatch"));
        Ok(())
    }

    #[test]
    fn external_adoption_rejects_doctor_json_missing_smoke_profile() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("doctor.json");
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "status": "pass",
                "checks": [
                    {"name": "cli-version", "status": "pass"},
                    {"name": "known-profiles", "status": "pass"}
                ],
                "known_profiles": ["scanner-safe", "oidc", "webhook", "runtime"]
            }))?,
        )?;

        let err = match verify_doctor_json(&path) {
            Ok(()) => bail!("doctor json missing a smoke profile was accepted"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("doctor known_profiles missing tls")
        );
        Ok(())
    }
}
