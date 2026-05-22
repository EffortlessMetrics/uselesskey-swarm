use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde::Deserialize;

const ACTIVE_GOAL_TOML: &str = ".uselesskey/goals/active.toml";
const CLAIM_LEDGER_TOML: &str = "policy/claim-ledger.toml";
const DOC_ARTIFACTS_TOML: &str = "policy/doc-artifacts.toml";
const HANDOFF_DIR: &str = "docs/handoffs";
const GOAL_ARCHIVE_DIR: &str = ".uselesskey/goals/archive";

#[derive(Debug, Deserialize)]
struct GoalManifest {
    id: String,
    title: String,
    status: String,
    owner: String,
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
    path: String,
    #[serde(default)]
    linked_adrs: Vec<String>,
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

#[derive(Debug, Default)]
struct SpecContext {
    linked_adrs: Vec<String>,
    support_tier_impact: Vec<String>,
    policy_impact: Vec<String>,
    claim_boundary: Option<String>,
    rollback: Option<String>,
}

#[derive(Debug)]
struct CloseoutContext {
    date: String,
    goal_id: String,
    goal_title: String,
    goal_status: String,
    goal_owner: String,
    linked_proposal: Option<String>,
    linked_specs: Vec<String>,
    linked_adrs: Vec<String>,
    linked_plan: Option<String>,
    support_tier_impact: Vec<String>,
    policy_impact: Vec<String>,
    done_items: Vec<WorkItemSummary>,
    remaining_items: Vec<WorkItemSummary>,
    proof_commands: Vec<String>,
    receipts: Vec<String>,
    related_claims: Vec<ClaimSummary>,
    claim_boundary: Option<String>,
    rollback: Option<String>,
    closeout_rel: String,
    archive_rel: String,
}

#[derive(Debug)]
struct WorkItemSummary {
    id: String,
    status: String,
    proposal: Option<String>,
    spec: Option<String>,
    plan: Option<String>,
    commands: Vec<String>,
}

#[derive(Debug)]
struct ClaimSummary {
    id: String,
    title: String,
    status: String,
    spec: String,
    surfaces: Vec<String>,
    proof_commands: Vec<String>,
}

#[derive(Debug)]
struct CloseoutOutputs {
    closeout_rel: String,
    archive_rel: String,
}

pub(crate) fn run(root: &Path, goal_id: &str) -> Result<()> {
    let date = Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let outputs = write_closeout(root, goal_id, &date)?;
    println!(
        "closeout: wrote {} and {}",
        outputs.closeout_rel, outputs.archive_rel
    );
    Ok(())
}

fn write_closeout(root: &Path, goal_id: &str, date: &str) -> Result<CloseoutOutputs> {
    let active_text = read_text(root, ACTIVE_GOAL_TOML)?;
    let active_value: toml::Value =
        toml::from_str(&active_text).with_context(|| format!("parse {ACTIVE_GOAL_TOML}"))?;
    let goal: GoalManifest =
        toml::from_str(&active_text).with_context(|| format!("parse {ACTIVE_GOAL_TOML}"))?;
    if goal.id != goal_id {
        bail!(
            "active goal `{}` does not match requested goal `{goal_id}`",
            goal.id
        );
    }
    if goal.status != "active" {
        bail!(
            "{ACTIVE_GOAL_TOML} has status `{}`; closeout requires status `active`",
            goal.status
        );
    }

    let slug = slugify(goal_id);
    let closeout_rel = format!("{HANDOFF_DIR}/{date}-{slug}-closeout.md");
    let archive_rel = format!("{GOAL_ARCHIVE_DIR}/{date}-{slug}.toml");
    let context = build_context(root, &goal, date, closeout_rel, archive_rel)?;
    let closeout = render_closeout(&context);
    let archive = render_archive(active_value)?;

    write_new_file(root, &context.closeout_rel, &closeout)?;
    write_new_file(root, &context.archive_rel, &archive)?;

    Ok(CloseoutOutputs {
        closeout_rel: context.closeout_rel,
        archive_rel: context.archive_rel,
    })
}

fn build_context(
    root: &Path,
    goal: &GoalManifest,
    date: &str,
    closeout_rel: String,
    archive_rel: String,
) -> Result<CloseoutContext> {
    let artifacts = read_artifact_index(root)?;
    let claims: ClaimLedger = read_toml(root, CLAIM_LEDGER_TOML)?;
    let linked_proposal = first_unique(
        goal.work_items
            .iter()
            .filter_map(|item| item.proposal.clone()),
    );
    let linked_specs = unique(goal.work_items.iter().filter_map(|item| item.spec.clone()));
    let linked_plan = first_unique(goal.work_items.iter().filter_map(|item| item.plan.clone()));
    let mut spec_context = SpecContext::default();

    for spec_id in &linked_specs {
        let Some(artifact) = artifacts.get(spec_id) else {
            continue;
        };
        spec_context
            .linked_adrs
            .extend(artifact.linked_adrs.iter().cloned());
        let spec_text = read_text(root, &artifact.path)?;
        let front_matter = parse_front_matter(&spec_text);
        spec_context.linked_adrs.extend(front_matter.linked_adrs);
        spec_context
            .support_tier_impact
            .extend(front_matter.support_tier_impact);
        spec_context
            .policy_impact
            .extend(front_matter.policy_impact);
        if spec_context.claim_boundary.is_none() {
            spec_context.claim_boundary = markdown_section(&spec_text, "Claim Boundary");
        }
        if spec_context.rollback.is_none() {
            spec_context.rollback = markdown_section(&spec_text, "Rollback");
        }
    }

    let linked_adrs = unique(spec_context.linked_adrs);
    let support_tier_impact = unique(spec_context.support_tier_impact);
    let policy_impact = unique(spec_context.policy_impact);
    let done_items = goal
        .work_items
        .iter()
        .filter(|item| item.status == "done")
        .map(summarize_work_item)
        .collect::<Vec<_>>();
    let remaining_items = goal
        .work_items
        .iter()
        .filter(|item| item.status != "done")
        .map(summarize_work_item)
        .collect::<Vec<_>>();
    let proof_commands = unique(
        done_items
            .iter()
            .flat_map(|item| item.commands.iter().cloned()),
    );
    let mut receipts = unique(
        goal.work_items
            .iter()
            .flat_map(|item| item.receipts.iter().cloned()),
    );
    receipts.push(closeout_rel.clone());
    receipts.push(archive_rel.clone());
    receipts = unique(receipts);
    let related_claims = claims
        .claim
        .into_iter()
        .filter(|claim| linked_specs.iter().any(|spec| spec == &claim.spec))
        .map(|claim| ClaimSummary {
            id: claim.id,
            title: claim.title,
            status: claim.status,
            spec: claim.spec,
            surfaces: claim.surfaces,
            proof_commands: claim.proof_commands,
        })
        .collect();

    Ok(CloseoutContext {
        date: date.to_string(),
        goal_id: goal.id.clone(),
        goal_title: goal.title.clone(),
        goal_status: goal.status.clone(),
        goal_owner: goal.owner.clone(),
        linked_proposal,
        linked_specs,
        linked_adrs,
        linked_plan,
        support_tier_impact,
        policy_impact,
        done_items,
        remaining_items,
        proof_commands,
        receipts,
        related_claims,
        claim_boundary: spec_context.claim_boundary,
        rollback: spec_context.rollback,
        closeout_rel,
        archive_rel,
    })
}

fn render_closeout(context: &CloseoutContext) -> String {
    let title = format!("{} closeout", context.goal_title);
    let mut out = String::new();
    out.push_str("+++\n");
    out.push_str(&format!(
        "id = {}\n",
        toml_quote(&format!(
            "USELESSKEY-HANDOFF-{}-{}",
            context.date,
            slugify(&context.goal_id)
        ))
    ));
    out.push_str("kind = \"closeout\"\n");
    out.push_str(&format!("title = {}\n", toml_quote(&title)));
    out.push_str("status = \"implemented\"\n");
    out.push_str(&format!("owner = {}\n", toml_quote(&context.goal_owner)));
    out.push_str(&format!("created = {}\n", toml_quote(&context.date)));
    out.push_str(&format!(
        "linked_proposal = {}\n",
        optional_toml_string(context.linked_proposal.as_deref())
    ));
    out.push_str(&format!(
        "linked_specs = {}\n",
        toml_array(&context.linked_specs)
    ));
    out.push_str(&format!(
        "linked_adrs = {}\n",
        toml_array(&context.linked_adrs)
    ));
    out.push_str(&format!(
        "linked_plan = {}\n",
        optional_toml_string(context.linked_plan.as_deref())
    ));
    out.push_str(&format!(
        "support_tier_impact = {}\n",
        toml_array(&context.support_tier_impact)
    ));
    out.push_str(&format!(
        "policy_impact = {}\n",
        toml_array(&context.policy_impact)
    ));
    out.push_str("linked_prs = []\n");
    out.push_str("+++\n\n");
    out.push_str(&format!("# {}\n\n", title_case(&title)));
    out.push_str(&format!(
        "Generated by `cargo xtask closeout --goal {}`.\n\n",
        context.goal_id
    ));
    out.push_str(&format!(
        "Active goal status at generation time: `{}`.\n\n",
        context.goal_status
    ));

    render_landed_work(&mut out, &context.done_items);
    render_required_evidence(&mut out, &context.proof_commands);
    render_receipts(&mut out, &context.receipts);
    render_support_tier_impact(&mut out, context);
    render_policy_impact(&mut out, context);
    render_claim_boundary(&mut out, context.claim_boundary.as_deref());
    render_remaining_work(&mut out, &context.remaining_items);
    render_known_risks(&mut out);
    render_rollback(&mut out, context.rollback.as_deref());
    render_next_safe_action(&mut out, &context.remaining_items);

    out
}

fn render_landed_work(out: &mut String, items: &[WorkItemSummary]) {
    out.push_str("## Landed Work\n\n");
    if items.is_empty() {
        out.push_str("None recorded.\n\n");
        return;
    }
    out.push_str("| Work item | Spec | Plan |\n");
    out.push_str("| --- | --- | --- |\n");
    for item in items {
        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            item.id,
            optional_code(item.spec.as_deref()),
            optional_code(item.plan.as_deref())
        ));
    }
    out.push('\n');
}

