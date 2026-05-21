use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::{read_json_file, write_json_pretty};

const BUNDLE_MANIFEST_SCHEMA_JSON: &str = "docs/schemas/bundle-manifest.schema.json";
const NEGATIVE_COVERAGE_SCHEMA_JSON: &str = "docs/schemas/negative-coverage.schema.json";
const PROFILES: &[&str] = &["scanner-safe", "tls", "oidc", "webhook", "runtime"];

#[derive(Debug, Serialize)]
struct BundleSchemaCheckReport {
    schema_version: u32,
    profiles_checked: usize,
    schemas_checked: Vec<String>,
    profile_reports: Vec<BundleSchemaProfileReport>,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BundleSchemaProfileReport {
    profile: String,
    bundle_dir: String,
    manifest_path: String,
    negative_coverage_path: String,
    artifact_count: usize,
    receipt_count: usize,
    negative_count: usize,
}

pub(crate) fn check(out: &Path) -> Result<()> {
    prepare_output_dir(out)?;

    let manifest_schema: Value = read_json_file(Path::new(BUNDLE_MANIFEST_SCHEMA_JSON))?;
    let negative_schema: Value = read_json_file(Path::new(NEGATIVE_COVERAGE_SCHEMA_JSON))?;

    let mut profile_reports = Vec::new();
    let mut errors = Vec::new();
    for profile in PROFILES {
        let bundle_dir = out.join(profile).join("bundle");
        generate_bundle(profile, &bundle_dir)?;
        let manifest_path = bundle_dir.join("manifest.json");
        let negative_coverage_path = bundle_dir.join("receipts/negative-coverage.json");
        let manifest: Value = read_json_file(&manifest_path)?;
        let negative_coverage: Value = read_json_file(&negative_coverage_path)?;

        validate_bundle_manifest(&manifest_schema, &manifest, &mut errors);
        validate_negative_coverage(&negative_schema, &negative_coverage, &mut errors);
        validate_manifest_receipt_link(profile, &manifest, &negative_coverage, &mut errors);

        profile_reports.push(BundleSchemaProfileReport {
            profile: (*profile).to_string(),
            bundle_dir: normalize_report_path(&bundle_dir),
            manifest_path: normalize_report_path(&manifest_path),
            negative_coverage_path: normalize_report_path(&negative_coverage_path),
            artifact_count: array_len(manifest.get("artifacts")),
            receipt_count: array_len(manifest.get("receipts")),
            negative_count: negative_coverage
                .get("negative_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
        });
    }

    let report = BundleSchemaCheckReport {
        schema_version: 1,
        profiles_checked: profile_reports.len(),
        schemas_checked: vec![
            BUNDLE_MANIFEST_SCHEMA_JSON.to_string(),
            NEGATIVE_COVERAGE_SCHEMA_JSON.to_string(),
        ],
        profile_reports,
        errors,
    };

    write_json_pretty(&out.join("bundle-schema-check.json"), &report)?;
    fs::write(
        out.join("bundle-schema-check.md"),
        render_bundle_schema_check_markdown(&report),
    )
    .with_context(|| format!("write {}", out.join("bundle-schema-check.md").display()))?;

    eprintln!(
        "bundle-schema-check: {} profiles; {} errors",
        report.profiles_checked,
        report.errors.len()
    );
    if !report.errors.is_empty() {
        for error in report.errors.iter().take(20) {
            eprintln!("  bundle-schema-check error: {error}");
        }
        bail!(
            "bundle-schema-check: {} schema contract error(s)",
            report.errors.len()
        );
    }
    Ok(())
}

fn prepare_output_dir(out: &Path) -> Result<()> {
    fs::create_dir_all("target").context("create target directory")?;
    let out = out
        .canonicalize()
        .or_else(|_| {
            let parent = out.parent().unwrap_or_else(|| Path::new("."));
            let canonical_parent = parent.canonicalize()?;
            Ok::<_, std::io::Error>(canonical_parent.join(out.file_name().unwrap_or_default()))
        })
        .with_context(|| format!("resolve {}", out.display()))?;
    let target = Path::new("target")
        .canonicalize()
        .context("resolve target directory")?;
    if !out.starts_with(&target) {
        bail!(
            "bundle schema check output must stay under target/: {}",
            out.display()
        );
    }
    if out.exists() {
        fs::remove_dir_all(&out).with_context(|| format!("remove {}", out.display()))?;
    }
    fs::create_dir_all(&out).with_context(|| format!("create {}", out.display()))
}

fn generate_bundle(profile: &str, bundle_dir: &Path) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "-p",
        "uselesskey-cli",
        "--",
        "bundle",
        "--profile",
        profile,
        "--out",
    ]);
    cmd.arg(bundle_dir);
    run_quiet_command(&mut cmd).with_context(|| format!("generate {profile} bundle"))
}

