use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const ACTIVE_GOAL_TOML: &str = ".uselesskey/goals/active.toml";
const CLAIM_LEDGER_TOML: &str = "policy/claim-ledger.toml";
const DOC_ARTIFACTS_TOML: &str = "policy/doc-artifacts.toml";
const DEFAULT_OUT: &str = "target/source-of-truth/pr-body.md";

#[derive(Debug, Deserialize)]
struct GoalManifest {
    id: String,
    title: String,
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
struct DocArtifactLedger {
    #[serde(default)]
    artifact: Vec<DocArtifact>,
}

#[derive(Debug, Deserialize)]
struct DocArtifact {
    id: String,
    kind: String,
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
    spec: String,
    #[serde(default)]
    surfaces: Vec<String>,
}

#[derive(Debug, Default)]
struct FrontMatter {
    title: Option<String>,
    linked_adrs: Vec<String>,
    support_tier_impact: Vec<String>,
    policy_impact: Vec<String>,
}

pub(crate) fn run(root: &Path, work_item: &str) -> Result<()> {
    let out = root.join(DEFAULT_OUT);
    write_pr_body(root, work_item, &out)?;
    println!("pr-body: wrote {DEFAULT_OUT}");
    Ok(())
}

fn write_pr_body(root: &Path, work_item_id: &str, out: &Path) -> Result<String> {
    let body = generate_pr_body(root, work_item_id)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(out, &body).with_context(|| format!("write {}", out.display()))?;
    Ok(body)
}

fn generate_pr_body(root: &Path, work_item_id: &str) -> Result<String> {
    let goal: GoalManifest = read_toml(root, ACTIVE_GOAL_TOML)?;
    let artifacts = read_artifact_index(root)?;
    let claims: ClaimLedger = read_toml(root, CLAIM_LEDGER_TOML)?;
    let work_item = goal
        .work_items
        .iter()
        .find(|item| item.id == work_item_id)
        .with_context(|| format!("work item `{work_item_id}` not found in {ACTIVE_GOAL_TOML}"))?;

    let proposal = linked_artifact(work_item.proposal.as_deref(), "proposal", &artifacts)?;
    let spec = linked_artifact(work_item.spec.as_deref(), "spec", &artifacts)?;
    let plan = work_item
        .plan
        .as_deref()
        .filter(|plan| !plan.trim().is_empty())
        .unwrap_or("none");

    let spec_text = read_optional_text(root, spec.map(|artifact| artifact.path.as_str()))?;
    let proposal_text = read_optional_text(root, proposal.map(|artifact| artifact.path.as_str()))?;
    let spec_front_matter = spec_text
        .as_deref()
        .map(parse_front_matter)
        .unwrap_or_default();
    let proposal_front_matter = proposal_text
        .as_deref()
        .map(parse_front_matter)
        .unwrap_or_default();
    let adrs = if spec_front_matter.linked_adrs.is_empty() {
        spec.map(|artifact| artifact.linked_adrs.clone())
            .unwrap_or_default()
    } else {
        spec_front_matter.linked_adrs.clone()
    };
    let linked_claims = claims
        .claim
        .iter()
        .filter(|claim| {
            work_item
                .spec
                .as_deref()
                .is_some_and(|spec| claim.spec == spec)
        })
        .collect::<Vec<_>>();

    let mut out = String::new();
    out.push_str("## Summary\n\n");
    out.push_str(&format!(
        "- Implements source-of-truth work item `{}` from active goal `{}`.\n",
        work_item.id, goal.id
    ));
    if let Some(title) = spec_front_matter
        .title
        .as_deref()
        .or(proposal_front_matter.title.as_deref())
    {
        out.push_str(&format!("- Linked contract: {}\n", sentence(title)));
    }
    out.push('\n');

    out.push_str("## Links\n\n");
    out.push_str(&format!("Proposal: {}\n", artifact_link(proposal)));
    out.push_str(&format!("Spec: {}\n", artifact_link(spec)));
    out.push_str(&format!("ADR: {}\n", inline_ids(&adrs)));
    out.push_str(&format!("Plan item: `{}`\n", work_item.id));
    out.push_str("Issue: none\n\n");

    out.push_str("## Scope\n\n");
    out.push_str(&format!("- Active goal: `{}` - {}\n", goal.id, goal.title));
    out.push_str(&format!("- Work item status: `{}`\n", work_item.status));
    out.push_str(&format!("- Plan: `{plan}`\n"));
    out.push_str("- Proof commands are taken from the active goal work item.\n\n");

    out.push_str("## Non-goals\n\n");
    out.push_str(
        "- No release, publish, signing, crates.io, GitHub release, tag, or source-sync changes.\n",
    );
    out.push_str(
        "- No new public claim unless the PR also updates the claim ledger and support tiers.\n",
    );
    out.push_str("- No blocking CI promotion unless the PR explicitly changes CI policy.\n\n");

    out.push_str("## Release/source boundary\n\n");
    out.push_str("Check one:\n\n");
    out.push_str(
        "- [ ] no release, publish, signing, crates.io push, GitHub release, tag, or source-sync work\n",
    );
    out.push_str(
        "- [ ] release/source boundary touched and explicitly approved in linked issue/spec\n\n",
    );

    out.push_str("## Support-tier impact\n\n");
    out.push_str("- [ ] none\n");
    out.push_str("- [ ] updates `docs/status/SUPPORT_TIERS.md`\n");
    out.push_str("- [ ] updates `policy/claim-ledger.toml`\n");
    if !spec_front_matter.support_tier_impact.is_empty() {
        out.push_str("\nLinked spec support-tier context:\n");
        for impact in &spec_front_matter.support_tier_impact {
            out.push_str(&format!("- `{impact}`\n"));
        }
    }
    if !linked_claims.is_empty() {
        out.push_str("\nClaim-ledger entries referencing this spec:\n");
        for claim in &linked_claims {
            out.push_str(&format!(
                "- `{}` surfaces: {}\n",
                claim.id,
                inline_values(&claim.surfaces)
            ));
        }
    }
    out.push('\n');

    out.push_str("## Policy impact\n\n");
    out.push_str("- [ ] none\n");
    out.push_str("- [ ] doc artifacts\n");
    out.push_str("- [ ] negative fixture ledger\n");
    out.push_str("- [ ] claim ledger\n");
    out.push_str("- [ ] CI lane\n");
    out.push_str("- [ ] package boundary\n");
    out.push_str("- [ ] lint / Clippy\n");
    out.push_str("- [ ] no-panic\n");
    out.push_str("- [ ] file policy\n");
    if !spec_front_matter.policy_impact.is_empty() {
        out.push_str("\nLinked spec policy context:\n");
        for impact in &spec_front_matter.policy_impact {
            out.push_str(&format!("- `{impact}`\n"));
        }
    }
    out.push('\n');

    out.push_str("## Proof\n\n");
    out.push_str("```bash\n");
    for command in &work_item.commands {
        out.push_str(command);
        out.push('\n');
    }
    out.push_str("```\n\n");

    out.push_str("## Claim boundary\n\n");
    if let Some(boundary) = spec_text
        .as_deref()
        .and_then(|text| markdown_section(text, "Claim Boundary"))
    {
        out.push_str(boundary.trim());
        out.push_str("\n\n");
    } else {
        out.push_str("This PR proves only the scoped work item and listed evidence. It does not prove release readiness, downstream verifier correctness, provider compatibility, production security, or source-sync readiness.\n\n");
    }

    out.push_str("## Rollback\n\n");
    if let Some(rollback) = spec_text
        .as_deref()
        .and_then(|text| markdown_section(text, "Rollback"))
    {
        out.push_str(rollback.trim());
        out.push('\n');
    } else {
        out.push_str("Revert this PR and return the active goal to the affected work item.\n");
    }

    Ok(out)
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
    let path = root.join(to_path(rel));
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse {rel}"))
}

fn read_optional_text(root: &Path, rel: Option<&str>) -> Result<Option<String>> {
    let Some(rel) = rel else {
        return Ok(None);
    };
    let path = root.join(to_path(rel));
    fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))
        .map(Some)
}