fn render_required_evidence(out: &mut String, commands: &[String]) {
    out.push_str("## Required Evidence\n\n");
    if commands.is_empty() {
        out.push_str("No proof commands recorded on completed work items.\n\n");
        return;
    }
    out.push_str("```bash\n");
    for command in commands {
        out.push_str(command);
        out.push('\n');
    }
    out.push_str("```\n\n");
}

fn render_receipts(out: &mut String, receipts: &[String]) {
    out.push_str("## Receipts\n\n");
    for receipt in receipts {
        out.push_str(&format!("- `{}`\n", escape_md(receipt)));
    }
    out.push('\n');
}

fn render_support_tier_impact(out: &mut String, context: &CloseoutContext) {
    out.push_str("## Support-Tier Impact\n\n");
    if context.support_tier_impact.is_empty() && context.related_claims.is_empty() {
        out.push_str("None recorded.\n\n");
        return;
    }
    for impact in &context.support_tier_impact {
        out.push_str(&format!("- `{}`\n", escape_md(impact)));
    }
    if !context.related_claims.is_empty() {
        out.push_str("\nRelated claims:\n");
        for claim in &context.related_claims {
            out.push_str(&format!(
                "- `{}` (`{}`) via `{}`",
                escape_md(&claim.id),
                escape_md(&claim.status),
                escape_md(&claim.spec)
            ));
            if !claim.title.trim().is_empty() {
                out.push_str(&format!(" - {}", escape_md(&claim.title)));
            }
            if !claim.surfaces.is_empty() {
                out.push_str(&format!("; surfaces: {}", inline_values(&claim.surfaces)));
            }
            if !claim.proof_commands.is_empty() {
                out.push_str(&format!(
                    "; proof: {}",
                    inline_values(&claim.proof_commands)
                ));
            }
            out.push('\n');
        }
    }
    out.push('\n');
}

