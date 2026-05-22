use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const CLAIM_LEDGER_TOML: &str = "policy/claim-ledger.toml";
const DOC_ARTIFACTS_TOML: &str = "policy/doc-artifacts.toml";
const SUPPORT_TIERS_MD: &str = "docs/status/SUPPORT_TIERS.md";
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
    status: String,
    #[serde(default)]
    spec: String,
    #[serde(default)]
    surfaces: Vec<String>,
    #[serde(default)]
    proof_commands: Vec<String>,
    #[serde(default)]
    docs: Vec<String>,
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
struct SupportRow {
    line: usize,
    surface: String,
    tier: String,
    claim: String,
    proof: String,
    docs: String,
    boundary: String,
    release_lane: String,
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let errors = validate(root)?;
    if errors.is_empty() {
        let ledger = read_claim_ledger(root)?;
        let rows = read_support_rows(root)?;
        println!(
            "support-tiers: {} claims; {} support rows; map ok",
            ledger.claim.len(),
            rows.len()
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
    let mut errors = Vec::new();

    if ledger.claim.is_empty() {
        errors.push(format!("{CLAIM_LEDGER_TOML}: no [[claim]] entries found"));
    }
    if rows.is_empty() {
        errors.push(format!(
            "{SUPPORT_TIERS_MD}: no parseable Claim Support Map rows found"
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

    for row in &rows {
        if !VALID_TIERS.contains(&row.tier.as_str()) {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}:{} surface `{}` has invalid tier `{}`",
                row.line, row.surface, row.tier
            ));
        }

        let Some(claim) = claims.get(row.claim.as_str()) else {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}:{} surface `{}` references unknown claim `{}`",
                row.line, row.surface, row.claim
            ));
            continue;
        };

        let row_proofs = inline_code_values(&row.proof);
        let claim_proofs = claim
            .proof_commands
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
            if row.release_lane.trim().is_empty() || row.release_lane.trim() == "none" {
                errors.push(format!(
                    "{SUPPORT_TIERS_MD}:{} `{}` tier requires a release lane",
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
        for doc in inline_code_values(&row.docs)
            .into_iter()
            .filter(|path| is_repo_doc_path(path))
        {
            validate_existing_path(root, SUPPORT_TIERS_MD, &row.surface, &doc, &mut errors);
        }
    }

    for claim in ledger
        .claim
        .iter()
        .filter(|claim| claim.status == "stable")
        .filter(|claim| {
            claim
                .surfaces
                .iter()
                .any(|surface| surface.to_ascii_lowercase().contains("readme"))
        })
    {
        if !support_claims.contains(claim.id.as_str()) {
            errors.push(format!(
                "{SUPPORT_TIERS_MD}: README stable claim `{}` is missing from the support map",
                claim.id
            ));
        }
    }

    Ok(errors)
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

fn read_support_rows(root: &Path) -> Result<Vec<SupportRow>> {
    let path = root.join(SUPPORT_TIERS_MD);
    let markdown = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_support_rows(&markdown))
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

fn validate_existing_path(
    root: &Path,
    source: &str,
    owner: &str,
    rel: &str,
    errors: &mut Vec<String>,
) {
    if !is_repo_doc_path(rel) {
        return;
    }
    let path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !path.exists() {
        errors.push(format!(
            "{source}: `{owner}` references missing path `{rel}`"
        ));
    }
}

fn is_repo_doc_path(path: &str) -> bool {
    path.starts_with("docs/") || path.starts_with("badges/") || path.starts_with("policy/")
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
"#,
        )?;
        assert_error(dir.path(), "references unknown spec `USELESSKEY-SPEC-9999`")
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
    fn rejects_stabilizing_without_proof() -> Result<()> {
        let dir = minimal_repo()?;
        write_support_tiers(dir.path(), "Stabilizing", "`scanner-safe-fixtures`", "none")?;
        assert_error(dir.path(), "tier requires visible proof commands")
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
        write_file(
            root,
            SUPPORT_TIERS_MD,
            &format!(
                r#"# Support Tiers

## Claim Support Map

| Surface | Tier | Claim | Proof command | Docs | Boundary | Release lane |
| --- | --- | --- | --- | --- | --- | --- |
| Scanner-safe fixtures | {tier} | {claim} | {proof} | `docs/VERIFICATION.md` | Boundary. | `pr` |

## Explicit Non-Support
"#
            ),
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
