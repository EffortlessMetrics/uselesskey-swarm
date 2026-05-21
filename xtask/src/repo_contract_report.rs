use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const ACTIVE_GOAL_TOML: &str = ".uselesskey/goals/active.toml";
const CLAIM_LEDGER_TOML: &str = "policy/claim-ledger.toml";
const DOC_ARTIFACTS_TOML: &str = "policy/doc-artifacts.toml";
const DEFAULT_OUT_DIR: &str = "target/source-of-truth";

#[derive(Debug, Deserialize)]
struct DocArtifactLedger {
    #[serde(default)]
    artifact: Vec<DocArtifact>,
}

#[derive(Debug, Deserialize)]
struct DocArtifact {
    id: String,
    kind: String,
    path: String,
    status: String,
    #[serde(default)]
    linked_proposal: Option<String>,
    #[serde(default)]
    linked_specs: Vec<String>,
    #[serde(default)]
    linked_adrs: Vec<String>,
    #[serde(default)]
    linked_plan: Option<String>,
    #[serde(default)]
    replaced_by: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoalManifest {
    id: String,
    title: String,
    status: String,
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
}

#[derive(Debug, Deserialize)]
struct ClaimLedger {
    #[serde(default)]
    claim: Vec<ClaimEntry>,
}

#[derive(Debug, Deserialize)]
struct ClaimEntry {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    spec: String,
    #[serde(default)]
    surfaces: Vec<String>,
    #[serde(default)]
    proof_commands: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: &'static str,
    generated_by: &'static str,
    active_goal: Option<GoalManifestSummary>,
    goal_manifest: GoalManifestSummary,
    ready_work_items: Vec<WorkItemSummary>,
    accepted_proposals: Vec<ArtifactSummary>,
    accepted_specs: Vec<ArtifactSummary>,
    open_adrs: Vec<ArtifactSummary>,
    support_tier_impacts: Vec<ClaimSummary>,
    policy_impacts: Vec<ArtifactSummary>,
    missing_links: Vec<String>,
    superseded_artifacts: Vec<ArtifactSummary>,
    recently_completed_work: Vec<WorkItemSummary>,
}

#[derive(Debug, Serialize)]
struct GoalManifestSummary {
    id: String,
    title: String,
    status: String,
    path: &'static str,
}

#[derive(Debug, Serialize)]
struct WorkItemSummary {
    id: String,
    status: String,
    proposal: Option<String>,
    spec: Option<String>,
    plan: Option<String>,
    commands: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ArtifactSummary {
    id: String,
    kind: String,
    status: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct ClaimSummary {
    id: String,
    title: String,
    status: String,
    spec: String,
    surfaces: Vec<String>,
    proof_commands: Vec<String>,
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let out_dir = root.join(DEFAULT_OUT_DIR);
    write_report(root, &out_dir)?;
    println!(
        "repo-contract-report: wrote {}/graph.md and {}/graph.json",
        DEFAULT_OUT_DIR, DEFAULT_OUT_DIR
    );
    Ok(())
}

fn write_report(root: &Path, out_dir: &Path) -> Result<Report> {
    let report = build_report(root)?;
    fs::create_dir_all(out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    fs::write(out_dir.join("graph.md"), render_markdown(&report))
        .with_context(|| format!("write {}", out_dir.join("graph.md").display()))?;
    fs::write(
        out_dir.join("graph.json"),
        serde_json::to_string_pretty(&report)?,
    )
    .with_context(|| format!("write {}", out_dir.join("graph.json").display()))?;
    Ok(report)
}

fn build_report(root: &Path) -> Result<Report> {
    let artifacts = read_doc_artifacts(root)?;
    let goal = read_active_goal(root)?;
    let claims = read_claim_ledger(root)?;
    let artifact_index = artifacts
        .artifact
        .iter()
        .map(|artifact| (artifact.id.as_str(), artifact))
        .collect::<BTreeMap<_, _>>();
    let mut missing_links = missing_artifact_links(root, &artifacts, &artifact_index);
    missing_links.extend(missing_goal_links(root, &goal, &artifact_index));
    missing_links.extend(missing_claim_links(&claims, &artifact_index));

    Ok(Report {
        schema_version: "1.0",
        generated_by: "cargo xtask repo-contract-report",
        active_goal: is_active_goal(&goal).then(|| summarize_goal_manifest(&goal)),
        goal_manifest: summarize_goal_manifest(&goal),
        ready_work_items: if is_active_goal(&goal) {
            summarize_work_items(&goal, "ready")
        } else {
            Vec::new()
        },
        accepted_proposals: summarize_artifacts(&artifacts, "proposal", "accepted"),
        accepted_specs: summarize_artifacts(&artifacts, "spec", "accepted"),
        open_adrs: artifacts
            .artifact
            .iter()
            .filter(|artifact| artifact.kind == "adr")
            .filter(|artifact| artifact.status != "archived" && artifact.status != "superseded")
            .map(summarize_artifact)
            .collect(),
        support_tier_impacts: claims
            .claim
            .iter()
            .filter(|claim| !claim.status.trim().is_empty())
            .map(summarize_claim)
            .collect(),
        policy_impacts: artifacts
            .artifact
            .iter()
            .filter(|artifact| artifact.kind == "policy")
            .map(summarize_artifact)
            .collect(),
        missing_links,
        superseded_artifacts: artifacts
            .artifact
            .iter()
            .filter(|artifact| artifact.status == "superseded")
            .map(summarize_artifact)
            .collect(),
        recently_completed_work: goal
            .work_items
            .iter()
            .filter(|item| item.status == "done")
            .rev()
            .take(8)
            .map(summarize_work_item)
            .collect(),
    })
}

fn summarize_goal_manifest(goal: &GoalManifest) -> GoalManifestSummary {
    GoalManifestSummary {
        id: goal.id.clone(),
        title: goal.title.clone(),
        status: goal.status.clone(),
        path: ACTIVE_GOAL_TOML,
    }
}

fn is_active_goal(goal: &GoalManifest) -> bool {
    goal.status == "active"
}

fn read_doc_artifacts(root: &Path) -> Result<DocArtifactLedger> {
    read_toml(root, DOC_ARTIFACTS_TOML)
}

fn read_active_goal(root: &Path) -> Result<GoalManifest> {
    read_toml(root, ACTIVE_GOAL_TOML)
}

fn read_claim_ledger(root: &Path) -> Result<ClaimLedger> {
    read_toml(root, CLAIM_LEDGER_TOML)
}

fn read_toml<T>(root: &Path, rel: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let path = root.join(to_path(rel));
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse {rel}"))
}

fn missing_artifact_links(
    root: &Path,
    artifacts: &DocArtifactLedger,
    artifact_index: &BTreeMap<&str, &DocArtifact>,
) -> Vec<String> {
    let mut missing = BTreeSet::new();
    for artifact in &artifacts.artifact {
        if let Some(proposal) = artifact.linked_proposal.as_deref()
            && !artifact_index.contains_key(proposal)
        {
            missing.insert(format!(
                "{} links missing proposal `{proposal}`",
                artifact.id
            ));
        }
        for spec in &artifact.linked_specs {
            if !artifact_index.contains_key(spec.as_str()) {
                missing.insert(format!("{} links missing spec `{spec}`", artifact.id));
            }
        }
        for adr in &artifact.linked_adrs {
            if !artifact_index.contains_key(adr.as_str()) {
                missing.insert(format!("{} links missing ADR `{adr}`", artifact.id));
            }
        }
        if let Some(replaced_by) = artifact.replaced_by.as_deref()
            && !artifact_index.contains_key(replaced_by)
        {
            missing.insert(format!(
                "{} links missing replacement `{replaced_by}`",
                artifact.id
            ));
        }
        if let Some(plan) = artifact.linked_plan.as_deref()
            && !root.join(to_path(plan)).exists()
        {
            missing.insert(format!("{} links missing plan `{plan}`", artifact.id));
        }
    }
    missing.into_iter().collect()
}

fn missing_goal_links(
    root: &Path,
    goal: &GoalManifest,
    artifact_index: &BTreeMap<&str, &DocArtifact>,
) -> Vec<String> {
    let mut missing = BTreeSet::new();
    for item in &goal.work_items {
        if let Some(proposal) = item.proposal.as_deref()
            && !artifact_index.contains_key(proposal)
        {
            missing.insert(format!(
                "work item `{}` links missing proposal `{proposal}`",
                item.id
            ));
        }
        if let Some(spec) = item.spec.as_deref()
            && !artifact_index.contains_key(spec)
        {
            missing.insert(format!(
                "work item `{}` links missing spec `{spec}`",
                item.id
            ));
        }
        if let Some(plan) = item.plan.as_deref()
            && !root.join(to_path(plan)).exists()
        {
            missing.insert(format!(
                "work item `{}` links missing plan `{plan}`",
                item.id
            ));
        }
    }
    missing.into_iter().collect()
}

fn missing_claim_links(
    claims: &ClaimLedger,
    artifact_index: &BTreeMap<&str, &DocArtifact>,
) -> Vec<String> {
    let mut missing = BTreeSet::new();
    for claim in &claims.claim {
        if !claim.spec.trim().is_empty() && !artifact_index.contains_key(claim.spec.as_str()) {
            missing.insert(format!(
                "claim `{}` links missing spec `{}`",
                claim.id, claim.spec
            ));
        }
    }
    missing.into_iter().collect()
}

fn summarize_artifacts(
    artifacts: &DocArtifactLedger,
    kind: &str,
    status: &str,
) -> Vec<ArtifactSummary> {
    artifacts
        .artifact
        .iter()
        .filter(|artifact| artifact.kind == kind && artifact.status == status)
        .map(summarize_artifact)
        .collect()
}

fn summarize_artifact(artifact: &DocArtifact) -> ArtifactSummary {
    ArtifactSummary {
        id: artifact.id.clone(),
        kind: artifact.kind.clone(),
        status: artifact.status.clone(),
        path: artifact.path.clone(),
    }
}

fn summarize_work_items(goal: &GoalManifest, status: &str) -> Vec<WorkItemSummary> {
    goal.work_items
        .iter()
        .filter(|item| item.status == status)
        .map(summarize_work_item)
        .collect()
}

fn summarize_work_item(item: &WorkItem) -> WorkItemSummary {
    WorkItemSummary {
        id: item.id.clone(),
        status: item.status.clone(),
        proposal: item.proposal.clone(),
        spec: item.spec.clone(),
        plan: item.plan.clone(),
        commands: item.commands.clone(),
    }
}

fn summarize_claim(claim: &ClaimEntry) -> ClaimSummary {
    ClaimSummary {
        id: claim.id.clone(),
        title: claim.title.clone(),
        status: claim.status.clone(),
        spec: claim.spec.clone(),
        surfaces: claim.surfaces.clone(),
        proof_commands: claim.proof_commands.clone(),
    }
}

fn render_markdown(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("# Source-of-Truth Graph\n\n");
    out.push_str("Generated by `cargo xtask repo-contract-report`.\n\n");
    out.push_str("## Active Goal\n\n");
    if let Some(active_goal) = &report.active_goal {
        out.push_str("| ID | Title | Status | Path |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        out.push_str(&format!(
            "| `{}` | {} | `{}` | `{}` |\n\n",
            active_goal.id,
            escape_md(&active_goal.title),
            active_goal.status,
            active_goal.path
        ));
    } else {
        out.push_str("None.\n\n");
        out.push_str(&format!(
            "Last goal manifest: `{}` is `{}` at `{}`.\n\n",
            report.goal_manifest.id, report.goal_manifest.status, report.goal_manifest.path
        ));
    }

    render_work_items(&mut out, "Ready Work Items", &report.ready_work_items);
    render_artifacts(&mut out, "Accepted Proposals", &report.accepted_proposals);
    render_artifacts(&mut out, "Accepted Specs", &report.accepted_specs);
    render_artifacts(&mut out, "Open ADRs", &report.open_adrs);
    render_claims(
        &mut out,
        "Support-Tier Impacts",
        &report.support_tier_impacts,
    );
    render_artifacts(&mut out, "Policy Impacts", &report.policy_impacts);
    render_list(&mut out, "Missing Links", &report.missing_links);
    render_artifacts(
        &mut out,
        "Superseded Artifacts",
        &report.superseded_artifacts,
    );
    render_work_items(
        &mut out,
        "Recently Completed Work",
        &report.recently_completed_work,
    );

    out
}

fn render_work_items(out: &mut String, title: &str, items: &[WorkItemSummary]) {
    out.push_str(&format!("## {title}\n\n"));
    if items.is_empty() {
        out.push_str("None.\n\n");
        return;
    }
    out.push_str("| ID | Status | Proposal | Spec | Plan | Proof commands |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for item in items {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | {} |\n",
            item.id,
            item.status,
            optional_code(item.proposal.as_deref()),
            optional_code(item.spec.as_deref()),
            optional_code(item.plan.as_deref()),
            inline_commands(&item.commands)
        ));
    }
    out.push('\n');
}

fn render_artifacts(out: &mut String, title: &str, artifacts: &[ArtifactSummary]) {
    out.push_str(&format!("## {title}\n\n"));
    if artifacts.is_empty() {
        out.push_str("None.\n\n");
        return;
    }
    out.push_str("| ID | Kind | Status | Path |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for artifact in artifacts {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` |\n",
            artifact.id, artifact.kind, artifact.status, artifact.path
        ));
    }
    out.push('\n');
}