fn run_quiet_command(cmd: &mut Command) -> Result<()> {
    eprintln!(" RUN {:?}", cmd);
    let status = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to spawn command")?;
    if !status.success() {
        bail!("command failed with status: {status}");
    }
    Ok(())
}

fn validate_bundle_manifest(schema: &Value, manifest: &Value, errors: &mut Vec<String>) {
    validate_required_fields(schema, "", manifest, "manifest.json", errors);
    validate_object_type(manifest, "manifest.json", errors);
    validate_positive_integer(manifest.get("version"), "manifest.json.version", errors);
    validate_enum(
        manifest.get("profile"),
        schema.pointer("/$defs/profile/enum"),
        "manifest.json.profile",
        errors,
    );
    validate_string(manifest.get("label"), "manifest.json.label", errors);
    validate_string(manifest.get("seed"), "manifest.json.seed", errors);
    validate_string(manifest.get("format"), "manifest.json.format", errors);

    let file_values = validate_array(manifest.get("files"), "manifest.json.files", errors);
    let mut seen_files = std::collections::BTreeSet::new();
    for (idx, value) in file_values.iter().enumerate() {
        let path = format!("manifest.json.files[{idx}]");
        validate_relative_path(value, &path, errors);
        if let Some(file) = value.as_str()
            && !seen_files.insert(file)
        {
            errors.push(format!("{path}: duplicate path `{file}`"));
        }
    }

    let artifacts = validate_array(manifest.get("artifacts"), "manifest.json.artifacts", errors);
    for (idx, artifact) in artifacts.iter().enumerate() {
        let path = format!("manifest.json.artifacts[{idx}]");
        validate_required_fields(schema, "/$defs/artifact", artifact, &path, errors);
        validate_object_type(artifact, &path, errors);
        validate_relative_path(
            artifact.get("path").unwrap_or(&Value::Null),
            &format!("{path}.path"),
            errors,
        );
        validate_string(artifact.get("kind"), &format!("{path}.kind"), errors);
        validate_string(artifact.get("format"), &format!("{path}.format"), errors);
        validate_enum(
            artifact.get("profile"),
            schema.pointer("/$defs/profile/enum"),
            &format!("{path}.profile"),
            errors,
        );
        validate_string_array(
            artifact.get("lanes"),
            &["scanner-safe", "runtime", "materialized"],
            &format!("{path}.lanes"),
            true,
            errors,
        );
        validate_bool(
            artifact.get("scanner_safe"),
            &format!("{path}.scanner_safe"),
            errors,
        );
        validate_string(
            artifact.get("description"),
            &format!("{path}.description"),
            errors,
        );
    }

    let receipts = validate_array(manifest.get("receipts"), "manifest.json.receipts", errors);
    for (idx, receipt) in receipts.iter().enumerate() {
        let path = format!("manifest.json.receipts[{idx}]");
        validate_required_fields(schema, "/$defs/receipt", receipt, &path, errors);
        validate_object_type(receipt, &path, errors);
        validate_relative_path(
            receipt.get("path").unwrap_or(&Value::Null),
            &format!("{path}.path"),
            errors,
        );
        validate_string(receipt.get("kind"), &format!("{path}.kind"), errors);
        validate_enum(
            receipt.get("profile"),
            schema.pointer("/$defs/profile/enum"),
            &format!("{path}.profile"),
            errors,
        );
        validate_string(
            receipt.get("description"),
            &format!("{path}.description"),
            errors,
        );
    }
}

fn validate_negative_coverage(schema: &Value, receipt: &Value, errors: &mut Vec<String>) {
    validate_required_fields(schema, "", receipt, "negative-coverage.json", errors);
    validate_object_type(receipt, "negative-coverage.json", errors);
    if receipt.get("receipt").and_then(Value::as_str) != Some("negative-coverage") {
        errors.push("negative-coverage.json.receipt: expected `negative-coverage`".to_string());
    }
    validate_positive_integer(
        receipt.get("version"),
        "negative-coverage.json.version",
        errors,
    );
    validate_enum(
        receipt.get("profile"),
        schema.pointer("/properties/profile/enum"),
        "negative-coverage.json.profile",
        errors,
    );
    validate_nonnegative_integer(
        receipt.get("negative_count"),
        "negative-coverage.json.negative_count",
        errors,
    );
    let coverage = validate_array(
        receipt.get("coverage"),
        "negative-coverage.json.coverage",
        errors,
    );
    if receipt.get("negative_count").and_then(Value::as_u64) != Some(coverage.len() as u64) {
        errors.push(format!(
            "negative-coverage.json.negative_count: expected {}, found {:?}",
            coverage.len(),
            receipt.get("negative_count")
        ));
    }
    for (idx, entry) in coverage.iter().enumerate() {
        let path = format!("negative-coverage.json.coverage[{idx}]");
        validate_required_fields(schema, "/$defs/coverage_entry", entry, &path, errors);
        validate_object_type(entry, &path, errors);
        validate_relative_path(
            entry.get("path").unwrap_or(&Value::Null),
            &format!("{path}.path"),
            errors,
        );
        validate_string(entry.get("kind"), &format!("{path}.kind"), errors);
        validate_stable_id(
            entry.get("failure_class"),
            &format!("{path}.failure_class"),
            errors,
        );
        validate_string(
            entry.get("expected_failure"),
            &format!("{path}.expected_failure"),
            errors,
        );
        validate_bool(
            entry.get("scanner_safe"),
            &format!("{path}.scanner_safe"),
            errors,
        );
        validate_bool(
            entry.get("runtime_material"),
            &format!("{path}.runtime_material"),
            errors,
        );
        validate_string(
            entry.get("description"),
            &format!("{path}.description"),
            errors,
        );
    }
    validate_string_array(
        receipt.get("boundaries"),
        &[],
        "negative-coverage.json.boundaries",
        true,
        errors,
    );
}

