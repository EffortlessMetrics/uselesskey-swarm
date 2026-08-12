use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const POLICY_PATH: &str = "policy/ci-checks.toml";
const STATUS_PATH: &str = "docs/status/ci-check-policy.md";
const WORKFLOW_PATH: &str = ".github/workflows/source-of-truth.yml";
const SOURCE_WORKFLOW: &str = "Source of Truth";
const SOURCE_CHECK: &str = "Source of Truth Advisory";
const ALLOWED_ROLES: &[&str] = &[
    "required",
    "triage-required",
    "advisory-by-default",
    "main-branch-proof",
    "conditional-route",
    "merge-blocker",
];

#[derive(Debug, Deserialize)]
struct Policy {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    updated: String,
    #[serde(default)]
    check: Vec<PolicyCheck>,
}

#[derive(Clone, Debug, Deserialize)]
struct PolicyCheck {
    #[serde(default)]
    name: String,
    #[serde(default)]
    workflow: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    operator_action: String,
    #[serde(default)]
    local_reproduction: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PolicyIdentity {
    name: String,
    role: String,
    workflow: String,
}

#[derive(Debug)]
struct MarkdownRow {
    line: usize,
    cells: Vec<String>,
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let errors = validate(root)?;
    if errors.is_empty() {
        println!("ci-check-policy: policy, status index, and Source of Truth commands synchronized");
        Ok(())
    } else {
        for error in &errors {
            eprintln!("ci-check-policy: {error}");
        }
        bail!("ci-check-policy: {} validation error(s)", errors.len());
    }
}

fn validate(root: &Path) -> Result<Vec<String>> {
    let policy_source = fs::read_to_string(root.join(POLICY_PATH))
        .with_context(|| format!("read {POLICY_PATH}"))?;
    let status = fs::read_to_string(root.join(STATUS_PATH))
        .with_context(|| format!("read {STATUS_PATH}"))?;
    let workflow = fs::read_to_string(root.join(WORKFLOW_PATH))
        .with_context(|| format!("read {WORKFLOW_PATH}"))?;
    validate_sources(root, &policy_source, &status, &workflow)
}

fn validate_sources(
    root: &Path,
    policy_source: &str,
    status: &str,
    workflow: &str,
) -> Result<Vec<String>> {
    let policy: Policy = toml::from_str(policy_source).context("parse CI check policy TOML")?;
    let role_rows = markdown_table(status, "Roles", 3)?;
    let current_rows = markdown_table(status, "Current Policy", 4)?;
    let mut errors = Vec::new();

    validate_policy(root, &policy, &mut errors);
    validate_markdown(&policy, &role_rows, &current_rows, &mut errors);
    validate_workflow(&policy, workflow, &mut errors);
    Ok(errors)
}

fn validate_policy(root: &Path, policy: &Policy, errors: &mut Vec<String>) {
    if policy.schema_version != "1.0" {
        errors.push(format!(
            "{POLICY_PATH}: schema_version must be `1.0`, found `{}`",
            policy.schema_version
        ));
    }
    for (field, value) in [("owner", &policy.owner), ("updated", &policy.updated)] {
        if value.trim().is_empty() {
            errors.push(format!("{POLICY_PATH}: `{field}` must be nonempty"));
        }
    }
    if policy.check.is_empty() {
        errors.push(format!("{POLICY_PATH}: no [[check]] entries found"));
    }

    let mut names = BTreeSet::new();
    for check in &policy.check {
        let owner = if check.name.trim().is_empty() {
            "<unnamed check>"
        } else {
            &check.name
        };
        for (field, value) in [
            ("name", &check.name),
            ("workflow", &check.workflow),
            ("role", &check.role),
            ("scope", &check.scope),
            ("operator_action", &check.operator_action),
        ] {
            if value.trim().is_empty() {
                errors.push(format!("{POLICY_PATH}: `{owner}` has empty `{field}`"));
            }
        }
        if !check.name.trim().is_empty() && !names.insert(check.name.as_str()) {
            errors.push(format!("{POLICY_PATH}: duplicate check name `{}`", check.name));
        }
        if !check.role.trim().is_empty() && !ALLOWED_ROLES.contains(&check.role.as_str()) {
            errors.push(format!("{POLICY_PATH}: `{owner}` has unknown role `{}`", check.role));
        }
        if check.local_reproduction.is_empty() {
            errors.push(format!("{POLICY_PATH}: `{owner}` has no local_reproduction commands"));
        }
        let mut commands = BTreeSet::new();
        for command in &check.local_reproduction {
            if command.trim().is_empty() {
                errors.push(format!("{POLICY_PATH}: `{owner}` has an empty local reproduction command"));
            } else {
                if !commands.insert(command.as_str()) {
                    errors.push(format!("{POLICY_PATH}: `{owner}` repeats local reproduction command `{command}`"));
                }
                if command.starts_with("cargo xtask ") || command.starts_with("cargo test ") {
                    crate::proof_commands::validate_repo_cargo_command(
                        root,
                        POLICY_PATH,
                        owner,
                        command,
                        "local reproduction",
                        errors,
                    );
                }
            }
        }
    }
}

fn validate_markdown(
    policy: &Policy,
    role_rows: &[MarkdownRow],
    current_rows: &[MarkdownRow],
    errors: &mut Vec<String>,
) {
    let mut roles = BTreeMap::<String, Vec<usize>>::new();
    for row in role_rows {
        let role = code_cell(&row.cells[0]);
        if role.is_empty() || row.cells[1].trim().is_empty() || row.cells[2].trim().is_empty() {
            errors.push(format!("{STATUS_PATH}:{}: Roles row has an empty required field", row.line));
        }
        roles.entry(role).or_default().push(row.line);
    }
    for role in policy.check.iter().map(|check| check.role.as_str()).collect::<BTreeSet<_>>() {
        match roles.get(role) {
            None => errors.push(format!("{STATUS_PATH}: role `{role}` used by policy is undocumented")),
            Some(lines) if lines.len() > 1 => errors.push(format!(
                "{STATUS_PATH}: role `{role}` is defined more than once at lines {}",
                join_lines(lines)
            )),
            Some(_) => {}
        }
    }
    for (role, lines) in &roles {
        if !policy.check.iter().any(|check| check.role == *role) {
            errors.push(format!(
                "{STATUS_PATH}:{}: role `{role}` is defined but unused by policy",
                lines[0]
            ));
        }
    }

    let policy_rows = policy
        .check
        .iter()
        .map(|check| PolicyIdentity {
            name: check.name.clone(),
            role: check.role.clone(),
            workflow: check.workflow.clone(),
        })
        .collect::<BTreeSet<_>>();
    let mut markdown_rows = BTreeMap::<PolicyIdentity, Vec<usize>>::new();
    let mut markdown_names = BTreeMap::<String, Vec<(PolicyIdentity, usize)>>::new();
    for row in current_rows {
        let identity = PolicyIdentity {
            name: code_cell(&row.cells[0]),
            role: code_cell(&row.cells[1]),
            workflow: code_cell(&row.cells[2]),
        };
        if identity.name.is_empty()
            || identity.role.is_empty()
            || identity.workflow.is_empty()
            || row.cells[3].trim().is_empty()
        {
            errors.push(format!("{STATUS_PATH}:{}: Current Policy row has an empty required field", row.line));
        }
        markdown_rows.entry(identity.clone()).or_default().push(row.line);
        markdown_names.entry(identity.name.clone()).or_default().push((identity, row.line));
    }
    for (identity, lines) in &markdown_rows {
        if lines.len() > 1 {
            errors.push(format!(
                "{STATUS_PATH}: duplicate Current Policy row `{}` at lines {}",
                identity.name,
                join_lines(lines)
            ));
        }
    }
    for expected in &policy_rows {
        if !markdown_rows.contains_key(expected) {
            if let Some(found) = markdown_names.get(&expected.name) {
                errors.push(format!(
                    "{STATUS_PATH}: Current Policy row `{}` mismatches policy role/workflow at line {}",
                    expected.name, found[0].1
                ));
            } else {
                errors.push(format!("{STATUS_PATH}: missing Current Policy row `{}`", expected.name));
            }
        }
    }
    for (actual, lines) in markdown_rows {
        if !policy_rows.contains(&actual) {
            errors.push(format!(
                "{STATUS_PATH}:{}: extra Current Policy row `{}`",
                lines[0], actual.name
            ));
        }
    }
}

fn validate_workflow(policy: &Policy, workflow: &str, errors: &mut Vec<String>) {
    let workflow_names = workflow
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            line.strip_prefix("name:")
                .map(|value| (idx + 1, unquote(value.trim()).to_owned()))
        })
        .collect::<Vec<_>>();
    if workflow_names.len() != 1 || workflow_names[0].1 != SOURCE_WORKFLOW {
        errors.push(format!(
            "{WORKFLOW_PATH}: expected exactly one top-level workflow name `{SOURCE_WORKFLOW}`"
        ));
    }

    let source_jobs = workflow
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            line.strip_prefix("    name:")
                .filter(|value| unquote(value.trim()) == SOURCE_CHECK)
                .map(|_| idx)
        })
        .collect::<Vec<_>>();
    if source_jobs.len() != 1 {
        errors.push(format!(
            "{WORKFLOW_PATH}: expected exactly one job named `{SOURCE_CHECK}`"
        ));
        return;
    }
    let start = source_jobs[0];
    let lines = workflow.lines().collect::<Vec<_>>();
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| line.starts_with("  ") && !line.starts_with("    ") && line.ends_with(':'))
        .map_or(lines.len(), |(idx, _)| idx);
    let mut run_commands = BTreeMap::<String, Vec<usize>>::new();
    for (idx, line) in lines.iter().enumerate().take(end).skip(start + 1) {
        let trimmed = line.trim();
        if let Some(command) = trimmed.strip_prefix("run:") {
            let command = unquote(command.trim());
            if command.is_empty() || matches!(command, "|" | ">" | "|-" | ">-") {
                errors.push(format!(
                    "{WORKFLOW_PATH}:{}: ambiguous multiline run command in governed job",
                    idx + 1
                ));
            } else {
                run_commands.entry(command.to_owned()).or_default().push(idx + 1);
            }
        }
    }

    let source_rows = policy
        .check
        .iter()
        .filter(|check| check.name == SOURCE_CHECK)
        .collect::<Vec<_>>();
    if source_rows.len() != 1 {
        errors.push(format!("{POLICY_PATH}: expected exactly one `{SOURCE_CHECK}` row"));
        return;
    }
    let expected = source_rows[0]
        .local_reproduction
        .iter()
        .filter(|command| command.starts_with("cargo xtask "))
        .cloned()
        .collect::<BTreeSet<_>>();
    for command in &expected {
        match run_commands.get(command) {
            None => errors.push(format!("{WORKFLOW_PATH}: governed command `{command}` is missing from `{SOURCE_CHECK}`")),
            Some(lines) if lines.len() > 1 => errors.push(format!(
                "{WORKFLOW_PATH}: governed command `{command}` appears more than once at lines {}",
                join_lines(lines)
            )),
            Some(_) => {}
        }
    }
    for (command, lines) in run_commands {
        if command.starts_with("cargo xtask ") && !expected.contains(&command) {
            errors.push(format!(
                "{WORKFLOW_PATH}:{}: extra cargo xtask command `{command}` is absent from `{SOURCE_CHECK}` policy",
                lines[0]
            ));
        }
    }
}