fn render_claims(out: &mut String, title: &str, claims: &[ClaimSummary]) {
    out.push_str(&format!("## {title}\n\n"));
    if claims.is_empty() {
        out.push_str("None.\n\n");
        return;
    }
    out.push_str("| Claim | Status | Spec | Surfaces | Proof commands |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for claim in claims {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} |\n",
            claim.id,
            claim.status,
            claim.spec,
            inline_values(&claim.surfaces),
            inline_commands(&claim.proof_commands)
        ));
    }
    out.push('\n');
}

fn render_list(out: &mut String, title: &str, items: &[String]) {
    out.push_str(&format!("## {title}\n\n"));
    if items.is_empty() {
        out.push_str("None.\n\n");
        return;
    }
    for item in items {
        out.push_str(&format!("- {}\n", escape_md(item)));
    }
    out.push('\n');
}

fn optional_code(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("`{value}`"))
        .unwrap_or_else(|| "none".to_string())
}

fn inline_commands(commands: &[String]) -> String {
    if commands.is_empty() {
        return "none".to_string();
    }
    inline_values(commands)
}

fn inline_values(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("`{}`", escape_md(value)))
        .collect::<Vec<_>>()
        .join("; ")
}

fn escape_md(value: &str) -> String {
    value.replace('|', "\\|")
}

