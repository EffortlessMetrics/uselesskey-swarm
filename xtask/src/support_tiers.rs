use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const CLAIM_LEDGER_TOML: &str = "policy/claim-ledger.toml";
const DOC_ARTIFACTS_TOML: &str = "policy/doc-artifacts.toml";
const SUPPORT_TIERS_MD: &str = "docs/status/SUPPORT_TIERS.md";
const WORKFLOW_SUPPORT_MD: &str = "docs/status/workflow-support.md";
const VALID_TIERS: &[&str] = &[
    "Stable",
    "Stabilizing",
    "Experimental",
    "Advisory",
    "Not supported",
];
const PROOF_REQUIRED_TIERS: &[&str] = &["Stable", "Stabilizing"];

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
    proof_commands: Vec<String>,
    #[serde(default)]
    docs: Vec<String>,
    #[serde(default)]
    release_lanes: Vec<String>,
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
pub(crate) struct SupportRow {
    pub(crate) line: usize,
    pub(crate) surface: String,
    pub(crate) tier: String,
    pub(crate) claim: String,
    pub(crate) proof: String,
    pub(crate) docs: String,
    pub(crate) boundary: String,
    pub(crate) release_lane: String,
}

#[derive(Debug)]
pub(crate) struct WorkflowRow {
    pub(crate) line: usize,
    pub(crate) workflow: String,
    pub(crate) support_tier: String,
    pub(crate) claim: String,
    pub(crate) primary_docs: String,
    pub(crate) proof_commands: String,
    pub(crate) receipts: String,
    pub(crate) boundary: String,
}

#[derive(Debug)]
struct WorkflowTierDefinition {
    line: usize,
    tier: String,
    meaning: String,
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let errors = validate(root)?;
    if errors.is_empty() {
        let ledger = read_claim_ledger(root)?;
        let rows = read_support_rows(root)?;
        let workflows = read_workflow_rows(root)?;
        println!(
            "support-tiers: {} claims; {} support rows; {} workflow rows; map ok",
            ledger.claim.len(),
            rows.len(),
            workflows.len()
        );
        Ok(())
    } else {
        for error in &errors {
            eprintln!("support-tiers: {error}");
        }
        bail!("support-tiers: {} validation error(s)", errors.len());
    }
}