fn linked_artifact<'a>(
    id: Option<&str>,
    kind: &str,
    artifacts: &'a BTreeMap<String, DocArtifact>,
) -> Result<Option<&'a DocArtifact>> {
    let Some(id) = id else {
        return Ok(None);
    };
    let Some(artifact) = artifacts.get(id) else {
        bail!("linked {kind} `{id}` is missing from {DOC_ARTIFACTS_TOML}");
    };
    if artifact.kind != kind {
        bail!(
            "linked {kind} `{id}` has kind `{}` in {DOC_ARTIFACTS_TOML}",
            artifact.kind
        );
    }
    Ok(Some(artifact))
}

fn parse_front_matter(markdown: &str) -> FrontMatter {
    let Some(rest) = markdown.strip_prefix("+++\n") else {
        return FrontMatter::default();
    };
    let Some((front_matter, _body)) = rest.split_once("\n+++") else {
        return FrontMatter::default();
    };
    let Ok(value) = toml::from_str::<toml::Value>(front_matter) else {
        return FrontMatter::default();
    };
    FrontMatter {
        title: value
            .get("title")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        linked_adrs: string_array(&value, "linked_adrs"),
        support_tier_impact: string_array(&value, "support_tier_impact"),
        policy_impact: string_array(&value, "policy_impact"),
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

fn artifact_link(artifact: Option<&DocArtifact>) -> String {
    artifact
        .map(|artifact| format!("`{}` - `{}`", artifact.id, artifact.path))
        .unwrap_or_else(|| "none".to_string())
}

fn inline_ids(ids: &[String]) -> String {
    if ids.is_empty() {
        "none".to_string()
    } else {
        inline_values(ids)
    }
}

fn inline_values(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values
        .iter()
        .map(|value| format!("`{}`", value.replace('|', "\\|")))
        .collect::<Vec<_>>()
        .join("; ")
}

fn sentence(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.ends_with('.') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

fn to_path(rel: &str) -> PathBuf {
    PathBuf::from(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_body_writes_linked_work_item_packet() -> Result<()> {
        let dir = minimal_repo()?;
        let out = dir.path().join("target/source-of-truth/pr-body.md");

        let body = write_pr_body(dir.path(), "goal-manifest-checker", &out)?;

        assert!(out.exists());
        assert!(body.contains("Proposal: `USELESSKEY-PROP-0002`"));
        assert!(body.contains("Spec: `USELESSKEY-SPEC-0023`"));
        assert!(body.contains("Plan item: `goal-manifest-checker`"));
        assert!(body.contains("cargo xtask check-goals"));
        assert!(body.contains("## Release/source boundary"));
        assert!(body.contains(
            "no release, publish, signing, crates.io push, GitHub release, tag, or source-sync work"
        ));
        assert!(body.contains("## Claim boundary"));
        Ok(())
    }

    #[test]
    fn pr_body_rejects_unknown_work_item() -> Result<()> {
        let dir = minimal_repo()?;
        let err = generate_pr_body(dir.path(), "missing") // error includes active manifest path
            .unwrap_err()
            .to_string();
        assert!(err.contains("work item `missing` not found"), "{err}");
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
status = "proposed"

[[artifact]]
id = "USELESSKEY-SPEC-0023"
kind = "spec"
path = "docs/specs/spec.md"
status = "accepted"
linked_proposal = "USELESSKEY-PROP-0002"
linked_adrs = ["USELESSKEY-ADR-0003"]

[[artifact]]
id = "USELESSKEY-ADR-0003"
kind = "adr"
path = "docs/adr/adr.md"
status = "accepted"
"#,
        )?;
        write_file(
            dir.path(),
            "docs/proposals/prop.md",
            r#"+++
id = "USELESSKEY-PROP-0002"
kind = "proposal"
title = "Source-of-truth control plane"
+++

# Proposal
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

This generated body proves only the linked work item.

## Rollback

Revert the PR.
"#,
        )?;
        write_file(
            dir.path(),
            CLAIM_LEDGER_TOML,
            r#"schema_version = "1.0"

[[claim]]
id = "metadata-only-audit-packets"
spec = "USELESSKEY-SPEC-0023"
surfaces = ["uselesskey audit-bundle --ci"]
"#,
        )?;
        write_file(
            dir.path(),
            ACTIVE_GOAL_TOML,
            r#"schema_version = "1.0"
id = "source-of-truth-control-plane"
title = "Source-of-truth control plane"
status = "active"
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
commands = ["cargo xtask check-goals", "git diff --check"]
"#,
        )?;
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