fn to_path(rel: &str) -> PathBuf {
    PathBuf::from(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_contract_report_writes_markdown_and_json() -> Result<()> {
        let dir = minimal_repo()?;
        let out_dir = dir.path().join("target/source-of-truth");

        let report = write_report(dir.path(), &out_dir)?;

        let active_goal = report
            .active_goal
            .as_ref()
            .context("expected report to include an active goal")?;
        assert_eq!(active_goal.id, "test-goal");
        assert_eq!(report.goal_manifest.status, "active");
        assert_eq!(report.ready_work_items.len(), 1);
        assert!(out_dir.join("graph.md").exists());
        assert!(out_dir.join("graph.json").exists());

        let markdown = fs::read_to_string(out_dir.join("graph.md"))?;
        assert!(markdown.contains("## Ready Work Items"));
        assert!(markdown.contains("`repo-contract-report`"));

        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out_dir.join("graph.json"))?)?;
        assert_eq!(json["schema_version"], "1.0");
        assert_eq!(json["active_goal"]["id"], "test-goal");
        assert_eq!(json["goal_manifest"]["status"], "active");
        assert_eq!(json["ready_work_items"][0]["id"], "repo-contract-report");
        Ok(())
    }

    #[test]
    fn repo_contract_report_does_not_treat_archived_manifest_as_active() -> Result<()> {
        let dir = minimal_repo()?;
        let out_dir = dir.path().join("target/source-of-truth");
        write_goal(
            dir.path(),
            "archived",
            r#"[[work_item]]
id = "previously-ready"
status = "ready"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-0023"
plan = "plans/source-of-truth-control-plane/implementation-plan.md"
commands = ["cargo xtask repo-contract-report"]
"#,
        )?;

        let report = write_report(dir.path(), &out_dir)?;

        assert!(report.active_goal.is_none());
        assert_eq!(report.goal_manifest.status, "archived");
        assert!(report.ready_work_items.is_empty());

        let markdown = fs::read_to_string(out_dir.join("graph.md"))?;
        assert!(markdown.contains("## Active Goal"));
        assert!(markdown.contains("None."));
        assert!(markdown.contains("Last goal manifest: `test-goal` is `archived`"));
        assert!(!markdown.contains("| `test-goal` | Test goal | `archived`"));

        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out_dir.join("graph.json"))?)?;
        assert!(json["active_goal"].is_null());
        assert_eq!(json["goal_manifest"]["status"], "archived");
        let ready_work_items = json["ready_work_items"]
            .as_array()
            .context("expected ready_work_items to be an array")?;
        assert_eq!(ready_work_items.len(), 0);
        Ok(())
    }

    #[test]
    fn repo_contract_report_records_missing_links() -> Result<()> {
        let dir = minimal_repo()?;
        write_active(
            dir.path(),
            r#"[[work_item]]
id = "missing-spec"
status = "ready"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-9999"
plan = "plans/source-of-truth-control-plane/implementation-plan.md"
commands = ["cargo xtask repo-contract-report"]
"#,
        )?;

        let report = build_report(dir.path())?;

        assert!(
            report
                .missing_links
                .iter()
                .any(|link| link.contains("USELESSKEY-SPEC-9999")),
            "missing links: {:?}",
            report.missing_links
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
id = "USELESSKEY-PROP-0002"
kind = "proposal"
path = "docs/proposals/prop.md"
status = "accepted"

[[artifact]]
id = "USELESSKEY-SPEC-0023"
kind = "spec"
path = "docs/specs/spec.md"
status = "accepted"
linked_proposal = "USELESSKEY-PROP-0002"

[[artifact]]
id = "USELESSKEY-ADR-0003"
kind = "adr"
path = "docs/adr/adr.md"
status = "accepted"
linked_proposal = "USELESSKEY-PROP-0002"
linked_specs = ["USELESSKEY-SPEC-0023"]

[[artifact]]
id = "USELESSKEY-POLICY-claim-ledger"
kind = "policy"
path = "policy/claim-ledger.toml"
status = "accepted"
linked_specs = ["USELESSKEY-SPEC-0023"]
"#,
        )?;
        write_file(
            dir.path(),
            "policy/claim-ledger.toml",
            r#"schema_version = "1.0"

[[claim]]
id = "metadata-only-audit-packets"
title = "Metadata-only audit packets"
status = "advisory"
spec = "USELESSKEY-SPEC-0023"
surfaces = ["uselesskey audit-bundle --ci"]
proof_commands = ["cargo xtask no-blob"]
"#,
        )?;
        write_file(
            dir.path(),
            "plans/source-of-truth-control-plane/implementation-plan.md",
            "# Plan\n",
        )?;
        write_active(
            dir.path(),
            r#"[[work_item]]
id = "repo-contract-report"
status = "ready"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-0023"
plan = "plans/source-of-truth-control-plane/implementation-plan.md"
commands = ["cargo xtask repo-contract-report"]

[[work_item]]
id = "advisory-source-of-truth-ci"
status = "done"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-0023"
plan = "plans/source-of-truth-control-plane/implementation-plan.md"
commands = ["cargo xtask check-goals"]
"#,
        )?;
        Ok(dir)
    }

    fn write_active(root: &Path, work_items: &str) -> Result<()> {
        write_goal(root, "active", work_items)
    }

    fn write_goal(root: &Path, status: &str, work_items: &str) -> Result<()> {
        write_file(
            root,
            ACTIVE_GOAL_TOML,
            &format!(
                r#"schema_version = "1.0"
id = "test-goal"
title = "Test goal"
status = "{status}"
owner = "codex"
created = "2026-05-21"
objective = "Test objective."
end_state = ["Done."]

{work_items}
"#
            ),
        )
    }

    fn write_file(root: &Path, rel: &str, content: &str) -> Result<()> {
        let path = root.join(to_path(rel));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
}
