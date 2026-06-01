use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const DOC_ARTIFACTS_TOML: &str = "policy/doc-artifacts.toml";
const VALID_KINDS: &[&str] = &["proposal", "spec", "adr", "plan", "policy", "status"];
const VALID_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "implemented",
    "superseded",
    "archived",
];

#[derive(Debug, Deserialize)]
struct DocArtifactLedger {
    schema_version: String,
    owner: String,
    updated: String,
    #[serde(default)]
    artifact: Vec<DocArtifact>,
}

#[derive(Debug, Deserialize)]
struct DocArtifact {
    id: String,
    kind: String,
    path: String,
    status: String,
    owner: String,
    #[serde(default)]
    linked_proposal: Option<String>,
    #[serde(default)]
    linked_specs: Vec<String>,
    #[serde(default)]
    linked_adrs: Vec<String>,
    #[serde(default)]
    linked_plan: Option<String>,
    #[serde(default)]
    standalone_reason: Option<String>,
    #[serde(default)]
    replaced_by: Option<String>,
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let errors = validate(root)?;
    if errors.is_empty() {
        let ledger = read_ledger(root)?;
        println!(
            "doc-artifacts: {} artifacts; ledger ok",
            ledger.artifact.len()
        );
        Ok(())
    } else {
        for error in &errors {
            eprintln!("doc-artifacts: {error}");
        }
        bail!("doc-artifacts: {} validation error(s)", errors.len());
    }
}

fn validate(root: &Path) -> Result<Vec<String>> {
    let ledger = read_ledger(root)?;
    let mut errors = Vec::new();

    if ledger.schema_version != "1.0" {
        errors.push(format!(
            "{DOC_ARTIFACTS_TOML}: schema_version must be `1.0`, got `{}`",
            ledger.schema_version
        ));
    }
    if ledger.owner.trim().is_empty() {
        errors.push(format!("{DOC_ARTIFACTS_TOML}: owner is required"));
    }
    if ledger.updated.trim().is_empty() {
        errors.push(format!("{DOC_ARTIFACTS_TOML}: updated is required"));
    }
    if ledger.artifact.is_empty() {
        errors.push(format!(
            "{DOC_ARTIFACTS_TOML}: at least one [[artifact]] entry is required"
        ));
    }

    let mut by_id = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for artifact in &ledger.artifact {
        if !seen.insert(artifact.id.clone()) {
            errors.push(format!(
                "{DOC_ARTIFACTS_TOML}: duplicate artifact id `{}`",
                artifact.id
            ));
        }
        by_id.insert(artifact.id.clone(), artifact);
    }

    for artifact in &ledger.artifact {
        validate_artifact(root, artifact, &by_id, &mut errors);
    }

    Ok(errors)
}

fn read_ledger(root: &Path) -> Result<DocArtifactLedger> {
    let path = root.join(DOC_ARTIFACTS_TOML);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse {DOC_ARTIFACTS_TOML}"))
}