fn validate_manifest_receipt_link(
    expected_profile: &str,
    manifest: &Value,
    negative_coverage: &Value,
    errors: &mut Vec<String>,
) {
    if manifest.get("profile").and_then(Value::as_str) != Some(expected_profile) {
        errors.push(format!(
            "manifest.json.profile: expected `{expected_profile}`, found {:?}",
            manifest.get("profile")
        ));
    }
    if negative_coverage.get("profile").and_then(Value::as_str) != Some(expected_profile) {
        errors.push(format!(
            "negative-coverage.json.profile: expected `{expected_profile}`, found {:?}",
            negative_coverage.get("profile")
        ));
    }
    let receipts = manifest
        .get("receipts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let has_negative_coverage = receipts.iter().any(|receipt| {
        receipt.get("path").and_then(Value::as_str) == Some("receipts/negative-coverage.json")
            && receipt.get("kind").and_then(Value::as_str) == Some("negative-coverage")
    });
    if !has_negative_coverage {
        errors.push(
            "manifest.json.receipts: missing receipts/negative-coverage.json linkage".to_string(),
        );
    }
}

fn validate_required_fields(
    schema: &Value,
    pointer: &str,
    value: &Value,
    path: &str,
    errors: &mut Vec<String>,
) {
    let required = schema
        .pointer(pointer)
        .unwrap_or(schema)
        .get("required")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for field in required {
        let Some(field) = field.as_str() else {
            continue;
        };
        if !value
            .as_object()
            .is_some_and(|object| object.contains_key(field))
        {
            errors.push(format!("{path}: missing required field `{field}`"));
        }
    }
}

fn validate_object_type(value: &Value, path: &str, errors: &mut Vec<String>) {
    if !value.is_object() {
        errors.push(format!("{path}: expected object"));
    }
}

fn validate_string(value: Option<&Value>, path: &str, errors: &mut Vec<String>) {
    if value.and_then(Value::as_str).is_none_or(str::is_empty) {
        errors.push(format!("{path}: expected non-empty string"));
    }
}

fn validate_positive_integer(value: Option<&Value>, path: &str, errors: &mut Vec<String>) {
    if value.and_then(Value::as_u64).is_none_or(|value| value < 1) {
        errors.push(format!("{path}: expected positive integer"));
    }
}

fn validate_nonnegative_integer(value: Option<&Value>, path: &str, errors: &mut Vec<String>) {
    if value.and_then(Value::as_u64).is_none() {
        errors.push(format!("{path}: expected non-negative integer"));
    }
}

fn validate_bool(value: Option<&Value>, path: &str, errors: &mut Vec<String>) {
    if !value.is_some_and(Value::is_boolean) {
        errors.push(format!("{path}: expected boolean"));
    }
}

fn validate_array<'a>(
    value: Option<&'a Value>,
    path: &str,
    errors: &mut Vec<String>,
) -> Vec<&'a Value> {
    match value.and_then(Value::as_array) {
        Some(values) => values.iter().collect(),
        None => {
            errors.push(format!("{path}: expected array"));
            Vec::new()
        }
    }
}