fn render_policy_impact(out: &mut String, context: &CloseoutContext) {
    out.push_str("## Policy Impact\n\n");
    if context.policy_impact.is_empty() {
        out.push_str("None recorded.\n\n");
        return;
    }
    for impact in &context.policy_impact {
        out.push_str(&format!("- `{}`\n", escape_md(impact)));
    }
    out.push('\n');
}

fn render_claim_boundary(out: &mut String, boundary: Option<&str>) {
    out.push_str("## Claim Boundary\n\n");
    if let Some(boundary) = boundary {
        out.push_str(boundary.trim());
        out.push_str("\n\n");
    } else {
        out.push_str("This closeout records repository control-plane state and command-backed evidence. It does not prove release readiness, source-sync readiness, provider compatibility, downstream verifier correctness, production security, or scanner evasion.\n\n");
    }
}

fn render_remaining_work(out: &mut String, items: &[WorkItemSummary]) {
    out.push_str("## Remaining Work\n\n");
    if items.is_empty() {
        out.push_str("None recorded in this active goal.\n\n");
        return;
    }
    out.push_str("| Work item | Status | Proposal | Spec | Plan |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for item in items {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} |\n",
            item.id,
            item.status,
            optional_code(item.proposal.as_deref()),
            optional_code(item.spec.as_deref()),
            optional_code(item.plan.as_deref())
        ));
    }
    out.push('\n');
}