fn validate(root: &Path) -> Result<Vec<String>> {
    let ledger = read_claim_ledger(root)?;
    let artifacts = read_doc_artifacts(root)?;
    let rows = read_support_rows(root)?;
    let workflow_rows = read_workflow_rows(root)?;
    let workflow_tiers = read_workflow_tier_definitions(root)?;
    let mut errors = Vec::new();

    if ledger.claim.is_empty() {
        errors.push(format!("{CLAIM_LEDGER_TOML}: no [[claim]] entries found"));
    }
    if rows.is_empty() {
        errors.push(format!(
            "{SUPPORT_TIERS_MD}: no parseable Claim Support Map rows found"
        ));
    }
    if workflow_rows.is_empty() {
        errors.push(format!(
            "{WORKFLOW_SUPPORT_MD}: no parseable Workflow Matrix rows found"
        ));
    }
    if workflow_tiers.is_empty() {
        errors.push(format!(
            "{WORKFLOW_SUPPORT_MD}: no parseable Support Tier Interpretation rows found"
        ));
    }

    let mut claim_ids = BTreeSet::new();
    for claim in &ledger.claim {
        if !claim_ids.insert(claim.id.clone()) {
            errors.push(format!(
                "{CLAIM_LEDGER_TOML}: duplicate claim id `{}`",
                claim.id
            ));
        }
        if claim
            .proof_commands
            .iter()
            .any(|command| command.trim().is_empty())
        {
            errors.push(format!(
                "{CLAIM_LEDGER_TOML}: claim `{}` has an empty proof command",
                claim.id
            ));
        }
        if claim.release_lanes.is_empty() {
            errors.push(format!(
                "{CLAIM_LEDGER_TOML}: claim `{}` has no release_lanes",
                claim.id
            ));
        }
        if claim
            .release_lanes
            .iter()
            .any(|lane| lane.trim().is_empty())
        {
            errors.push(format!(
                "{CLAIM_LEDGER_TOML}: claim `{}` has an empty release lane",
                claim.id
            ));
        }
        for doc in &claim.docs {
            validate_existing_path(root, CLAIM_LEDGER_TOML, &claim.id, doc, &mut errors);
        }
    }

    let specs = artifacts
        .artifact
        .iter()
        .filter(|artifact| artifact.kind == "spec")
        .map(|artifact| artifact.id.as_str())
        .collect::<BTreeSet<_>>();
    for claim in &ledger.claim {
        if claim.spec.trim().is_empty() {
            errors.push(format!(
                "{CLAIM_LEDGER_TOML}: claim `{}` has no spec",
                claim.id
            ));
        } else if !specs.contains(claim.spec.as_str()) {
            errors.push(format!(
                "{CLAIM_LEDGER_TOML}: claim `{}` references unknown spec `{}`",
                claim.id, claim.spec
            ));
        }
    }

    let claims = ledger
        .claim
        .iter()
        .map(|claim| (claim.id.as_str(), claim))
        .collect::<BTreeMap<_, _>>();
    let support_claims = rows
        .iter()
        .map(|row| row.claim.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen_surfaces = BTreeSet::new();

    for claim in &ledger.claim {
        if !support_claims.contains(claim.id.as_str()) {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}: claim `{}` is missing from the support map",
                claim.id
            ));
        }
    }

    for row in &rows {
        if !seen_surfaces.insert(row.surface.as_str()) {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}:{} duplicate support surface `{}`",
                row.line, row.surface
            ));
        }

        if !VALID_TIERS.contains(&row.tier.as_str()) {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}:{} surface `{}` has invalid tier `{}`",
                row.line, row.surface, row.tier
            ));
        }

        let row_docs = inline_code_values(&row.docs);
        let Some(claim) = claims.get(row.claim.as_str()) else {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}:{} surface `{}` references unknown claim `{}`",
                row.line, row.surface, row.claim
            ));
            continue;
        };

        let row_proofs = inline_code_values(&row.proof);
        let row_release_lanes = inline_code_values(&row.release_lane);
        let claim_proofs = claim
            .proof_commands
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let claim_release_lanes = claim
            .release_lanes
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        for proof in &row_proofs {
            if !claim_proofs.contains(proof.as_str()) {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} surface `{}` lists proof command `{}` that is not in claim `{}`",
                    row.line, row.surface, proof, claim.id
                ));
            }
        }
        for proof in &claim.proof_commands {
            if !row_proofs.iter().any(|row_proof| row_proof == proof) {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} surface `{}` omits claim-ledger proof command `{}` for claim `{}`",
                    row.line, row.surface, proof, claim.id
                ));
            }
        }
        for lane in &row_release_lanes {
            if !claim_release_lanes.contains(lane.as_str()) {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} surface `{}` lists release lane `{}` that is not in claim `{}`",
                    row.line, row.surface, lane, claim.id
                ));
            }
        }
        for lane in &claim.release_lanes {
            if !row_release_lanes
                .iter()
                .any(|row_release_lane| row_release_lane == lane)
            {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} surface `{}` omits claim-ledger release lane `{}` for claim `{}`",
                    row.line, row.surface, lane, claim.id
                ));
            }
        }

        if PROOF_REQUIRED_TIERS.contains(&row.tier.as_str()) {
            if row.proof.trim().is_empty() || row.proof.trim() == "none" {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} `{}` tier requires visible proof commands",
                    row.line, row.tier
                ));
            }
            if claim.proof_commands.is_empty() {
                errors.push(format!(
                    "{CLAIM_LEDGER_TOML}: claim `{}` is `{}` in support tiers but has no proof_commands",
                    claim.id, row.tier
                ));
            }
            if row_release_lanes.is_empty() {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} `{}` tier requires visible release lanes",
                    row.line, row.tier
                ));
            }
            if !row_docs.iter().any(|path| is_repo_path(path)) {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} `{}` tier requires at least one visible docs path",
                    row.line, row.tier
                ));
            }
        }

        if row.boundary.trim().is_empty() {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}:{} surface `{}` has an empty boundary",
                row.line, row.surface
            ));
        }
        for doc in row_docs.into_iter().filter(|path| is_repo_path(path)) {
            validate_existing_path(root, SUPPORT_TIERS_MD, &row.surface, &doc, &mut errors);
        }
    }

    validate_workflow_tier_definitions(&workflow_tiers, &mut errors);
    validate_workflow_rows(&workflow_rows, &workflow_tiers, &claims, root, &mut errors);
    validate_matching_workflow_support_proofs(&rows, &workflow_rows, &mut errors);

    Ok(errors)
}