fn validate_string_array(
    value: Option<&Value>,
    allowed_values: &[&str],
    path: &str,
    require_non_empty: bool,
    errors: &mut Vec<String>,
) {
    let Some(values) = value.and_then(Value::as_array) else {
        errors.push(format!("{path}: expected array"));
        return;
    };
    if require_non_empty && values.is_empty() {
        errors.push(format!("{path}: expected at least one item"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (idx, value) in values.iter().enumerate() {
        let item_path = format!("{path}[{idx}]");
        let Some(value) = value.as_str().filter(|value| !value.is_empty()) else {
            errors.push(format!("{item_path}: expected non-empty string"));
            continue;
        };
        if !allowed_values.is_empty() && !allowed_values.contains(&value) {
            errors.push(format!("{item_path}: unsupported value `{value}`"));
        }
        if !seen.insert(value) {
            errors.push(format!("{path}: duplicate value `{value}`"));
        }
    }
}

fn validate_enum(
    value: Option<&Value>,
    enum_values: Option<&Value>,
    path: &str,
    errors: &mut Vec<String>,
) {
    let Some(value) = value.and_then(Value::as_str) else {
        errors.push(format!("{path}: expected enum string"));
        return;
    };
    let allowed = enum_values
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !allowed
        .iter()
        .any(|allowed| allowed.as_str() == Some(value))
    {
        errors.push(format!("{path}: unsupported enum value `{value}`"));
    }
}

fn validate_relative_path(value: &Value, path: &str, errors: &mut Vec<String>) {
    let Some(path_value) = value.as_str().filter(|value| !value.is_empty()) else {
        errors.push(format!("{path}: expected non-empty relative path"));
        return;
    };
    if !is_safe_relative_path(path_value) {
        errors.push(format!("{path}: unsafe relative path `{path_value}`"));
    }
}

fn validate_stable_id(value: Option<&Value>, path: &str, errors: &mut Vec<String>) {
    let Some(value) = value.and_then(Value::as_str) else {
        errors.push(format!("{path}: expected stable ID string"));
        return;
    };
    if !is_stable_id(value) {
        errors.push(format!("{path}: invalid stable ID `{value}`"));
    }
}

fn is_safe_relative_path(path: &str) -> bool {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.as_bytes().get(1) == Some(&b':')
    {
        return false;
    }
    path.replace('\\', "/")
        .split('/')
        .all(|component| !component.is_empty() && component != "..")
}

fn is_stable_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|ch| matches!(ch, 'a'..='z' | '0'..='9' | '_'))
        && !value.ends_with('_')
        && !value.contains("__")
}

fn array_len(value: Option<&Value>) -> usize {
    value.and_then(Value::as_array).map_or(0, Vec::len)
}

fn normalize_report_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn render_bundle_schema_check_markdown(report: &BundleSchemaCheckReport) -> String {
    let mut out = String::new();
    out.push_str("# Bundle schema check\n\n");
    out.push_str(&format!(
        "- Profiles checked: {}\n",
        report.profiles_checked
    ));
    out.push_str(&format!("- Errors: {}\n\n", report.errors.len()));
    out.push_str("## Schemas\n\n");
    for schema in &report.schemas_checked {
        out.push_str(&format!("- `{schema}`\n"));
    }
    out.push_str("\n## Profiles\n\n");
    out.push_str("| Profile | Artifacts | Receipts | Negative classes |\n");
    out.push_str("| --- | ---: | ---: | ---: |\n");
    for profile in &report.profile_reports {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            profile.profile, profile.artifact_count, profile.receipt_count, profile.negative_count
        ));
    }
    if !report.errors.is_empty() {
        out.push_str("\n## Errors\n\n");
        for error in &report.errors {
            out.push_str(&format!("- {error}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn safe_relative_path_rejects_absolute_and_parent_paths() {
        assert!(is_safe_relative_path("receipts/negative-coverage.json"));
        assert!(!is_safe_relative_path("../secret.pem"));
        assert!(!is_safe_relative_path("/tmp/secret.pem"));
        assert!(!is_safe_relative_path("C:/tmp/secret.pem"));
    }

    #[test]
    fn negative_coverage_validator_rejects_count_drift() {
        let schema = json!({
            "required": ["receipt", "version", "profile", "negative_count", "coverage", "boundaries"],
            "properties": {
                "profile": { "enum": ["scanner-safe"] }
            },
            "$defs": {
                "coverage_entry": {
                    "required": [
                        "path",
                        "kind",
                        "failure_class",
                        "expected_failure",
                        "scanner_safe",
                        "runtime_material",
                        "description"
                    ]
                }
            }
        });
        let receipt = json!({
            "receipt": "negative-coverage",
            "version": 1,
            "profile": "scanner-safe",
            "negative_count": 2,
            "coverage": [{
                "path": "token.json",
                "kind": "token",
                "failure_class": "token_near_miss",
                "expected_failure": "policy rejects token shape",
                "scanner_safe": true,
                "runtime_material": false,
                "description": "scanner-safe token near miss"
            }],
            "boundaries": ["metadata-only"]
        });
        let mut errors = Vec::new();
        validate_negative_coverage(&schema, &receipt, &mut errors);
        assert!(errors.iter().any(|error| error.contains("negative_count")));
    }
}