fn render_known_risks(out: &mut String) {
    out.push_str("## Known Risks\n\n");
    out.push_str("- Generated closeouts are summaries of repo manifests and ledgers; they do not replace hosted CI evidence.\n");
    out.push_str("- Archive copies preserve the active goal at generation time; follow-up work must choose or update the next active goal explicitly.\n");
    out.push_str("- Release, publish, signing, tags, GitHub releases, and public source sync remain outside the swarm boundary.\n\n");
}

fn render_rollback(out: &mut String, rollback: Option<&str>) {
    out.push_str("## Rollback\n\n");
    if let Some(rollback) = rollback {
        out.push_str(rollback.trim());
        out.push_str("\n\n");
    } else {
        out.push_str("Revert the generator PR or remove the generated closeout and archive files before relying on them as handoff state.\n\n");
    }
}

fn render_next_safe_action(out: &mut String, items: &[WorkItemSummary]) {
    out.push_str("## Next Safe Action\n\n");
    if let Some(item) = items.iter().find(|item| item.status == "ready") {
        out.push_str(&format!(
            "Continue with ready work item `{}` from `{}`.",
            item.id,
            item.plan.as_deref().unwrap_or("the active goal")
        ));
    } else if let Some(item) = items.first() {
        out.push_str(&format!(
            "Resolve or advance `{}` (`{}`) before closing the goal.",
            item.id, item.status
        ));
    } else {
        out.push_str("Create or select the next active goal before starting new work.");
    }
    out.push('\n');
}

fn render_archive(mut value: toml::Value) -> Result<String> {
    let Some(table) = value.as_table_mut() else {
        bail!("{ACTIVE_GOAL_TOML} must be a TOML table");
    };
    table.insert(
        "status".to_string(),
        toml::Value::String("archived".to_string()),
    );
    let mut text = toml::to_string_pretty(&value)?;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    Ok(text)
}

fn read_artifact_index(root: &Path) -> Result<BTreeMap<String, DocArtifact>> {
    let ledger: DocArtifactLedger = read_toml(root, DOC_ARTIFACTS_TOML)?;
    Ok(ledger
        .artifact
        .into_iter()
        .map(|artifact| (artifact.id.clone(), artifact))
        .collect())
}

fn read_toml<T>(root: &Path, rel: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let text = read_text(root, rel)?;
    toml::from_str(&text).with_context(|| format!("parse {rel}"))
}

fn read_text(root: &Path, rel: &str) -> Result<String> {
    let path = root.join(to_path(rel));
    fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))
}

fn write_new_file(root: &Path, rel: &str, content: &str) -> Result<()> {
    let path = root.join(to_path(rel));
    if path.exists() {
        bail!("refusing to overwrite {}", path.display());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, content).with_context(|| format!("write {}", path.display()))
}

fn parse_front_matter(markdown: &str) -> SpecContext {
    let Some(rest) = markdown.strip_prefix("+++\n") else {
        return SpecContext::default();
    };
    let Some((front_matter, _body)) = rest.split_once("\n+++") else {
        return SpecContext::default();
    };
    let Ok(value) = toml::from_str::<toml::Value>(front_matter) else {
        return SpecContext::default();
    };
    SpecContext {
        linked_adrs: string_array(&value, "linked_adrs"),
        support_tier_impact: string_array(&value, "support_tier_impact"),
        policy_impact: string_array(&value, "policy_impact"),
        claim_boundary: None,
        rollback: None,
    }
}