fn validate_artifact(
    root: &Path,
    artifact: &DocArtifact,
    by_id: &BTreeMap<String, &DocArtifact>,
    errors: &mut Vec<String>,
) {
    let label = format!("{DOC_ARTIFACTS_TOML}: artifact `{}`", artifact.id);

    if artifact.id.trim().is_empty() {
        errors.push(format!("{DOC_ARTIFACTS_TOML}: artifact id is required"));
    }
    if artifact.owner.trim().is_empty() {
        errors.push(format!("{label} owner is required"));
    }
    if !VALID_KINDS.contains(&artifact.kind.as_str()) {
        errors.push(format!("{label} has invalid kind `{}`", artifact.kind));
    }
    if !VALID_STATUSES.contains(&artifact.status.as_str()) {
        errors.push(format!("{label} has invalid status `{}`", artifact.status));
    }
    let artifact_path_shape_ok = validate_path_shape(&label, "path", &artifact.path, errors);
    if artifact_path_shape_ok && !kind_matches_path(&artifact.kind, &artifact.path) {
        errors.push(format!(
            "{label} kind `{}` does not match path `{}`",
            artifact.kind, artifact.path
        ));
    }

    if artifact_path_shape_ok {
        let artifact_path = root.join(artifact.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        match fs::read_to_string(&artifact_path) {
            Ok(text) => {
                if !text.contains(&artifact.id) {
                    errors.push(format!("{label} id does not appear in `{}`", artifact.path));
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                errors.push(format!("{label} file `{}` does not exist", artifact.path));
            }
            Err(err) => {
                errors.push(format!(
                    "{label} file `{}` cannot be read: {err}",
                    artifact.path
                ));
            }
        }
    }

    if artifact.kind == "spec"
        && artifact.status == "accepted"
        && artifact
            .linked_proposal
            .as_ref()
            .is_none_or(|proposal| proposal.trim().is_empty())
        && artifact
            .standalone_reason
            .as_ref()
            .is_none_or(|reason| reason.trim().is_empty())
    {
        errors.push(format!(
            "{label} accepted spec requires linked_proposal or standalone_reason"
        ));
    }

    if artifact.status == "superseded"
        && artifact
            .replaced_by
            .as_ref()
            .is_none_or(|replacement| replacement.trim().is_empty())
    {
        errors.push(format!("{label} superseded artifact requires replaced_by"));
    }

    validate_id_link(
        &label,
        "proposal",
        artifact.linked_proposal.as_deref(),
        by_id,
        errors,
    );
    for spec in &artifact.linked_specs {
        validate_id_link(&label, "spec", Some(spec), by_id, errors);
    }
    for adr in &artifact.linked_adrs {
        validate_id_link(&label, "adr", Some(adr), by_id, errors);
    }
    if let Some(replaced_by) = artifact.replaced_by.as_deref() {
        validate_any_link(&label, replaced_by, by_id, errors);
    }
    if let Some(plan) = artifact.linked_plan.as_deref()
        && validate_path_shape(&label, "linked_plan", plan, errors)
    {
        if !plan.starts_with("plans/") || !plan.ends_with(".md") {
            errors.push(format!(
                "{label} linked_plan `{plan}` must start with `plans/` and end with `.md`"
            ));
        }
        let plan_path = root.join(plan.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !plan_path.exists() {
            errors.push(format!("{label} links missing plan `{plan}`"));
        }
    }
}

fn validate_path_shape(label: &str, field: &str, path: &str, errors: &mut Vec<String>) -> bool {
    let trimmed = path.trim();
    let mut valid = true;

    if trimmed.is_empty() {
        errors.push(format!("{label} {field} is empty"));
        return false;
    }
    if trimmed != path {
        errors.push(format!(
            "{label} {field} `{path}` has leading or trailing whitespace"
        ));
        valid = false;
    }
    if trimmed.contains('\\') {
        errors.push(format!(
            "{label} {field} `{trimmed}` must use `/` separators"
        ));
        valid = false;
    }
    if is_absolute_or_drive_path(trimmed) {
        errors.push(format!("{label} {field} `{trimmed}` must be relative"));
        valid = false;
    }
    if trimmed.split('/').any(str::is_empty) {
        errors.push(format!(
            "{label} {field} `{trimmed}` has an empty path component"
        ));
        valid = false;
    }
    if trimmed
        .split('/')
        .any(|component| matches!(component, "." | ".."))
    {
        errors.push(format!(
            "{label} {field} `{trimmed}` must not contain `.` or `..` path components"
        ));
        valid = false;
    }

    valid
}

fn is_absolute_or_drive_path(path: &str) -> bool {
    path.starts_with('/')
        || path.starts_with('\\')
        || path.as_bytes().get(1).is_some_and(|byte| *byte == b':')
}

fn validate_id_link(
    label: &str,
    expected_kind: &str,
    linked_id: Option<&str>,
    by_id: &BTreeMap<String, &DocArtifact>,
    errors: &mut Vec<String>,
) {
    let Some(linked_id) = linked_id else {
        return;
    };
    let Some(target) = by_id.get(linked_id) else {
        errors.push(format!(
            "{label} links missing {expected_kind} `{linked_id}`"
        ));
        return;
    };
    if target.kind != expected_kind {
        errors.push(format!(
            "{label} links `{linked_id}` as {expected_kind}, but it is {}",
            target.kind
        ));
    }
}

fn validate_any_link(
    label: &str,
    linked_id: &str,
    by_id: &BTreeMap<String, &DocArtifact>,
    errors: &mut Vec<String>,
) {
    if !by_id.contains_key(linked_id) {
        errors.push(format!("{label} links missing replacement `{linked_id}`"));
    }
}

fn kind_matches_path(kind: &str, path: &str) -> bool {
    match kind {
        "proposal" => path.starts_with("docs/proposals/") && path.ends_with(".md"),
        "spec" => path.starts_with("docs/specs/") && path.ends_with(".md"),
        "adr" => path.starts_with("docs/adr/") && path.ends_with(".md"),
        "plan" => path.starts_with("plans/") && path.ends_with(".md"),
        "policy" => path.starts_with("policy/") && path.ends_with(".toml"),
        "status" => path.starts_with("docs/status/") && path.ends_with(".md"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_artifact_id() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"

[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        assert_error(dir.path(), "duplicate artifact id `USELESSKEY-SPEC-0001`")
    }

    #[test]
    fn rejects_missing_artifact_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ledger(
            dir.path(),
            artifact("USELESSKEY-SPEC-0001", "spec", "docs/specs/missing.md"),
        )?;
        assert_error(dir.path(), "file `docs/specs/missing.md` does not exist")
    }

    #[test]
    fn rejects_artifact_path_with_parent_component() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/escaped.md", "USELESSKEY-SPEC-0001")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/../escaped.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        assert_error(
            dir.path(),
            "path `docs/specs/../escaped.md` must not contain `.` or `..` path components",
        )
    }

    #[test]
    fn rejects_artifact_path_with_backslashes() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs\\specs\\a.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        assert_error(
            dir.path(),
            "path `docs\\specs\\a.md` must use `/` separators",
        )
    }

    #[test]
    fn rejects_unknown_status() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "unknown"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        assert_error(dir.path(), "invalid status `unknown`")
    }

    #[test]
    fn rejects_missing_linked_proposal() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_ledger(
            dir.path(),
            artifact("USELESSKEY-SPEC-0001", "spec", "docs/specs/a.md"),
        )?;
        assert_error(
            dir.path(),
            "accepted spec requires linked_proposal or standalone_reason",
        )
    }

    #[test]
    fn rejects_missing_linked_plan() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