fn validate_workflow_rows(
    rows: &[WorkflowRow],
    workflow_tiers: &[WorkflowTierDefinition],
    claims: &BTreeMap<&str, &ClaimEntry>,
    root: &Path,
    errors: &mut Vec<String>,
) {
    let workflow_tier_names = workflow_tiers
        .iter()
        .map(|tier| tier.tier.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen_workflows = BTreeSet::new();

    for row in rows {
        if !seen_workflows.insert(row.workflow.as_str()) {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} duplicate workflow `{}`",
                row.line, row.workflow
            ));
        }

        if row.support_tier.trim().is_empty() {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` has an empty support tier",
                row.line, row.workflow
            ));
        } else if !workflow_tier_names.contains(row.support_tier.as_str()) {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` uses undefined support tier `{}`",
                row.line, row.workflow, row.support_tier
            ));
        }

        let Some(claim) = claims.get(row.claim.as_str()) else {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` references unknown claim `{}`",
                row.line, row.workflow, row.claim
            ));
            continue;
        };

        let docs = inline_code_values(&row.primary_docs);
        if docs.is_empty() {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` has no primary docs",
                row.line, row.workflow
            ));
        }
        for doc in docs.into_iter().filter(|path| is_repo_path(path)) {
            validate_existing_path(root, WORKFLOW_SUPPORT_MD, &row.workflow, &doc, errors);
        }

        let proof_commands = inline_code_values(&row.proof_commands);
        if proof_commands.is_empty() {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` has no visible proof commands",
                row.line, row.workflow
            ));
        } else if !claim.proof_commands.is_empty()
            && !proof_commands.iter().any(|proof| {
                claim
                    .proof_commands
                    .iter()
                    .any(|claim_proof| claim_proof == proof)
            })
        {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` has no proof command from claim `{}`",
                row.line, row.workflow, claim.id
            ));
        }

        if inline_code_values(&row.receipts).is_empty() {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` has no receipt paths",
                row.line, row.workflow
            ));
        }
        if row.boundary.trim().is_empty() {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` has an empty boundary",
                row.line, row.workflow
            ));
        }
    }
}

fn validate_matching_workflow_support_proofs(
    support_rows: &[SupportRow],
    workflow_rows: &[WorkflowRow],
    errors: &mut Vec<String>,
) {
    let support_by_surface = support_rows
        .iter()
        .map(|row| (row.surface.as_str(), row))
        .collect::<BTreeMap<_, _>>();

    for workflow in workflow_rows {
        let Some(support) = support_by_surface.get(workflow.workflow.as_str()) else {
            continue;
        };
        let workflow_proofs = inline_code_values(&workflow.proof_commands)
            .into_iter()
            .collect::<BTreeSet<_>>();
        for proof in inline_code_values(&support.proof) {
            if !workflow_proofs.contains(&proof) {
                errors.push(format!(
                    "{WORKFLOW_SUPPORT_MD}:{} workflow `{}` omits support-tier proof command `{}` from matching support surface",
                    workflow.line, workflow.workflow, proof
                ));
            }
        }
    }
}

fn validate_workflow_tier_definitions(
    workflow_tiers: &[WorkflowTierDefinition],
    errors: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();
    for tier in workflow_tiers {
        if !seen.insert(tier.tier.as_str()) {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} duplicate workflow support tier `{}`",
                tier.line, tier.tier
            ));
        }
        if tier.meaning.trim().is_empty() {
            errors.push(format!(
                "{WORKFLOW_SUPPORT_MD}:{} workflow support tier `{}` has an empty meaning",
                tier.line, tier.tier
            ));
        }
    }
}

fn read_claim_ledger(root: &Path) -> Result<ClaimLedger> {
    let path = root.join(CLAIM_LEDGER_TOML);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse {CLAIM_LEDGER_TOML}"))
}

fn read_doc_artifacts(root: &Path) -> Result<DocArtifactLedger> {
    let path = root.join(DOC_ARTIFACTS_TOML);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse {DOC_ARTIFACTS_TOML}"))
}

pub(crate) fn read_support_rows(root: &Path) -> Result<Vec<SupportRow>> {
    let path = root.join(SUPPORT_TIERS_MD);
    let markdown = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_support_rows(&markdown))
}