fn markdown_table(source: &str, heading: &str, width: usize) -> Result<Vec<MarkdownRow>> {
    let lines = source.lines().collect::<Vec<_>>();
    let marker = format!("## {heading}");
    let headings = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == marker)
        .map(|(idx, _)| idx)
        .collect::<Vec<_>>();
    if headings.len() != 1 {
        bail!("{STATUS_PATH}: expected exactly one `{marker}` heading");
    }
    let end = lines
        .iter()
        .enumerate()
        .skip(headings[0] + 1)
        .find(|(_, line)| line.starts_with("## "))
        .map_or(lines.len(), |(idx, _)| idx);
    let mut rows = Vec::new();
    for (idx, line) in lines.iter().enumerate().take(end).skip(headings[0] + 1) {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_owned())
            .collect::<Vec<_>>();
        if cells.len() != width {
            bail!("{STATUS_PATH}:{}: `{marker}` table must have {width} columns", idx + 1);
        }
        if cells.iter().all(|cell| cell.chars().all(|ch| ch == '-' || ch == ':' || ch.is_whitespace()))
            || cells[0] == "Role"
            || cells[0] == "Check"
        {
            continue;
        }
        rows.push(MarkdownRow { line: idx + 1, cells });
    }
    Ok(rows)
}

fn code_cell(value: &str) -> String {
    value.trim().trim_matches('`').trim().to_owned()
}