linked_plan = "plans/missing/implementation-plan.md"
"#,
        )?;
        assert_error(
            dir.path(),
            "links missing plan `plans/missing/implementation-plan.md`",
        )
    }

    #[test]
    fn rejects_linked_plan_with_parent_component() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_file(dir.path(), "plans/escaped.md", "# Escaped plan\n")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
linked_plan = "plans/test/../escaped.md"
"#,
        )?;
        assert_error(
            dir.path(),
            "linked_plan `plans/test/../escaped.md` must not contain `.` or `..` path components",
        )
    }

    #[test]
    fn rejects_linked_plan_outside_plans_tree() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_file(dir.path(), "docs/plan.md", "# Plan\n")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
linked_plan = "docs/plan.md"
"#,
        )?;
        assert_error(
            dir.path(),
            "linked_plan `docs/plan.md` must start with `plans/` and end with `.md`",
        )
    }

    #[test]
    fn rejects_linked_proposal_with_wrong_kind() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_file(dir.path(), "docs/specs/b.md", "USELESSKEY-SPEC-0002")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "accepted"
owner = "repo-infra"
linked_proposal = "USELESSKEY-SPEC-0002"

[[artifact]]
id = "USELESSKEY-SPEC-0002"
kind = "spec"
path = "docs/specs/b.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        assert_error(
            dir.path(),
            "links `USELESSKEY-SPEC-0002` as proposal, but it is spec",
        )
    }

    #[test]
    fn rejects_superseded_without_replacement() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "superseded"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        assert_error(dir.path(), "superseded artifact requires replaced_by")
    }

    #[test]
    fn accepts_standalone_reason() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/a.md", "USELESSKEY-SPEC-0001")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/a.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test fixture"
"#,
        )?;
        let errors = validate(dir.path())?;
        assert!(errors.is_empty(), "errors: {errors:?}");
        Ok(())
    }

    #[test]
    fn accepts_superseded_with_replacement() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_file(dir.path(), "docs/specs/old.md", "USELESSKEY-SPEC-0001")?;
        write_file(dir.path(), "docs/specs/new.md", "USELESSKEY-SPEC-0002")?;
        write_ledger(
            dir.path(),
            r#"
[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/old.md"
status = "superseded"
owner = "repo-infra"
standalone_reason = "test"
replaced_by = "USELESSKEY-SPEC-0002"

[[artifact]]
id = "USELESSKEY-SPEC-0002"
kind = "spec"
path = "docs/specs/new.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        let errors = validate(dir.path())?;
        assert!(errors.is_empty(), "errors: {errors:?}");
        Ok(())
    }

    fn assert_error(root: &Path, needle: &str) -> Result<()> {
        let errors = validate(root)?;
        assert!(
            errors.iter().any(|error| error.contains(needle)),
            "expected `{needle}` in {errors:?}"
        );
        Ok(())
    }

    fn artifact(id: &str, kind: &str, path: &str) -> String {
        format!(
            r#"
[[artifact]]
id = "{id}"
kind = "{kind}"
path = "{path}"
status = "accepted"
owner = "repo-infra"
"#
        )
    }

    fn write_ledger(root: &Path, body: impl AsRef<str>) -> Result<()> {
        let text = format!(
            r#"schema_version = "1.0"
owner = "EffortlessMetrics"
updated = "2026-05-21"
{}
"#,
            body.as_ref()
        );
        write_file(root, DOC_ARTIFACTS_TOML, &text)
    }

    fn write_file(root: &Path, rel: &str, content: &str) -> Result<()> {
        let path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
}