pub(crate) fn read_workflow_rows(root: &Path) -> Result<Vec<WorkflowRow>> {
    let path = root.join(WORKFLOW_SUPPORT_MD);
    let markdown = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_workflow_rows(&markdown))
}

fn read_workflow_tier_definitions(root: &Path) -> Result<Vec<WorkflowTierDefinition>> {
    let path = root.join(WORKFLOW_SUPPORT_MD);
    let markdown = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_workflow_tier_definitions(&markdown))
}

fn parse_support_rows(markdown: &str) -> Vec<SupportRow> {
    let mut rows = Vec::new();
    let mut in_map = false;

    for (idx, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "## Claim Support Map" {
            in_map = true;
            continue;
        }
        if in_map && trimmed.starts_with("## ") {
            break;
        }
        if !in_map || !trimmed.starts_with('|') {
            continue;
        }

        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 7 {
            continue;
        }
        if cells[0] == "Surface" || cells[0].starts_with("---") {
            continue;
        }

        rows.push(SupportRow {
            line: idx + 1,
            surface: cells[0].to_string(),
            tier: strip_inline_code(cells[1]),
            claim: strip_inline_code(cells[2]),
            proof: cells[3].to_string(),
            docs: cells[4].to_string(),
            boundary: cells[5].to_string(),
            release_lane: cells[6].to_string(),
        });
    }

    rows
}

fn parse_workflow_rows(markdown: &str) -> Vec<WorkflowRow> {
    let mut rows = Vec::new();
    let mut in_map = false;

    for (idx, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "## Workflow Matrix" {
            in_map = true;
            continue;
        }
        if in_map && trimmed.starts_with("## ") {
            break;
        }
        if !in_map || !trimmed.starts_with('|') {
            continue;
        }

        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 7 {
            continue;
        }
        if cells[0] == "Workflow" || cells[0].starts_with("---") {
            continue;
        }

        rows.push(WorkflowRow {
            line: idx + 1,
            workflow: cells[0].to_string(),
            support_tier: cells[1].to_string(),
            claim: strip_inline_code(cells[2]),
            primary_docs: cells[3].to_string(),
            proof_commands: cells[4].to_string(),
            receipts: cells[5].to_string(),
            boundary: cells[6].to_string(),
        });
    }

    rows
}

fn parse_workflow_tier_definitions(markdown: &str) -> Vec<WorkflowTierDefinition> {
    let mut tiers = Vec::new();
    let mut in_map = false;

    for (idx, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "## Support Tier Interpretation" {
            in_map = true;
            continue;
        }
        if in_map && trimmed.starts_with("## ") {
            break;
        }
        if !in_map || !trimmed.starts_with('|') {
            continue;
        }

        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 2 {
            continue;
        }
        if cells[0] == "Tier" || cells[0].starts_with("---") {
            continue;
        }

        tiers.push(WorkflowTierDefinition {
            line: idx + 1,
            tier: strip_inline_code(cells[0]),
            meaning: cells[1].to_string(),
        });
    }

    tiers
}

fn validate_existing_path(
    root: &Path,
    source: &str,
    owner: &str,
    rel: &str,
    errors: &mut Vec<String>,
) {
    if !is_repo_path(rel) {
        return;
    }
    let path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !path.exists() {
        errors.push(format!(
            "{source}: `{owner}` references missing path `{rel}`"
        ));
    }
}

fn is_repo_path(path: &str) -> bool {
    path.starts_with("docs/")
        || path.starts_with("badges/")
        || path.starts_with("policy/")
        || path.starts_with("examples/")
}

fn strip_inline_code(value: &str) -> String {
    value.trim().trim_matches('`').trim().to_string()
}