fn unquote(value: &str) -> &str {
    value.trim_matches(|ch| ch == '\'' || ch == '"')
}

fn join_lines(lines: &[usize]) -> String {
    lines.iter().map(usize::to_string).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::{Context, Result, ensure};
    use tempfile::tempdir;

    use super::validate_sources;

    const POLICY: &str = r#"schema_version = "1.0"
owner = "owner"
updated = "2026-08-12"

[[check]]
name = "Source of Truth Advisory"
workflow = "Source of Truth"
role = "triage-required"
scope = "contract"
operator_action = "inspect"
local_reproduction = ["cargo xtask check-doc-artifacts", "cargo xtask check-goals"]

[[check]]
name = "Result"
workflow = "Routed"
role = "required"
scope = "merge"
operator_action = "wait"
local_reproduction = ["cargo test -p xtask ci_check_policy"]
"#;

    const STATUS: &str = r#"# CI

## Roles

| Role | Meaning | Normal action |
| --- | --- | --- |
| `required` | merge | wait |
| `triage-required` | inspect | fix |

## Current Policy

| Check | Role | Workflow | Boundary |
| --- | --- | --- | --- |
| `Source of Truth Advisory` | `triage-required` | `Source of Truth` | contract |
| `Result` | `required` | `Routed` | merge |
"#;

    const WORKFLOW: &str = r#"name: Source of Truth
jobs:
  source-of-truth:
    name: Source of Truth Advisory
    steps:
      - name: hygiene
        run: ci/check-bare-self-hosted.sh
      - name: docs
        run: cargo xtask check-doc-artifacts
      - name: goals
        run: cargo xtask check-goals
"#;

    fn errors(policy: &str, status: &str, workflow: &str) -> Result<Vec<String>> {
        let dir = tempdir().context("create CI policy fixture root")?;
        fs::create_dir_all(dir.path().join("xtask/src")).context("create xtask fixture")?;
        fs::write(dir.path().join("xtask/Cargo.toml"), "[package]\nname='xtask'\nversion='0.0.0'\n").context("write xtask manifest fixture")?;
        validate_sources(dir.path(), policy, status, workflow)
    }

    fn require_error(errors: &[String], needle: &str) -> Result<()> {
        ensure!(errors.iter().any(|error| error.contains(needle)), "missing diagnostic `{needle}` in {errors:#?}");
        Ok(())
    }

    #[test]
    fn ci_check_policy_accepts_current_repository_contract() -> Result<()> {
        let errors = super::validate(&crate::workspace_root_path())?;
        ensure!(errors.is_empty(), "current repository contract failed: {errors:#?}");
        Ok(())
    }

    #[test]
    fn ci_check_policy_accepts_current_contract_reordering_and_hygiene_guard() -> Result<()> {
        ensure!(errors(POLICY, STATUS, WORKFLOW)?.is_empty());
        let (header, checks) = POLICY
            .split_once("[[check]]")
            .context("split policy fixture header")?;
        let (first, second) = checks
            .split_once("\n[[check]]\n")
            .context("split policy fixture checks")?;
        let reordered = format!("{header}[[check]]\n{second}\n[[check]]{first}");
        let status = STATUS.replace(
            "| `Source of Truth Advisory` | `triage-required` | `Source of Truth` | contract |\n| `Result` | `required` | `Routed` | merge |",
            "| `Result` | `required` | `Routed` | merge |\n| `Source of Truth Advisory` | `triage-required` | `Source of Truth` | contract |",
        );
        ensure!(errors(&reordered, &status, WORKFLOW)?.is_empty());
        Ok(())
    }

    #[test]
    fn ci_check_policy_rejects_invalid_policy_fields_and_commands() -> Result<()> {
        let cases = [
            (POLICY.replace("name = \"Result\"", "name = \"Source of Truth Advisory\""), "duplicate check name"),
            (POLICY.replace("role = \"required\"", "role = \"unknown\""), "unknown role"),
            (POLICY.replace("scope = \"merge\"", "scope = \"\""), "empty `scope`"),
            (POLICY.replace("local_reproduction = [\"cargo test -p xtask ci_check_policy\"]", "local_reproduction = []"), "no local_reproduction"),
            (POLICY.replace("local_reproduction = [\"cargo test -p xtask ci_check_policy\"]", "local_reproduction = [\"cargo test -p xtask ci_check_policy\", \"cargo test -p xtask ci_check_policy\"]"), "repeats local reproduction"),
            (POLICY.replace("cargo test -p xtask ci_check_policy", "cargo xtask unknown-command"), "unknown xtask"),
            (POLICY.replace("cargo test -p xtask ci_check_policy", ""), "empty local reproduction"),
        ];
        for (policy, needle) in cases {
            require_error(&errors(&policy, STATUS, WORKFLOW)?, needle)?;
        }
        Ok(())
    }

    #[test]
    fn ci_check_policy_rejects_markdown_membership_and_identity_drift() -> Result<()> {
        let cases = [
            (STATUS.replace("| `Result` | `required` | `Routed` | merge |\n", ""), "missing Current Policy row `Result`"),
            (STATUS.replace("| `Result` | `required` | `Routed` | merge |", "| `Extra` | `required` | `Routed` | merge |"), "extra Current Policy row `Extra`"),
            (STATUS.replace("| `Result` | `required` | `Routed` | merge |", "| `Result` | `triage-required` | `Routed` | merge |"), "mismatches policy"),
            (STATUS.replace("| `Result` | `required` | `Routed` | merge |", "| `Result` | `required` | `Other` | merge |"), "mismatches policy"),
            (STATUS.replace("| `Result` | `required` | `Routed` | merge |", "| `Result` | `required` | `Routed` | merge |\n| `Result` | `required` | `Routed` | merge |"), "duplicate Current Policy row"),
        ];
        for (status, needle) in cases {
            require_error(&errors(POLICY, &status, WORKFLOW)?, needle)?;
        }
        Ok(())
    }

    #[test]
    fn ci_check_policy_rejects_role_definition_drift() -> Result<()> {
        let cases = [
            (STATUS.replace("| `required` | merge | wait |\n", ""), "role `required` used by policy is undocumented"),
            (STATUS.replace("| `required` | merge | wait |", "| `required` | merge | wait |\n| `required` | again | wait |"), "defined more than once"),
            (STATUS.replace("| `required` | merge | wait |", "| `required` | merge | wait |\n| `unused` | extra | none |"), "defined but unused"),
        ];
        for (status, needle) in cases {
            require_error(&errors(POLICY, &status, WORKFLOW)?, needle)?;
        }
        Ok(())
    }

    #[test]
    fn ci_check_policy_rejects_source_workflow_contract_drift() -> Result<()> {
        let cases = [
            (POLICY.replace("Source of Truth Advisory", "Other Advisory"), WORKFLOW.to_owned(), "expected exactly one `Source of Truth Advisory` row"),
            (POLICY.to_owned(), WORKFLOW.replace("name: Source of Truth\n", "name: Other\n"), "top-level workflow name"),
            (POLICY.to_owned(), WORKFLOW.replace("name: Source of Truth Advisory", "name: Other Advisory"), "job named `Source of Truth Advisory`"),
            (POLICY.to_owned(), WORKFLOW.replace("        run: cargo xtask check-goals\n", ""), "governed command `cargo xtask check-goals` is missing"),
            (POLICY.to_owned(), WORKFLOW.replace("        run: cargo xtask check-goals", "        run: cargo xtask check-goals\n      - name: goals again\n        run: cargo xtask check-goals"), "appears more than once"),
            (POLICY.to_owned(), WORKFLOW.replace("        run: cargo xtask check-goals", "        run: cargo xtask check-goals\n      - name: support\n        run: cargo xtask check-support-tiers"), "extra cargo xtask command"),
        ];
        for (policy, workflow, needle) in cases {
            require_error(&errors(&policy, STATUS, &workflow)?, needle)?;
        }
        Ok(())
    }
}
