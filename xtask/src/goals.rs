use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const ACTIVE_GOAL_TOML: &str = ".uselesskey/goals/active.toml";
const GOAL_ARCHIVE_DIR: &str = ".uselesskey/goals/archive";
const DOC_ARTIFACTS_TOML: &str = "policy/doc-artifacts.toml";
const VALID_GOAL_STATUSES: &[&str] = &["active", "archived"];
const VALID_WORK_ITEM_STATUSES: &[&str] = &["planned", "ready", "active", "done", "blocked"];
const TOP_LEVEL_FIELDS: &[&str] = &[
    "schema_version",
    "id",
    "title",
    "status",
    "owner",
    "created",
    "objective",
    "end_state",
    "work_item",
];
const WORK_ITEM_FIELDS: &[&str] = &[
    "id",
    "status",
    "proposal",
    "spec",
    "plan",
    "commands",
    "blocked_by",
    "receipts",
];

#[derive(Debug, Deserialize)]
struct GoalManifest {
    id: String,
    title: String,
    status: String,
    owner: String,
    created: String,
    objective: String,
    #[serde(default)]
    end_state: Vec<String>,
    #[serde(default, rename = "work_item")]
    work_items: Vec<WorkItem>,
}

#[derive(Debug, Deserialize)]
struct WorkItem {
    id: String,
    status: String,
    #[serde(default)]
    proposal: Option<String>,
    #[serde(default)]
    spec: Option<String>,
    #[serde(default)]
    plan: Option<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    blocked_by: Option<toml::Value>,
    #[serde(default)]
    receipts: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DocArtifactLedger {
    #[serde(default)]
    artifact: Vec<DocArtifact>,
}

#[derive(Debug, Deserialize)]
struct DocArtifact {
    id: String,
    kind: String,
}

#[derive(Debug)]
struct GoalFile {
    rel_path: String,
    value: toml::Value,
    manifest: GoalManifest,
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let errors = validate(root)?;
    if errors.is_empty() {
        let files = read_goal_files(root)?;
        let work_items = files
            .iter()
            .map(|file| file.manifest.work_items.len())
            .sum::<usize>();
        println!(
            "goals: {} manifests; {} work items; goals ok",
            files.len(),
            work_items
        );
        Ok(())
    } else {
        for error in &errors {
            eprintln!("goals: {error}");
        }
        bail!("goals: {} validation error(s)", errors.len());
    }
}

fn validate(root: &Path) -> Result<Vec<String>> {
    let goal_files = read_goal_files(root)?;
    let artifact_index = read_artifact_index(root)?;
    let mut errors = Vec::new();

    if goal_files.is_empty() {
        errors.push(format!("{ACTIVE_GOAL_TOML}: no goal manifests found"));
    }

    for goal_file in &goal_files {
        validate_allowed_fields(goal_file, &mut errors);
        validate_manifest(root, goal_file, &artifact_index, &mut errors);
    }

    Ok(errors)
}

fn read_goal_files(root: &Path) -> Result<Vec<GoalFile>> {
    let mut paths = Vec::new();
    let active_path = root.join(ACTIVE_GOAL_TOML);
    if active_path.exists() {
        paths.push((ACTIVE_GOAL_TOML.to_string(), active_path));
    }

    let archive_dir = root.join(GOAL_ARCHIVE_DIR);
    if archive_dir.exists() {
        let mut archive_paths = fs::read_dir(&archive_dir)
            .with_context(|| format!("read {}", archive_dir.display()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect::<Vec<_>>();
        archive_paths.sort();
        for path in archive_paths {
            let rel = rel_path(root, &path);
            paths.push((rel, path));
        }
    }

    let mut files = Vec::new();
    for (rel_path, path) in paths {
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let value: toml::Value =
            toml::from_str(&text).with_context(|| format!("parse {rel_path}"))?;
        let manifest: GoalManifest =
            toml::from_str(&text).with_context(|| format!("parse {rel_path}"))?;
        files.push(GoalFile {
            rel_path,
            value,
            manifest,
        });
    }
    Ok(files)
}

fn read_artifact_index(root: &Path) -> Result<BTreeMap<String, String>> {
    let path = root.join(DOC_ARTIFACTS_TOML);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let ledger: DocArtifactLedger =
        toml::from_str(&text).with_context(|| format!("parse {DOC_ARTIFACTS_TOML}"))?;
    Ok(ledger
        .artifact
        .into_iter()
        .map(|artifact| (artifact.id, artifact.kind))
        .collect())
}

fn validate_allowed_fields(goal_file: &GoalFile, errors: &mut Vec<String>) {
    let Some(table) = goal_file.value.as_table() else {
        errors.push(format!(
            "{}: manifest must be a TOML table",
            goal_file.rel_path
        ));
        return;
    };

    for key in table.keys() {
        if !TOP_LEVEL_FIELDS.contains(&key.as_str()) {
            errors.push(format!(
                "{}: unsupported top-level field `{key}`",
                goal_file.rel_path
            ));
        }
    }

    let Some(items) = table.get("work_item").and_then(toml::Value::as_array) else {
        return;
    };
    for (idx, item) in items.iter().enumerate() {
        let Some(item_table) = item.as_table() else {
            errors.push(format!(
                "{}: work_item {} must be a TOML table",
                goal_file.rel_path,
                idx + 1
            ));
            continue;
        };
        let item_id = item_table
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        for key in item_table.keys() {
            if !WORK_ITEM_FIELDS.contains(&key.as_str()) {
                errors.push(format!(
                    "{}: work_item `{item_id}` has unsupported field `{key}`",
                    goal_file.rel_path
                ));
            }
        }
    }
}

fn validate_manifest(
    root: &Path,
    goal_file: &GoalFile,
    artifact_index: &BTreeMap<String, String>,
    errors: &mut Vec<String>,
) {
    let manifest = &goal_file.manifest;
    for (field, value) in [
        ("id", manifest.id.as_str()),
        ("title", manifest.title.as_str()),
        ("status", manifest.status.as_str()),
        ("owner", manifest.owner.as_str()),
        ("created", manifest.created.as_str()),
        ("objective", manifest.objective.as_str()),
    ] {
        if value.trim().is_empty() {
            errors.push(format!("{}: empty {field}", goal_file.rel_path));
        }
    }

    if !VALID_GOAL_STATUSES.contains(&manifest.status.as_str()) {
        errors.push(format!(
            "{}: invalid goal status `{}`",
            goal_file.rel_path, manifest.status
        ));
    }
    if manifest.end_state.is_empty() {
        errors.push(format!("{}: end_state is empty", goal_file.rel_path));
    }
    if manifest.work_items.is_empty() {
        errors.push(format!("{}: no work_item entries", goal_file.rel_path));
    }

    let mut seen_items = BTreeSet::new();
    for item in &manifest.work_items {
        validate_work_item(
            root,
            goal_file,
            item,
            artifact_index,
            &mut seen_items,
            errors,
        );
    }
}

fn validate_work_item(
    root: &Path,
    goal_file: &GoalFile,
    item: &WorkItem,
    artifact_index: &BTreeMap<String, String>,
    seen_items: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let label = format!("{}: work_item `{}`", goal_file.rel_path, item.id);
    if item.id.trim().is_empty() {
        errors.push(format!("{}: work_item with empty id", goal_file.rel_path));
    } else if !seen_items.insert(item.id.clone()) {
        errors.push(format!("{label} is duplicated"));
    }

    if !VALID_WORK_ITEM_STATUSES.contains(&item.status.as_str()) {
        errors.push(format!("{label} has invalid status `{}`", item.status));
    }

    validate_artifact_link(
        &label,
        "proposal",
        item.proposal.as_deref(),
        artifact_index,
        errors,
    );
    validate_artifact_link(&label, "spec", item.spec.as_deref(), artifact_index, errors);

    match item.plan.as_deref() {
        Some(plan) if !plan.trim().is_empty() => {
            let plan_path = root.join(plan.replace('/', std::path::MAIN_SEPARATOR_STR));
            if !plan_path.exists() {
                errors.push(format!("{label} links missing plan `{plan}`"));
            }
        }
        _ => errors.push(format!("{label} missing plan")),
    }

    match item.status.as_str() {
        "ready" | "active" => {
            if item.commands.is_empty() {
                errors.push(format!(
                    "{label} status `{}` requires commands",
                    item.status
                ));
            }
        }
        "done" => {
            if item.commands.is_empty() && item.receipts.is_empty() {
                errors.push(format!("{label} done item requires commands or receipts"));
            }
        }
        "blocked" => {
            if blocked_by_is_empty(item.blocked_by.as_ref()) {
                errors.push(format!("{label} blocked item requires blocked_by"));
            }
        }
        _ => {}
    }

    if item
        .commands
        .iter()
        .any(|command| command.trim().is_empty())
    {
        errors.push(format!("{label} has an empty command"));
    }
    if item
        .receipts
        .iter()
        .any(|receipt| receipt.trim().is_empty())
    {
        errors.push(format!("{label} has an empty receipt reference"));
    }
}

fn validate_artifact_link(
    label: &str,
    expected_kind: &str,
    linked_id: Option<&str>,
    artifact_index: &BTreeMap<String, String>,
    errors: &mut Vec<String>,
) {
    let Some(linked_id) = linked_id else {
        errors.push(format!("{label} missing {expected_kind}"));
        return;
    };
    let Some(kind) = artifact_index.get(linked_id) else {
        errors.push(format!(
            "{label} links missing {expected_kind} `{linked_id}`"
        ));
        return;
    };
    if kind != expected_kind {
        errors.push(format!(
            "{label} links `{linked_id}` as {expected_kind}, but it is {kind}"
        ));
    }
}

fn blocked_by_is_empty(value: Option<&toml::Value>) -> bool {
    match value {
        None => true,
        Some(toml::Value::String(text)) => text.trim().is_empty(),
        Some(toml::Value::Array(items)) => items.is_empty(),
        Some(_) => false,
    }
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_linked_spec() -> Result<()> {
        let dir = minimal_repo()?;
        write_active(
            dir.path(),
            work_item(
                "missing-spec",
                "ready",
                r#"proposal = "USELESSKEY-PROP-0001"
spec = "USELESSKEY-SPEC-9999"
plan = "plans/test/implementation-plan.md"
commands = ["cargo xtask check-goals"]
"#,
            ),
        )?;
        assert_error(dir.path(), "links missing spec `USELESSKEY-SPEC-9999`")
    }

    #[test]
    fn rejects_fake_human_merge_required() -> Result<()> {
        let dir = minimal_repo()?;
        write_active(
            dir.path(),
            work_item(
                "fake-field",
                "ready",
                r#"proposal = "USELESSKEY-PROP-0001"
spec = "USELESSKEY-SPEC-0001"
plan = "plans/test/implementation-plan.md"
commands = ["cargo xtask check-goals"]
requires_human_merge = true
"#,
            ),
        )?;
        assert_error(dir.path(), "unsupported field `requires_human_merge`")
    }

    #[test]
    fn rejects_done_without_proof() -> Result<()> {
        let dir = minimal_repo()?;
        write_active(
            dir.path(),
            work_item(
                "done-without-proof",
                "done",
                r#"proposal = "USELESSKEY-PROP-0001"
spec = "USELESSKEY-SPEC-0001"
plan = "plans/test/implementation-plan.md"
commands = []
"#,
            ),
        )?;
        assert_error(dir.path(), "done item requires commands or receipts")
    }

    #[test]
    fn accepts_blocked_with_blocked_by() -> Result<()> {
        let dir = minimal_repo()?;
        write_active(
            dir.path(),
            work_item(
                "blocked-with-reason",
                "blocked",
                r#"proposal = "USELESSKEY-PROP-0001"
spec = "USELESSKEY-SPEC-0001"
plan = "plans/test/implementation-plan.md"
blocked_by = "external credential"
"#,
            ),
        )?;
        let errors = validate(dir.path())?;
        assert!(errors.is_empty(), "errors: {errors:?}");
        Ok(())
    }

    #[test]
    fn accepts_ready_with_commands() -> Result<()> {
        let dir = minimal_repo()?;
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

    fn minimal_repo() -> Result<tempfile::TempDir> {
        let dir = tempfile::tempdir()?;
        write_file(
            dir.path(),
            DOC_ARTIFACTS_TOML,
            r#"schema_version = "1.0"
owner = "EffortlessMetrics"
updated = "2026-05-21"

[[artifact]]
id = "USELESSKEY-PROP-0001"
kind = "proposal"
path = "docs/proposals/prop.md"
status = "accepted"
owner = "repo-infra"

[[artifact]]
id = "USELESSKEY-SPEC-0001"
kind = "spec"
path = "docs/specs/spec.md"
status = "accepted"
owner = "repo-infra"
linked_proposal = "USELESSKEY-PROP-0001"
"#,
        )?;
        write_file(
            dir.path(),
            "plans/test/implementation-plan.md",
            "# Test plan\n",
        )?;
        write_active(
            dir.path(),
            work_item(
                "ready-with-commands",
                "ready",
                r#"proposal = "USELESSKEY-PROP-0001"
spec = "USELESSKEY-SPEC-0001"
plan = "plans/test/implementation-plan.md"
commands = ["cargo xtask check-goals"]
"#,
            ),
        )?;
        write_file(dir.path(), ".uselesskey/goals/archive/.gitkeep", "")?;
        Ok(dir)
    }

    fn write_active(root: &Path, work_item: String) -> Result<()> {
        write_file(
            root,
            ACTIVE_GOAL_TOML,
            &format!(
                r#"schema_version = "1.0"
id = "test-goal"
title = "Test goal"
status = "active"
owner = "codex"
created = "2026-05-21"
objective = "Test objective."
end_state = ["Done."]

{work_item}
"#
            ),
        )
    }

    fn work_item(id: &str, status: &str, body: &str) -> String {
        format!(
            r#"[[work_item]]
id = "{id}"
status = "{status}"
{body}
"#
        )
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