fn string_array(value: &toml::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn markdown_section(markdown: &str, title: &str) -> Option<String> {
    let heading = format!("## {title}");
    let mut lines = markdown.lines();
    for line in lines.by_ref() {
        if line.trim() == heading {
            break;
        }
    }
    let section = lines
        .take_while(|line| !line.starts_with("## "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();
    if section.is_empty() {
        None
    } else {
        Some(section)
    }
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

fn unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn first_unique(values: impl IntoIterator<Item = String>) -> Option<String> {
    unique(values).into_iter().next()
}

fn optional_code(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("`{}`", escape_md(value)))
        .unwrap_or_else(|| "none".to_string())
}

fn inline_values(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values
        .iter()
        .map(|value| format!("`{}`", escape_md(value)))
        .collect::<Vec<_>>()
        .join("; ")
}

fn toml_array(values: &[String]) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }
    let body = values
        .iter()
        .map(|value| format!("  {},", toml_quote(value)))
        .collect::<Vec<_>>()
        .join("\n");
    format!("[\n{body}\n]")
}

fn optional_toml_string(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .map(toml_quote)
        .unwrap_or_else(|| "\"\"".to_string())
}

fn toml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
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
    fn closeout_writes_markdown_and_archived_goal() -> Result<()> {
        let dir = minimal_repo()?;

        let outputs = write_closeout(dir.path(), "test-goal", "2026-05-21")?;

        assert_eq!(
            outputs.closeout_rel,
            "docs/handoffs/2026-05-21-test-goal-closeout.md"
        );
        assert_eq!(
            outputs.archive_rel,
            ".uselesskey/goals/archive/2026-05-21-test-goal.toml"
        );

        let closeout = fs::read_to_string(dir.path().join(to_path(&outputs.closeout_rel)))?;
        assert!(closeout.contains("## Landed Work"));
        assert!(closeout.contains("`goal-manifest-checker`"));
        assert!(closeout.contains("cargo xtask check-goals"));
        assert!(closeout.contains("## Remaining Work"));
        assert!(closeout.contains("`next-work`"));
        assert!(closeout.contains("metadata-only-audit-packets"));
        assert!(closeout.contains("policy/doc-artifacts.toml"));

        let archive = fs::read_to_string(dir.path().join(to_path(&outputs.archive_rel)))?;
        let archive_value: toml::Value = toml::from_str(&archive)?;
        assert_eq!(
            archive_value.get("status").and_then(toml::Value::as_str),
            Some("archived")
        );
        Ok(())
    }

    #[test]
    fn closeout_rejects_wrong_goal_id() -> Result<()> {
        let dir = minimal_repo()?;

        let err = write_closeout(dir.path(), "wrong-goal", "2026-05-21")
            .unwrap_err()
            .to_string();

        assert!(
            err.contains("does not match requested goal"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    #[test]
    fn closeout_rejects_archived_goal_manifest() -> Result<()> {
        let dir = minimal_repo_with_goal_status("archived")?;

        let err = write_closeout(dir.path(), "test-goal", "2026-05-21")
            .unwrap_err()
            .to_string();

        assert!(
            err.contains(".uselesskey/goals/active.toml has status `archived`"),
            "unexpected error: {err}"
        );
        Ok(())
    }

    fn minimal_repo() -> Result<tempfile::TempDir> {
        minimal_repo_with_goal_status("active")
    }

    fn minimal_repo_with_goal_status(goal_status: &str) -> Result<tempfile::TempDir> {
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
linked_adrs = ["USELESSKEY-ADR-0003"]
"#,
        )?;
        write_file(
            dir.path(),
            "docs/specs/spec.md",
            r#"+++
id = "USELESSKEY-SPEC-0023"
kind = "spec"
title = "Source-of-truth enforcement"
linked_adrs = ["USELESSKEY-ADR-0003"]
support_tier_impact = ["docs/status/SUPPORT_TIERS.md"]
policy_impact = ["policy/doc-artifacts.toml"]
+++

# Spec

## Claim Boundary

This generated closeout proves only the linked goal state.

## Rollback

Revert the generated files.
"#,
        )?;
        write_file(
            dir.path(),
            CLAIM_LEDGER_TOML,
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
        let active_goal = format!(
            r#"schema_version = "1.0"
id = "test-goal"
title = "Test goal"
status = "{goal_status}"
owner = "codex"
created = "2026-05-21"
objective = "Test objective."
end_state = ["Done."]

[[work_item]]
id = "goal-manifest-checker"
status = "done"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-0023"
plan = "plans/source-of-truth-control-plane/implementation-plan.md"
commands = ["cargo xtask check-goals"]

[[work_item]]
id = "next-work"
status = "ready"
proposal = "USELESSKEY-PROP-0002"
spec = "USELESSKEY-SPEC-0023"
plan = "plans/source-of-truth-control-plane/implementation-plan.md"
commands = ["cargo xtask next"]
"#,
        );
        write_file(dir.path(), ACTIVE_GOAL_TOML, &active_goal)?;
        Ok(dir)
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