fn inline_code_values(value: &str) -> Vec<String> {
    value
        .split('`')
        .enumerate()
        .filter_map(|(idx, part)| {
            if idx % 2 == 1 {
                Some(part.trim().to_string())
            } else {
                None
            }
        })
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_support_tier_map() -> Result<()> {
        let dir = minimal_repo()?;
        let errors = validate(dir.path())?;
        assert!(errors.is_empty(), "errors: {errors:?}");
        Ok(())
    }

    #[test]
    fn rejects_duplicate_claim_id() -> Result<()> {
        let dir = minimal_repo()?;
        append_claim(dir.path(), valid_claim("scanner-safe-fixtures"))?;
        assert_error(dir.path(), "duplicate claim id `scanner-safe-fixtures`")
    }

    #[test]
    fn rejects_invalid_tier() -> Result<()> {
        let dir = minimal_repo()?;
        write_support_tiers(
            dir.path(),
            "Beta",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`",
        )?;
        assert_error(dir.path(), "invalid tier `Beta`")
    }

    #[test]
    fn rejects_missing_doc_path() -> Result<()> {
        let dir = minimal_repo()?;
        write_claim_ledger(
            dir.path(),
            &valid_claim_with_docs("scanner-safe-fixtures", &["docs/missing.md"]),
        )?;
        assert_error(dir.path(), "references missing path `docs/missing.md`")
    }

    #[test]
    fn rejects_missing_spec_in_doc_artifacts() -> Result<()> {
        let dir = minimal_repo()?;
        write_claim_ledger(
            dir.path(),
            r#"
[[claim]]
id = "scanner-safe-fixtures"
title = "Scanner-safe fixtures"
status = "stable"
spec = "USELESSKEY-SPEC-9999"
surfaces = ["README"]
proof_commands = ["cargo xtask no-blob"]
docs = ["docs/VERIFICATION.md"]
release_lanes = ["pr"]
"#,
        )?;
        assert_error(dir.path(), "references unknown spec `USELESSKEY-SPEC-9999`")
    }

    #[test]
    fn rejects_claim_without_release_lanes() -> Result<()> {
        let dir = minimal_repo()?;
        write_claim_ledger(
            dir.path(),
            r#"
[[claim]]
id = "scanner-safe-fixtures"
title = "Scanner-safe fixtures"
status = "stable"
spec = "USELESSKEY-SPEC-0002"
surfaces = ["README"]
proof_commands = ["cargo xtask no-blob"]
docs = ["docs/VERIFICATION.md"]
"#,
        )?;
        assert_error(
            dir.path(),
            "claim `scanner-safe-fixtures` has no release_lanes",
        )
    }

    #[test]
    fn rejects_support_row_unknown_claim() -> Result<()> {
        let dir = minimal_repo()?;
        write_support_tiers(
            dir.path(),
            "Stable",
            "`unknown-claim`",
            "`cargo xtask no-blob`",
        )?;
        assert_error(dir.path(), "references unknown claim `unknown-claim`")
    }

    #[test]
    fn rejects_claim_without_support_row() -> Result<()> {
        let dir = minimal_repo()?;
        append_claim(dir.path(), valid_claim("ripr-pr-review-evidence"))?;
        assert_error(
            dir.path(),
            "claim `ripr-pr-review-evidence` is missing from the support map",
        )
    }

    #[test]
    fn rejects_stabilizing_without_proof() -> Result<()> {
        let dir = minimal_repo()?;
        write_support_tiers(dir.path(), "Stabilizing", "`scanner-safe-fixtures`", "none")?;
        assert_error(dir.path(), "tier requires visible proof commands")
    }

    #[test]
    fn rejects_stable_without_docs_path() -> Result<()> {
        let dir = minimal_repo()?;
        write_support_tiers_full(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`",
            "none",
        )?;
        assert_error(dir.path(), "tier requires at least one visible docs path")
    }

    #[test]
    fn rejects_support_row_unbacked_proof_command() -> Result<()> {
        let dir = minimal_repo()?;
        write_support_tiers(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`; `cargo xtask fake-proof`",
        )?;
        assert_error(
            dir.path(),
            "lists proof command `cargo xtask fake-proof` that is not in claim `scanner-safe-fixtures`",
        )
    }

    #[test]
    fn rejects_support_row_missing_claim_proof_command() -> Result<()> {
        let dir = minimal_repo()?;
        write_claim_ledger(
            dir.path(),
            r#"
[[claim]]
id = "scanner-safe-fixtures"
title = "Scanner-safe fixtures"
status = "stable"
spec = "USELESSKEY-SPEC-0002"
surfaces = ["README"]
proof_commands = ["cargo xtask no-blob", "cargo xtask badges --check"]
docs = ["docs/VERIFICATION.md"]
release_lanes = ["pr"]
"#,
        )?;
        write_support_tiers(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`",
        )?;
        assert_error(
            dir.path(),
            "omits claim-ledger proof command `cargo xtask badges --check` for claim `scanner-safe-fixtures`",
        )
    }

    #[test]
    fn rejects_support_row_unbacked_release_lane() -> Result<()> {
        let dir = minimal_repo()?;
        write_support_tiers_with_release_lane(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`",
            "`docs/VERIFICATION.md`",
            "`nightly`",
        )?;
        assert_error(
            dir.path(),
            "lists release lane `nightly` that is not in claim `scanner-safe-fixtures`",
        )
    }

    #[test]
    fn rejects_support_row_missing_claim_release_lane() -> Result<()> {
        let dir = minimal_repo()?;
        write_claim_ledger(
            dir.path(),
            r#"
[[claim]]
id = "scanner-safe-fixtures"
title = "Scanner-safe fixtures"
status = "stable"
spec = "USELESSKEY-SPEC-0002"
surfaces = ["README"]
proof_commands = ["cargo xtask no-blob"]
docs = ["docs/VERIFICATION.md"]
release_lanes = ["pr", "minor"]
"#,
        )?;
        write_support_tiers(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`",
        )?;
        assert_error(
            dir.path(),
            "omits claim-ledger release lane `minor` for claim `scanner-safe-fixtures`",
        )
    }

    #[test]
    fn rejects_duplicate_support_surface() -> Result<()> {
        let dir = minimal_repo()?;
        append_duplicate_support_row(dir.path())?;
        assert_error(
            dir.path(),
            "duplicate support surface `Scanner-safe fixtures`",
        )
    }

    #[test]
    fn rejects_workflow_row_unknown_claim() -> Result<()> {
        let dir = minimal_repo()?;
        write_workflow_support(
            dir.path(),
            "`unknown-claim`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask no-blob`",
        )?;
        assert_error(dir.path(), "references unknown claim `unknown-claim`")
    }

    #[test]
    fn rejects_workflow_row_without_claim_proof_command() -> Result<()> {
        let dir = minimal_repo()?;
        write_workflow_support(
            dir.path(),
            "`scanner-safe-fixtures`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask fake-proof`",
        )?;
        assert_error(
            dir.path(),
            "has no proof command from claim `scanner-safe-fixtures`",
        )
    }

    #[test]
    fn rejects_workflow_row_missing_primary_doc() -> Result<()> {
        let dir = minimal_repo()?;
        write_workflow_support(
            dir.path(),
            "`scanner-safe-fixtures`",
            "`docs/missing.md`",
            "`cargo xtask no-blob`",
        )?;
        assert_error(dir.path(), "references missing path `docs/missing.md`")
    }

    #[test]
    fn rejects_workflow_row_undefined_support_tier() -> Result<()> {
        let dir = minimal_repo()?;
        write_workflow_support_with_tier(
            dir.path(),
            "future workflow tier",
            "`scanner-safe-fixtures`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask no-blob`",
        )?;
        assert_error(
            dir.path(),
            "uses undefined support tier `future workflow tier`",
        )
    }

    #[test]
    fn rejects_workflow_row_without_receipts() -> Result<()> {
        let dir = minimal_repo()?;
        write_workflow_support_full(
            dir.path(),
            "stable bundle workflow",
            "`scanner-safe-fixtures`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask no-blob`",
            "none",
            "| stable bundle workflow | Installed CLI bundle path covered by external adoption smoke and metadata receipts. |",
        )?;
        assert_error(
            dir.path(),
            "workflow `Scanner-safe bundle handoff` has no receipt paths",
        )
    }

    #[test]
    fn rejects_workflow_tier_without_meaning() -> Result<()> {
        let dir = minimal_repo()?;
        write_workflow_support_full(
            dir.path(),
            "stable bundle workflow",
            "`scanner-safe-fixtures`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask no-blob`",
            "`target/external-adoption-smoke/report.json`",
            "| stable bundle workflow |  |",
        )?;
        assert_error(
            dir.path(),
            "workflow support tier `stable bundle workflow` has an empty meaning",
        )
    }

    #[test]
    fn rejects_duplicate_workflow_row() -> Result<()> {
        let dir = minimal_repo()?;
        append_duplicate_workflow_row(dir.path())?;
        assert_error(
            dir.path(),
            "duplicate workflow `Scanner-safe bundle handoff`",
        )
    }

    #[test]
    fn rejects_matching_workflow_row_missing_support_proof() -> Result<()> {
        let dir = minimal_repo()?;
        write_claim_ledger(
            dir.path(),
            r#"
[[claim]]
id = "scanner-safe-fixtures"
title = "Scanner-safe fixtures"
status = "stable"
spec = "USELESSKEY-SPEC-0002"
surfaces = ["README"]
proof_commands = ["cargo xtask no-blob", "cargo xtask badges --check"]
docs = ["docs/VERIFICATION.md"]
release_lanes = ["pr"]
"#,
        )?;
        write_support_tiers(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`; `cargo xtask badges --check`",
        )?;
        write_workflow_support_named(
            dir.path(),
            "Scanner-safe fixtures",
            "stable bundle workflow",
            "`scanner-safe-fixtures`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask no-blob`",
            "`target/external-adoption-smoke/report.json`",
        )?;
        assert_error(
            dir.path(),
            "workflow `Scanner-safe fixtures` omits support-tier proof command `cargo xtask badges --check`",
        )
    }

    #[test]
    fn allows_differently_named_workflow_row_to_use_subset_of_claim_proofs() -> Result<()> {
        let dir = minimal_repo()?;
        write_claim_ledger(
            dir.path(),
            r#"
[[claim]]
id = "scanner-safe-fixtures"
title = "Scanner-safe fixtures"
status = "stable"
spec = "USELESSKEY-SPEC-0002"
surfaces = ["README"]
proof_commands = ["cargo xtask no-blob", "cargo xtask badges --check"]
docs = ["docs/VERIFICATION.md"]
release_lanes = ["pr"]
"#,
        )?;
        write_support_tiers(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`; `cargo xtask badges --check`",
        )?;
        write_workflow_support(
            dir.path(),
            "`scanner-safe-fixtures`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask no-blob`",
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

    fn minimal_repo() -> Result<tempfile::TempDir> {
        let dir = tempfile::tempdir()?;
        write_file(
            dir.path(),
            "policy/doc-artifacts.toml",
            r#"schema_version = "1.0"
owner = "EffortlessMetrics"
updated = "2026-05-21"

[[artifact]]
id = "USELESSKEY-SPEC-0002"
kind = "spec"
path = "docs/specs/spec.md"
status = "accepted"
owner = "repo-infra"
standalone_reason = "test"
"#,
        )?;
        write_file(dir.path(), "docs/specs/spec.md", "USELESSKEY-SPEC-0002")?;
        write_file(dir.path(), "docs/VERIFICATION.md", "# Verification\n")?;
        write_claim_ledger(dir.path(), &valid_claim("scanner-safe-fixtures"))?;
        write_support_tiers(
            dir.path(),
            "Stable",
            "`scanner-safe-fixtures`",
            "`cargo xtask no-blob`",
        )?;
        write_workflow_support(
            dir.path(),
            "`scanner-safe-fixtures`",
            "`docs/VERIFICATION.md`",
            "`cargo xtask no-blob`",
        )?;
        Ok(dir)
    }

    fn valid_claim(id: &str) -> String {
        valid_claim_with_docs(id, &["docs/VERIFICATION.md"])
    }

    fn valid_claim_with_docs(id: &str, docs: &[&str]) -> String {
        let docs = docs
            .iter()
            .map(|doc| format!("\"{doc}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"
[[claim]]
id = "{id}"
title = "Scanner-safe fixtures"
status = "stable"
spec = "USELESSKEY-SPEC-0002"
surfaces = ["README"]
proof_commands = ["cargo xtask no-blob"]
docs = [{docs}]
release_lanes = ["pr"]
"#
        )
    }

    fn write_claim_ledger(root: &Path, body: &str) -> Result<()> {
        write_file(
            root,
            CLAIM_LEDGER_TOML,
            &format!(
                r#"schema_version = 1
owner = "EffortlessMetrics"
updated = "2026-05-21"
{body}
"#
            ),
        )
    }

    fn append_claim(root: &Path, body: String) -> Result<()> {
        let path = root.join(CLAIM_LEDGER_TOML.replace('/', std::path::MAIN_SEPARATOR_STR));
        let mut text = fs::read_to_string(&path)?;
        text.push_str(&body);
        fs::write(path, text)?;
        Ok(())
    }

    fn write_support_tiers(root: &Path, tier: &str, claim: &str, proof: &str) -> Result<()> {
        write_support_tiers_full(root, tier, claim, proof, "`docs/VERIFICATION.md`")
    }

    fn write_support_tiers_full(
        root: &Path,
        tier: &str,
        claim: &str,
        proof: &str,
        docs: &str,
    ) -> Result<()> {
        write_support_tiers_with_release_lane(root, tier, claim, proof, docs, "`pr`")
    }

    fn write_support_tiers_with_release_lane(
        root: &Path,
        tier: &str,
        claim: &str,
        proof: &str,
        docs: &str,
        release_lane: &str,
    ) -> Result<()> {
        write_file(
            root,
            SUPPORT_TIERS_MD,
            &format!(
                r#"# Support Tiers

## Claim Support Map

| Surface | Tier | Claim | Proof command | Docs | Boundary | Release lane |
| --- | --- | --- | --- | --- | --- | --- |
| Scanner-safe fixtures | {tier} | {claim} | {proof} | {docs} | Boundary. | {release_lane} |

## Explicit Non-Support
"#
            ),
        )
    }

    fn write_workflow_support(
        root: &Path,
        claim: &str,
        primary_docs: &str,
        proof: &str,
    ) -> Result<()> {
        write_workflow_support_with_tier(root, "stable bundle workflow", claim, primary_docs, proof)
    }

    fn write_workflow_support_with_tier(
        root: &Path,
        tier: &str,
        claim: &str,
        primary_docs: &str,
        proof: &str,
    ) -> Result<()> {
        write_workflow_support_full(
            root,
            tier,
            claim,
            primary_docs,
            proof,
            "`target/external-adoption-smoke/report.json`",
            "| stable bundle workflow | Installed CLI bundle path covered by external adoption smoke and metadata receipts. |",
        )
    }

    fn write_workflow_support_full(
        root: &Path,
        tier: &str,
        claim: &str,
        primary_docs: &str,
        proof: &str,
        receipts: &str,
        tier_definition: &str,
    ) -> Result<()> {
        write_file(
            root,
            WORKFLOW_SUPPORT_MD,
            &format!(
                r#"# Workflow Support

## Workflow Matrix

| Workflow | Support tier | Public claim | Primary docs | Proof commands | Receipts | Boundary |
| --- | --- | --- | --- | --- | --- | --- |
| Scanner-safe bundle handoff | {tier} | {claim} | {primary_docs} | {proof} | {receipts} | Boundary. |

## Support Tier Interpretation

| Tier | Meaning |
| --- | --- |
{tier_definition}
"#
            ),
        )
    }

    fn write_workflow_support_named(
        root: &Path,
        workflow: &str,
        tier: &str,
        claim: &str,
        primary_docs: &str,
        proof: &str,
        receipts: &str,
    ) -> Result<()> {
        write_file(
            root,
            WORKFLOW_SUPPORT_MD,
            &format!(
                r#"# Workflow Support

## Workflow Matrix

| Workflow | Support tier | Public claim | Primary docs | Proof commands | Receipts | Boundary |
| --- | --- | --- | --- | --- | --- | --- |
| {workflow} | {tier} | {claim} | {primary_docs} | {proof} | {receipts} | Boundary. |

## Support Tier Interpretation

| Tier | Meaning |
| --- | --- |
| stable bundle workflow | Installed CLI bundle path covered by external adoption smoke and metadata receipts. |
"#
            ),
        )
    }

    fn append_duplicate_support_row(root: &Path) -> Result<()> {
        let path = root.join(SUPPORT_TIERS_MD.replace('/', std::path::MAIN_SEPARATOR_STR));
        let text = fs::read_to_string(&path)?;
        let duplicate = "| Scanner-safe fixtures | Stable | `scanner-safe-fixtures` | `cargo xtask no-blob` | `docs/VERIFICATION.md` | Boundary. | `pr` |\n";
        let updated = text.replacen(
            "\n## Explicit Non-Support",
            &format!("\n{duplicate}\n## Explicit Non-Support"),
            1,
        );
        fs::write(path, updated)?;
        Ok(())
    }

    fn append_duplicate_workflow_row(root: &Path) -> Result<()> {
        let path = root.join(WORKFLOW_SUPPORT_MD.replace('/', std::path::MAIN_SEPARATOR_STR));
        let text = fs::read_to_string(&path)?;
        let duplicate = "| Scanner-safe bundle handoff | stable bundle workflow | `scanner-safe-fixtures` | `docs/VERIFICATION.md` | `cargo xtask no-blob` | `target/external-adoption-smoke/report.json` | Boundary. |\n";
        let updated = text.replacen(
            "\n## Support Tier Interpretation",
            &format!("\n{duplicate}\n## Support Tier Interpretation"),
            1,
        );
        fs::write(path, updated)?;
        Ok(())
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
