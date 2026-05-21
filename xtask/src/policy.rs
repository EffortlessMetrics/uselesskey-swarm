//! Effortless Metrics policy stack: no-panic, file-policy, lint-policy.
//!
//! See `docs/CLIPPY_POLICY.md`, `docs/NO_PANIC_POLICY.md`, `docs/FILE_POLICY.md`,
//! and `docs/POLICY_ALLOWLISTS.md`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use regex::Regex;
use serde::{Deserialize, Serialize};

const NO_PANIC_TOML: &str = "policy/no-panic-allowlist.toml";
const NO_PANIC_BASELINE_TOML: &str = "policy/no-panic-baseline.toml";
const NON_RUST_TOML: &str = "policy/non-rust-allowlist.toml";
const CLIPPY_LINTS_TOML: &str = "policy/clippy-lints.toml";
const CLIPPY_DEBT_TOML: &str = "policy/clippy-debt.toml";
const NEGATIVE_FIXTURES_TOML: &str = "policy/negative-fixtures.toml";
const NEGATIVE_FIXTURE_MATRIX_MD: &str = "docs/status/negative-fixture-matrix.md";
const BUNDLE_MANIFEST_SCHEMA_JSON: &str = "docs/schemas/bundle-manifest.schema.json";
const NEGATIVE_COVERAGE_SCHEMA_JSON: &str = "docs/schemas/negative-coverage.schema.json";

const TARGET_DIR: &str = "target";
const PROPOSED_DIR: &str = "target/policy-proposed";

// =============================================================================
// Glob matching (forward-slash, supports **, *, ?)
// =============================================================================

fn glob_to_regex(glob: &str) -> Regex {
    let mut re = String::from("^");
    let mut chars = glob.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    // consume optional trailing '/'
                    if chars.peek() == Some(&'/') {
                        chars.next();
                        re.push_str("(?:.*/)?");
                    } else {
                        re.push_str(".*");
                    }
                } else {
                    re.push_str("[^/]*");
                }
            }
            '?' => re.push_str("[^/]"),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                re.push('\\');
                re.push(c);
            }
            _ => re.push(c),
        }
    }
    re.push('$');
    Regex::new(&re).expect("valid generated regex")
}

fn glob_match(glob: &str, path: &str) -> bool {
    glob_to_regex(glob).is_match(path)
}

// =============================================================================
// Common helpers
// =============================================================================

fn read_toml<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    toml::from_str(&raw).with_context(|| format!("parse {path}"))
}

fn git_ls_files() -> Result<Vec<String>> {
    let out = Command::new("git")
        .args(["ls-files"])
        .output()
        .context("git ls-files")?;
    if !out.status.success() {
        bail!("git ls-files exited with {:?}", out.status);
    }
    let s = String::from_utf8(out.stdout).context("git ls-files: utf-8")?;
    Ok(s.lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect())
}

fn write_outputs(name: &str, json: &serde_json::Value, markdown: &str) -> Result<()> {
    fs::create_dir_all(TARGET_DIR).ok();
    let json_path = format!("{TARGET_DIR}/{name}.json");
    let md_path = format!("{TARGET_DIR}/{name}.md");
    fs::write(&json_path, serde_json::to_string_pretty(json)?)
        .with_context(|| format!("write {json_path}"))?;
    fs::write(&md_path, markdown).with_context(|| format!("write {md_path}"))?;
    Ok(())
}

fn today() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

// =============================================================================
// no-panic allowlist
// =============================================================================

#[derive(Debug, Deserialize)]
struct NoPanicConfig {
    #[expect(
        dead_code,
        reason = "schema validation; surfaced in policy reports later"
    )]
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    policy: NoPanicPolicy,
    #[serde(default)]
    allow: Vec<NoPanicAllow>,
}

#[derive(Debug, Default, Deserialize)]
struct NoPanicPolicy {
    // `families` is part of the schema and parsed for validation; the scanner
    // currently scans all known families.
    #[expect(
        dead_code,
        reason = "schema validation; consumed once families become tunable"
    )]
    #[serde(default)]
    families: Vec<String>,
    #[serde(default = "default_no_panic_mode")]
    mode: String,
}

fn default_no_panic_mode() -> String {
    "advisory".into()
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "schema fields preserved for surfacing in reports")]
struct NoPanicAllow {
    id: String,
    path: String,
    family: String,
    classification: String,
    owner: String,
    #[serde(default)]
    explanation: Option<String>,
    expires: String,
    selector: NoPanicSelector,
    #[serde(default)]
    last_seen: Option<NoPanicLastSeen>,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "schema fields used by the matching reducer")]
struct NoPanicSelector {
    kind: String,
    #[serde(default)]
    container: Option<String>,
    callee: String,
    #[serde(default)]
    receiver_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "advisory `last_seen` hints surface drift later")]
struct NoPanicLastSeen {
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    column: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PanicFinding {
    pub path: String,
    pub family: String,
    pub line: u32,
    pub column: u32,
    pub selector_kind: String,
    pub selector_callee: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct NoPanicBaseline {
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    generated_at: Option<String>,
    #[serde(default)]
    generated_by: Option<String>,
    #[serde(default)]
    summary: BaselineSummary,
    #[serde(default)]
    entry: Vec<BaselineEntry>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct BaselineSummary {
    #[serde(default)]
    total: usize,
    #[serde(default)]
    by_family: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BaselineKey {
    path: String,
    family: String,
    selector_kind: String,
    selector_callee: String,
    snippet: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
struct BaselineEntry {
    path: String,
    family: String,
    #[serde(default)]
    selector_kind: String,
    selector_callee: String,
    #[serde(default)]
    snippet: String,
    #[serde(default = "default_baseline_count")]
    count: usize,
}

impl BaselineEntry {
    fn from_key(key: BaselineKey, count: usize) -> Self {
        Self {
            path: key.path,
            family: key.family,
            selector_kind: key.selector_kind,
            selector_callee: key.selector_callee,
            snippet: key.snippet,
            count,
        }
    }
}

fn default_baseline_count() -> usize {
    1
}

fn baseline_key(f: &PanicFinding) -> BaselineKey {
    BaselineKey {
        path: f.path.clone(),
        family: f.family.clone(),
        selector_kind: f.selector_kind.clone(),
        selector_callee: f.selector_callee.clone(),
        snippet: f.snippet.clone(),
    }
}

fn baseline_entry_key(e: &BaselineEntry) -> BaselineKey {
    BaselineKey {
        path: e.path.clone(),
        family: e.family.clone(),
        selector_kind: e.selector_kind.clone(),
        selector_callee: e.selector_callee.clone(),
        snippet: e.snippet.clone(),
    }
}

fn baseline_counts(entries: &[BaselineEntry]) -> HashMap<BaselineKey, usize> {
    let mut counts = HashMap::new();
    for entry in entries {
        *counts.entry(baseline_entry_key(entry)).or_default() += entry.count;
    }
    counts
}

fn baseline_entries_from_findings(
    findings: &[PanicFinding],
    config: &NoPanicConfig,
) -> Vec<BaselineEntry> {
    let mut counts: BTreeMap<BaselineKey, usize> = BTreeMap::new();
    for finding in findings {
        if config
            .allow
            .iter()
            .any(|entry| entry_matches(entry, finding))
        {
            continue;
        }
        *counts.entry(baseline_key(finding)).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(key, count)| BaselineEntry::from_key(key, count))
        .collect()
}

fn new_baseline_debt<'a>(
    current_entries: &'a [BaselineEntry],
    existing_counts: &HashMap<BaselineKey, usize>,
) -> Vec<(&'a BaselineEntry, usize)> {
    current_entries
        .iter()
        .filter_map(|entry| {
            let current = entry.count;
            let existing = existing_counts
                .get(&baseline_entry_key(entry))
                .copied()
                .unwrap_or(0);
            if current > existing {
                Some((entry, current - existing))
            } else {
                None
            }
        })
        .collect()
}

pub fn check_no_panic_family() -> Result<()> {
    let config: NoPanicConfig = if Path::new(NO_PANIC_TOML).exists() {
        read_toml(NO_PANIC_TOML)?
    } else {
        bail!("missing {NO_PANIC_TOML}");
    };

    let baseline: NoPanicBaseline = if Path::new(NO_PANIC_BASELINE_TOML).exists() {
        read_toml(NO_PANIC_BASELINE_TOML)?
    } else {
        NoPanicBaseline::default()
    };
    let baseline_counts = baseline_counts(&baseline.entry);

    match config.policy.mode.as_str() {
        "advisory" | "no-new-debt" | "blocking" => {}
        other => bail!(
            "unknown no-panic policy mode `{other}` (expected advisory, no-new-debt, or blocking)"
        ),
    }

    let findings = scan_panic_findings()?;
    let mut report = NoPanicReport::default();
    let mut matched = vec![false; config.allow.len()];
    let mut unallowlisted: Vec<&PanicFinding> = Vec::new();
    let mut new_findings: Vec<&PanicFinding> = Vec::new();
    let mut baseline_used_counts: HashMap<BaselineKey, usize> = HashMap::new();
    let mut allowlisted_count = 0usize;
    let mut baselined_count = 0usize;
    let baseline_active = config.policy.mode != "blocking";

    for finding in &findings {
        let mut matched_one = false;
        for (idx, entry) in config.allow.iter().enumerate() {
            if entry_matches(entry, finding) {
                matched[idx] = true;
                matched_one = true;
                break;
            }
        }
        if matched_one {
            allowlisted_count += 1;
            continue;
        }
        let key = baseline_key(finding);
        let used = baseline_used_counts.get(&key).copied().unwrap_or(0);
        let baseline_limit = baseline_counts.get(&key).copied().unwrap_or(0);
        if baseline_active && used < baseline_limit {
            baseline_used_counts.insert(key, used + 1);
            baselined_count += 1;
            continue;
        }
        // Unallowlisted *and* not in the baseline → genuinely new debt.
        unallowlisted.push(finding);
        new_findings.push(finding);
    }

    let mut stale: Vec<&NoPanicAllow> = Vec::new();
    let mut expired: Vec<&NoPanicAllow> = Vec::new();
    let now = today();
    for (idx, entry) in config.allow.iter().enumerate() {
        if !matched[idx] {
            stale.push(entry);
        }
        if let Ok(d) = NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d") {
            if d < now {
                expired.push(entry);
            }
        } else {
            bail!(
                "no-panic allowlist entry `{}`: invalid expires date `{}` (expected YYYY-MM-DD)",
                entry.id,
                entry.expires
            );
        }
    }

    let stale_baseline_entries: Vec<&BaselineEntry> = if baseline_active {
        baseline
            .entry
            .iter()
            .filter(|e| {
                baseline_used_counts
                    .get(&baseline_entry_key(e))
                    .copied()
                    .unwrap_or(0)
                    < e.count
            })
            .collect()
    } else {
        Vec::new()
    };

    report.total_findings = findings.len();
    report.allowlisted = allowlisted_count;
    report.baselined = baselined_count;
    report.unallowlisted = unallowlisted.len();
    report.stale_entries = stale.len();
    report.expired_entries = expired.len();
    report.baseline_total = baseline.entry.len();
    report.baseline_finding_total = baseline.entry.iter().map(|entry| entry.count).sum();
    report.baseline_unique_hit = baseline_used_counts.len();
    report.baseline_stale = stale_baseline_entries.len();
    report.new_debt = new_findings.len();
    report.mode = config.policy.mode.clone();
    report.by_family = group_by_family(&findings);
    report.by_crate = group_by_crate(&findings);

    let markdown = render_no_panic_md(&report, &unallowlisted, &stale, &expired);
    write_outputs("no-panic", &serde_json::to_value(&report)?, &markdown)?;

    eprintln!(
        "no-panic: {} finding(s); {} allowlisted; {} baselined; {} new-debt; {} stale-baseline; {} expired (mode={}; baseline={}/{})",
        report.total_findings,
        report.allowlisted,
        report.baselined,
        report.new_debt,
        report.baseline_stale,
        report.expired_entries,
        report.mode,
        report.baseline_unique_hit,
        report.baseline_total,
    );

    match config.policy.mode.as_str() {
        "blocking"
            if (report.unallowlisted > 0
                || report.stale_entries > 0
                || report.expired_entries > 0) =>
        {
            bail!(
                "no-panic policy is blocking and there are {} unallowlisted, {} stale, {} expired",
                report.unallowlisted,
                report.stale_entries,
                report.expired_entries
            );
        }
        "blocking" => {}
        "no-new-debt"
            if (report.new_debt > 0 || report.stale_entries > 0 || report.expired_entries > 0) =>
        {
            if !new_findings.is_empty() {
                eprintln!("no-panic: new debt sites:");
                for f in new_findings.iter().take(20) {
                    eprintln!(
                        "  {}:{} ({} via {})",
                        f.path, f.line, f.family, f.selector_callee
                    );
                }
            }
            bail!(
                "no-panic policy is no-new-debt and there are {} new debt site(s), {} stale allowlist entries, and {} expired allowlist entries",
                report.new_debt,
                report.stale_entries,
                report.expired_entries
            );
        }
        "no-new-debt" => {}
        _ => {}
    }
    Ok(())
}

pub fn no_panic_baseline(reset: bool) -> Result<()> {
    let config: NoPanicConfig = if Path::new(NO_PANIC_TOML).exists() {
        read_toml(NO_PANIC_TOML)?
    } else {
        bail!("missing {NO_PANIC_TOML}");
    };
    let findings = scan_panic_findings()?;
    let current_entries = baseline_entries_from_findings(&findings, &config);

    let entries = if reset {
        current_entries
    } else {
        if !Path::new(NO_PANIC_BASELINE_TOML).exists() {
            bail!(
                "missing {NO_PANIC_BASELINE_TOML}; use `cargo xtask no-panic baseline --reset` for an intentional initial baseline"
            );
        }
        let existing: NoPanicBaseline = read_toml(NO_PANIC_BASELINE_TOML)?;
        let existing_counts = baseline_counts(&existing.entry);
        let new_debt = new_baseline_debt(&current_entries, &existing_counts);
        if !new_debt.is_empty() {
            eprintln!("no-panic baseline: refusing to absorb new debt:");
            for (entry, added) in new_debt.iter().take(20) {
                eprintln!(
                    "  {} ({}, {} via {}, +{}): {}",
                    entry.path,
                    entry.family,
                    entry.selector_kind,
                    entry.selector_callee,
                    added,
                    entry.snippet
                );
            }
            bail!(
                "no-panic baseline refresh would add {} new baseline entry/count change(s); remove or allowlist the new debt, or use --reset only for an intentional baseline reset",
                new_debt.len()
            );
        }

        current_entries
            .into_iter()
            .filter_map(|mut entry| {
                let key = baseline_entry_key(&entry);
                let existing_count = existing_counts.get(&key).copied()?;
                entry.count = entry.count.min(existing_count);
                (entry.count > 0).then_some(entry)
            })
            .collect()
    };

    let mut by_family: BTreeMap<String, usize> = BTreeMap::new();
    for f in &findings {
        *by_family.entry(f.family.clone()).or_default() += 1;
    }

    let baseline = NoPanicBaseline {
        schema_version: Some("1.0".into()),
        generated_at: Some(today().format("%Y-%m-%d").to_string()),
        generated_by: Some(if reset {
            "cargo xtask no-panic baseline --reset".into()
        } else {
            "cargo xtask no-panic baseline".into()
        }),
        summary: BaselineSummary {
            total: findings.len(),
            by_family,
        },
        entry: entries,
    };

    let mut buf = String::new();
    buf.push_str("# Effortless Metrics — no-panic baseline snapshot\n");
    buf.push_str("#\n");
    buf.push_str("# This file pins the panic-family findings present at the time of\n");
    buf.push_str("# generation. New findings outside this baseline are blocked when the\n");
    buf.push_str("# no-panic checker is in `mode = \"no-new-debt\"`. Move entries into\n");
    buf.push_str("# `policy/no-panic-allowlist.toml` (with owner/reason/expiry) as they\n");
    buf.push_str("# are reviewed; entries that disappear naturally are dropped on next\n");
    buf.push_str("# refresh. Normal refreshes refuse to add new entries; use --reset\n");
    buf.push_str("# only for an intentional baseline reset.\n");
    buf.push_str("#\n");
    buf.push_str("# Refresh: `cargo xtask no-panic baseline`\n");
    buf.push_str("# Reset:   `cargo xtask no-panic baseline --reset`\n\n");
    buf.push_str(&toml::to_string(&baseline).context("serialize baseline")?);

    fs::write(NO_PANIC_BASELINE_TOML, buf)
        .with_context(|| format!("write {NO_PANIC_BASELINE_TOML}"))?;
    eprintln!(
        "no-panic baseline: wrote {} ({} unique entries from {} findings; reset={})",
        NO_PANIC_BASELINE_TOML,
        baseline.entry.len(),
        findings.len(),
        reset,
    );
    Ok(())
}

pub fn no_panic_propose() -> Result<()> {
    let findings = scan_panic_findings()?;
    fs::create_dir_all(PROPOSED_DIR).ok();
    let path = format!("{PROPOSED_DIR}/no-panic-proposed-allowlist.toml");

    let mut buf = String::new();
    buf.push_str("# Proposed no-panic allowlist (review before copying into\n");
    buf.push_str("# policy/no-panic-allowlist.toml). Add owner/reason/expiry per entry.\n");
    buf.push_str("schema_version = \"0.3\"\n\n");

    let stub_expires = stub_expiry();
    for (idx, f) in findings.iter().enumerate() {
        let id = format!("panic-proposed-{idx:05}");
        buf.push_str("[[allow]]\n");
        buf.push_str(&format!("id = \"{id}\"\n"));
        buf.push_str(&format!("path = \"{}\"\n", f.path));
        buf.push_str(&format!("family = \"{}\"\n", f.family));
        buf.push_str("classification = \"test_helper\"  # FIXME: classify\n");
        buf.push_str("owner = \"FIXME\"\n");
        buf.push_str(&format!(
            "explanation = \"FIXME: explain or migrate. Snippet: {}\"\n",
            escape_toml(&f.snippet)
        ));
        buf.push_str(&format!("expires = \"{stub_expires}\"\n"));
        buf.push_str("\n[allow.selector]\n");
        buf.push_str(&format!("kind = \"{}\"\n", f.selector_kind));
        buf.push_str(&format!("callee = \"{}\"\n", f.selector_callee));
        buf.push_str("\n[allow.last_seen]\n");
        buf.push_str(&format!("line = {}\n", f.line));
        buf.push_str(&format!("column = {}\n", f.column));
        buf.push('\n');
    }

    fs::write(&path, buf).with_context(|| format!("write {path}"))?;
    eprintln!(
        "no-panic propose: wrote {} ({} candidate entries)",
        path,
        findings.len()
    );
    Ok(())
}

fn stub_expiry() -> String {
    let d = today() + chrono::Duration::days(180);
    d.format("%Y-%m-%d").to_string()
}

fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn entry_matches(entry: &NoPanicAllow, finding: &PanicFinding) -> bool {
    entry.path == finding.path
        && entry.family == finding.family
        && entry.selector.kind == finding.selector_kind
        && entry.selector.callee == finding.selector_callee
}

#[derive(Debug, Default, Serialize)]
struct NoPanicReport {
    total_findings: usize,
    /// Findings matched by an entry in `policy/no-panic-allowlist.toml`.
    allowlisted: usize,
    /// Findings absorbed by `policy/no-panic-baseline.toml`.
    baselined: usize,
    /// Findings matched neither by the allowlist nor by the baseline.
    unallowlisted: usize,
    stale_entries: usize,
    expired_entries: usize,
    /// Number of entries in the baseline file.
    baseline_total: usize,
    /// Total finding count represented by all baseline entries.
    baseline_finding_total: usize,
    /// Number of distinct baseline entries hit at least once during the scan.
    baseline_unique_hit: usize,
    /// Baseline entries with no matching finding (candidate for removal).
    baseline_stale: usize,
    /// Findings that are not in the allowlist *and* not in the baseline.
    new_debt: usize,
    mode: String,
    by_family: BTreeMap<String, usize>,
    by_crate: BTreeMap<String, usize>,
}

fn group_by_family(findings: &[PanicFinding]) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for f in findings {
        *map.entry(f.family.clone()).or_default() += 1;
    }
    map
}

fn group_by_crate(findings: &[PanicFinding]) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for f in findings {
        let krate = f.path.split('/').take(2).collect::<Vec<_>>().join("/");
        *map.entry(krate).or_default() += 1;
    }
    map
}

fn render_no_panic_md(
    report: &NoPanicReport,
    unallowlisted: &[&PanicFinding],
    stale: &[&NoPanicAllow],
    expired: &[&NoPanicAllow],
) -> String {
    let mut s = String::new();
    s.push_str("# No-panic policy report\n\n");
    s.push_str(&format!("- Mode: `{}`\n", report.mode));
    s.push_str(&format!(
        "- Total findings: **{}**\n",
        report.total_findings
    ));
    s.push_str(&format!("- Allowlisted: {}\n", report.allowlisted));
    s.push_str(&format!(
        "- Baselined: {} (across {}/{} baseline entries; {} total baseline finding slots; {} stale)\n",
        report.baselined,
        report.baseline_unique_hit,
        report.baseline_total,
        report.baseline_finding_total,
        report.baseline_stale,
    ));
    s.push_str(&format!("- New debt: **{}**\n", report.new_debt));
    s.push_str(&format!(
        "- Unallowlisted (allowlist + baseline gap): **{}**\n",
        report.unallowlisted
    ));
    s.push_str(&format!(
        "- Stale allowlist entries: {}\n",
        report.stale_entries
    ));
    s.push_str(&format!(
        "- Expired allowlist entries: {}\n\n",
        report.expired_entries
    ));

    s.push_str("## By family\n\n");
    for (k, v) in &report.by_family {
        s.push_str(&format!("- `{}`: {}\n", k, v));
    }
    s.push('\n');

    s.push_str("## By crate (top 20)\n\n");
    let mut by_crate: Vec<_> = report.by_crate.iter().collect();
    by_crate.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (k, v) in by_crate.iter().take(20) {
        s.push_str(&format!("- `{}`: {}\n", k, v));
    }
    s.push('\n');

    if !unallowlisted.is_empty() {
        s.push_str(&format!(
            "## Unallowlisted findings ({})\n\n",
            unallowlisted.len()
        ));
        s.push_str("First 50 shown.\n\n");
        for f in unallowlisted.iter().take(50) {
            s.push_str(&format!(
                "- `{}:{}:{}` — `{}` ({})\n",
                f.path, f.line, f.column, f.family, f.selector_callee
            ));
        }
        s.push('\n');
    }

    if !stale.is_empty() {
        s.push_str(&format!("## Stale allowlist entries ({})\n\n", stale.len()));
        for e in stale {
            s.push_str(&format!("- `{}` ({}, {})\n", e.id, e.path, e.family));
        }
        s.push('\n');
    }

    if !expired.is_empty() {
        s.push_str(&format!(
            "## Expired allowlist entries ({})\n\n",
            expired.len()
        ));
        for e in expired {
            s.push_str(&format!(
                "- `{}` expired {} ({}, {})\n",
                e.id, e.expires, e.path, e.family
            ));
        }
        s.push('\n');
    }

    s
}

// =============================================================================
// panic-family scanning (regex-based)
// =============================================================================

fn scan_panic_findings() -> Result<Vec<PanicFinding>> {
    let files = git_ls_files()?;
    let rust_files: Vec<&str> = files
        .iter()
        .filter(|p| p.ends_with(".rs"))
        .map(String::as_str)
        .collect();

    // Skip tests/build helpers under target, and ignore generated files.
    let unwrap_re = Regex::new(r"\.unwrap\s*\(\s*\)").expect("valid regex");
    let expect_re = Regex::new(r"\.expect\s*\(").expect("valid regex");
    let get_unwrap_re =
        Regex::new(r"\.get(?:_mut)?\s*\([^)]*\)\s*\.\s*unwrap\s*\(").expect("valid regex");
    let panic_macro_re = Regex::new(r"\bpanic!\s*\(").expect("valid regex");
    let todo_re = Regex::new(r"\btodo!\s*\(").expect("valid regex");
    let unimplemented_re = Regex::new(r"\bunimplemented!\s*\(").expect("valid regex");
    let unreachable_re = Regex::new(r"\bunreachable!\s*\(").expect("valid regex");

    let mut out = Vec::new();
    for file in &rust_files {
        let content = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for (idx, raw_line) in content.lines().enumerate() {
            // Skip pure-comment / doc lines as a fast first pass; this is not perfect
            // (block comments slip through) but is a reasonable signal-to-noise.
            let trimmed = raw_line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            // Strip the trailing `// ...` comment (if any) before regex matching, so
            // that `let x = foo(); // .unwrap()` does not produce a false positive.
            // This is a naive strip that ignores `//` inside string literals; for
            // panic-family detection in real code that is acceptable noise.
            let stripped = strip_line_comment(raw_line);
            // Replace same-line string bodies with whitespace so panic-family
            // names embedded inside string literals (e.g. `"use .unwrap()"`) are
            // not flagged. Multi-line string contents are not handled here.
            let stripped_owned = blank_string_literals_on_line(stripped);
            let line = stripped_owned.as_str();

            // get_unwrap takes precedence over plain unwrap so we don't double-count.
            if let Some(m) = get_unwrap_re.find(line) {
                out.push(PanicFinding {
                    path: file.to_string(),
                    family: "get_unwrap".into(),
                    line: (idx + 1) as u32,
                    column: (m.start() + 1) as u32,
                    selector_kind: "method_call".into(),
                    selector_callee: "unwrap".into(),
                    snippet: line.trim().to_string(),
                });
                continue;
            }
            if let Some(m) = unwrap_re.find(line) {
                out.push(PanicFinding {
                    path: file.to_string(),
                    family: "unwrap".into(),
                    line: (idx + 1) as u32,
                    column: (m.start() + 1) as u32,
                    selector_kind: "method_call".into(),
                    selector_callee: "unwrap".into(),
                    snippet: line.trim().to_string(),
                });
            }
            if let Some(m) = expect_re.find(line) {
                out.push(PanicFinding {
                    path: file.to_string(),
                    family: "expect".into(),
                    line: (idx + 1) as u32,
                    column: (m.start() + 1) as u32,
                    selector_kind: "method_call".into(),
                    selector_callee: "expect".into(),
                    snippet: line.trim().to_string(),
                });
            }
            if let Some(m) = panic_macro_re.find(line) {
                out.push(PanicFinding {
                    path: file.to_string(),
                    family: "panic_macro".into(),
                    line: (idx + 1) as u32,
                    column: (m.start() + 1) as u32,
                    selector_kind: "macro".into(),
                    selector_callee: "panic".into(),
                    snippet: line.trim().to_string(),
                });
            }
            if let Some(m) = todo_re.find(line) {
                out.push(PanicFinding {
                    path: file.to_string(),
                    family: "todo".into(),
                    line: (idx + 1) as u32,
                    column: (m.start() + 1) as u32,
                    selector_kind: "macro".into(),
                    selector_callee: "todo".into(),
                    snippet: line.trim().to_string(),
                });
            }
            if let Some(m) = unimplemented_re.find(line) {
                out.push(PanicFinding {
                    path: file.to_string(),
                    family: "unimplemented".into(),
                    line: (idx + 1) as u32,
                    column: (m.start() + 1) as u32,
                    selector_kind: "macro".into(),
                    selector_callee: "unimplemented".into(),
                    snippet: line.trim().to_string(),
                });
            }
            if let Some(m) = unreachable_re.find(line) {
                out.push(PanicFinding {
                    path: file.to_string(),
                    family: "unreachable".into(),
                    line: (idx + 1) as u32,
                    column: (m.start() + 1) as u32,
                    selector_kind: "macro".into(),
                    selector_callee: "unreachable".into(),
                    snippet: line.trim().to_string(),
                });
            }
        }
    }
    Ok(out)
}

fn strip_line_comment(line: &str) -> &str {
    if let Some(idx) = line.find("//") {
        &line[..idx]
    } else {
        line
    }
}

/// Replace the body of every same-line `"..."` string literal with spaces so
/// regex-based scanners don't pick up panic-family names embedded inside
/// strings. Escaped quotes and raw-string literals are not handled (the
/// false-positive surface they introduce is small in practice).
fn blank_string_literals_on_line(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut in_string = false;
    let mut escape = false;
    for &b in bytes {
        if in_string {
            out.push(b' ');
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
                // overwrite the closing quote we just blanked with the actual quote
                let last = out.len() - 1;
                out[last] = b'"';
            }
        } else {
            out.push(b);
            if b == b'"' {
                in_string = true;
            }
        }
    }
    // SAFETY: input was UTF-8 and we only replaced characters with ASCII spaces
    // or kept them; the result is still valid UTF-8.
    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

// =============================================================================
// non-rust file policy
// =============================================================================

#[derive(Debug, Deserialize)]
struct FilePolicyConfig {
    #[expect(
        dead_code,
        reason = "schema validation; surfaced in policy reports later"
    )]
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    policy: FilePolicySettings,
    #[serde(default)]
    allow: Vec<FilePolicyAllow>,
}

#[derive(Debug, Default, Deserialize)]
struct FilePolicySettings {
    #[serde(default = "default_allowed_extensions")]
    default_allowed_extensions: Vec<String>,
    #[serde(default = "default_allowed_filenames")]
    default_allowed_filenames: Vec<String>,
}

fn default_allowed_extensions() -> Vec<String> {
    vec!["rs".into(), "toml".into(), "md".into()]
}

fn default_allowed_filenames() -> Vec<String> {
    vec![
        "LICENSE-APACHE".into(),
        "LICENSE-MIT".into(),
        "CODEOWNERS".into(),
        ".gitignore".into(),
        ".gitattributes".into(),
        ".editorconfig".into(),
    ]
}

#[derive(Debug, Deserialize)]
struct FilePolicyAllow {
    #[serde(default)]
    glob: Option<String>,
    #[serde(default)]
    path: Option<String>,
    kind: String,
    owner: String,
    surface: String,
    classification: String,
    #[expect(dead_code, reason = "human-readable, surfaced in reports later")]
    reason: String,
    #[serde(default)]
    covered_by: Vec<String>,
    #[expect(
        dead_code,
        reason = "documents regeneration command for `generated` entries"
    )]
    #[serde(default)]
    generated_by: Option<String>,
    #[serde(default)]
    expires: Option<String>,
    #[serde(default)]
    retired: bool,
}

pub fn check_file_policy() -> Result<()> {
    let config: FilePolicyConfig = read_toml(NON_RUST_TOML)?;
    let files = git_ls_files()?;
    let now = today();

    let mut unmatched: Vec<String> = Vec::new();
    let mut matched_count = 0usize;
    let mut entry_hits = vec![0usize; config.allow.len()];
    let mut expired: Vec<&FilePolicyAllow> = Vec::new();
    let mut missing_metadata: Vec<String> = Vec::new();

    for entry in &config.allow {
        if let Some(exp) = &entry.expires {
            match NaiveDate::parse_from_str(exp, "%Y-%m-%d") {
                Ok(d) if d < now => expired.push(entry),
                Err(e) => bail!(
                    "file-policy entry kind={} owner={}: invalid expires `{}`: {}",
                    entry.kind,
                    entry.owner,
                    exp,
                    e
                ),
                _ => {}
            }
        }
        if matches!(
            entry.classification.as_str(),
            "production" | "test" | "tooling"
        ) && entry.covered_by.is_empty()
        {
            missing_metadata.push(format!(
                "kind={} surface={} owner={}: covered_by required for classification={}",
                entry.kind, entry.surface, entry.owner, entry.classification
            ));
        }
    }

    for file in &files {
        // Allowlist entries claim ownership first; default-allowed is a
        // fallback for tracked files that have no explicit owner.
        let mut hit = false;
        for (idx, entry) in config.allow.iter().enumerate() {
            if entry_matches_file(entry, file) {
                entry_hits[idx] += 1;
                hit = true;
                break;
            }
        }
        if hit {
            matched_count += 1;
            continue;
        }
        if is_default_allowed(file, &config.policy) {
            matched_count += 1;
            continue;
        }
        unmatched.push(file.clone());
    }

    let mut unused: Vec<&FilePolicyAllow> = Vec::new();
    for (idx, entry) in config.allow.iter().enumerate() {
        if entry_hits[idx] == 0 && !entry.retired {
            unused.push(entry);
        }
    }

    let report = FilePolicyReport {
        total_files: files.len(),
        matched: matched_count,
        unmatched: unmatched.len(),
        unused_entries: unused.len(),
        expired_entries: expired.len(),
        unmatched_paths: unmatched.iter().take(50).cloned().collect(),
    };

    let md = render_file_policy_md(&report, &unmatched, &unused, &expired, &missing_metadata);
    write_outputs("file-policy", &serde_json::to_value(&report)?, &md)?;

    eprintln!(
        "file-policy: {} files; {} matched; {} unmatched; {} unused; {} expired",
        report.total_files,
        report.matched,
        report.unmatched,
        report.unused_entries,
        report.expired_entries
    );

    if !missing_metadata.is_empty() {
        for m in &missing_metadata {
            eprintln!("  policy schema error: {m}");
        }
        bail!(
            "file-policy: {} entries missing required metadata",
            missing_metadata.len()
        );
    }

    if !unmatched.is_empty() || !unused.is_empty() || !expired.is_empty() {
        bail!(
            "file-policy: {} unmatched, {} unused (retire or remove), {} expired",
            unmatched.len(),
            unused.len(),
            expired.len()
        );
    }
    Ok(())
}

fn is_default_allowed(path: &str, settings: &FilePolicySettings) -> bool {
    let p = Path::new(path);
    if let Some(ext) = p.extension().and_then(|s| s.to_str())
        && settings.default_allowed_extensions.iter().any(|x| x == ext)
    {
        return true;
    }
    let basename = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
    settings
        .default_allowed_filenames
        .iter()
        .any(|x| x == basename)
}

fn entry_matches_file(entry: &FilePolicyAllow, file: &str) -> bool {
    if let Some(p) = &entry.path
        && p == file
    {
        return true;
    }
    if let Some(g) = &entry.glob
        && glob_match(g, file)
    {
        return true;
    }
    false
}

#[derive(Debug, Serialize)]
struct FilePolicyReport {
    total_files: usize,
    matched: usize,
    unmatched: usize,
    unused_entries: usize,
    expired_entries: usize,
    unmatched_paths: Vec<String>,
}

fn render_file_policy_md(
    report: &FilePolicyReport,
    unmatched: &[String],
    unused: &[&FilePolicyAllow],
    expired: &[&FilePolicyAllow],
    missing_metadata: &[String],
) -> String {
    let mut s = String::new();
    s.push_str("# File-policy report\n\n");
    s.push_str(&format!("- Tracked files: {}\n", report.total_files));
    s.push_str(&format!("- Matched: {}\n", report.matched));
    s.push_str(&format!("- Unmatched: **{}**\n", report.unmatched));
    s.push_str(&format!("- Unused entries: {}\n", report.unused_entries));
    s.push_str(&format!(
        "- Expired entries: {}\n\n",
        report.expired_entries
    ));

    if !missing_metadata.is_empty() {
        s.push_str("## Missing metadata\n\n");
        for m in missing_metadata {
            s.push_str(&format!("- {m}\n"));
        }
        s.push('\n');
    }

    if !unmatched.is_empty() {
        s.push_str(&format!("## Unmatched files ({})\n\n", unmatched.len()));
        for p in unmatched.iter().take(50) {
            s.push_str(&format!("- `{p}`\n"));
        }
        s.push('\n');
    }
    if !unused.is_empty() {
        s.push_str(&format!("## Unused entries ({})\n\n", unused.len()));
        for e in unused {
            let pat = e.path.as_deref().or(e.glob.as_deref()).unwrap_or("?");
            s.push_str(&format!(
                "- `{}` (kind={}, owner={})\n",
                pat, e.kind, e.owner
            ));
        }
        s.push('\n');
    }
    if !expired.is_empty() {
        s.push_str(&format!("## Expired entries ({})\n\n", expired.len()));
        for e in expired {
            let pat = e.path.as_deref().or(e.glob.as_deref()).unwrap_or("?");
            s.push_str(&format!(
                "- `{}` expired {}\n",
                pat,
                e.expires.as_deref().unwrap_or("?")
            ));
        }
        s.push('\n');
    }
    s
}

// =============================================================================
// negative fixture taxonomy policy
// =============================================================================

#[derive(Debug, Deserialize)]
struct NegativeFixturePolicy {
    schema_version: String,
    owner: String,
    updated: String,
    #[serde(default)]
    negative: Vec<NegativeFixtureEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct NegativeFixtureEntry {
    stable_id: String,
    family: String,
    status: String,
    #[serde(default)]
    owner_crate: Option<String>,
    #[serde(default)]
    public_surface: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    docs: Vec<String>,
    #[serde(default)]
    tests: Vec<String>,
    #[serde(default)]
    scanner_safe: Option<bool>,
    #[serde(default)]
    runtime_material: Option<bool>,
    #[serde(default)]
    bundle_exposed: Option<bool>,
    #[serde(default)]
    bundle_profiles: Vec<String>,
    #[serde(default)]
    claim: Option<String>,
    #[serde(default)]
    does_not_prove: Vec<String>,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug)]
struct NegativeFixtureMatrixRow {
    status: String,
    public_surface: String,
    bundle_exposed: String,
    proof: String,
}

#[derive(Debug, Serialize)]
struct NegativeFixturePolicyReport {
    schema_version: String,
    owner: String,
    updated: String,
    total_entries: usize,
    implemented: usize,
    accepted_planned: usize,
    deferred: usize,
    out_of_scope: usize,
    matrix_rows: usize,
    schemas_checked: Vec<String>,
    errors: Vec<String>,
}

pub fn check_negative_fixtures() -> Result<()> {
    let policy: NegativeFixturePolicy = read_toml(NEGATIVE_FIXTURES_TOML)?;
    let matrix_raw = fs::read_to_string(NEGATIVE_FIXTURE_MATRIX_MD)
        .with_context(|| format!("read {NEGATIVE_FIXTURE_MATRIX_MD}"))?;
    let (matrix_rows, duplicate_matrix_ids) = parse_negative_fixture_matrix(&matrix_raw);

    let mut errors = validate_negative_fixture_policy(&policy, &matrix_rows);
    for stable_id in duplicate_matrix_ids {
        errors.push(format!(
            "{NEGATIVE_FIXTURE_MATRIX_MD}: duplicate row for `{stable_id}`"
        ));
    }
    validate_negative_fixture_matrix(&policy, &matrix_rows, &mut errors);

    let schemas_checked = vec![
        BUNDLE_MANIFEST_SCHEMA_JSON.to_string(),
        NEGATIVE_COVERAGE_SCHEMA_JSON.to_string(),
    ];
    for schema_path in &schemas_checked {
        validate_json_schema_file(schema_path, &mut errors);
    }

    let report = NegativeFixturePolicyReport {
        schema_version: policy.schema_version.clone(),
        owner: policy.owner.clone(),
        updated: policy.updated.clone(),
        total_entries: policy.negative.len(),
        implemented: count_negative_fixture_status(&policy, "implemented"),
        accepted_planned: count_negative_fixture_status(&policy, "accepted_planned"),
        deferred: count_negative_fixture_status(&policy, "deferred"),
        out_of_scope: count_negative_fixture_status(&policy, "out_of_scope"),
        matrix_rows: matrix_rows.len(),
        schemas_checked,
        errors,
    };

    let md = render_negative_fixture_policy_md(&report);
    write_outputs("negative-fixtures", &serde_json::to_value(&report)?, &md)?;

    eprintln!(
        "negative-fixtures: {} entries; {} implemented; {} matrix rows; {} errors",
        report.total_entries,
        report.implemented,
        report.matrix_rows,
        report.errors.len()
    );

    if !report.errors.is_empty() {
        for error in report.errors.iter().take(20) {
            eprintln!("  negative-fixtures error: {error}");
        }
        bail!(
            "negative-fixtures: {} contract drift error(s)",
            report.errors.len()
        );
    }

    Ok(())
}

fn validate_negative_fixture_policy(
    policy: &NegativeFixturePolicy,
    matrix_rows: &BTreeMap<String, NegativeFixtureMatrixRow>,
) -> Vec<String> {
    let mut errors = Vec::new();
    if policy.schema_version != "1.0" {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: schema_version must be `1.0`, got `{}`",
            policy.schema_version
        ));
    }
    if policy.owner.trim().is_empty() {
        errors.push(format!("{NEGATIVE_FIXTURES_TOML}: owner is required"));
    }
    if NaiveDate::parse_from_str(&policy.updated, "%Y-%m-%d").is_err() {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: updated must be ISO date YYYY-MM-DD, got `{}`",
            policy.updated
        ));
    }
    if policy.negative.is_empty() {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: at least one [[negative]] entry is required"
        ));
    }

    let mut seen = BTreeSet::new();
    for entry in &policy.negative {
        validate_negative_fixture_entry(entry, &mut errors);
        if !seen.insert(entry.stable_id.clone()) {
            errors.push(format!(
                "{NEGATIVE_FIXTURES_TOML}: duplicate stable_id `{}`",
                entry.stable_id
            ));
        }
        if !matrix_rows.contains_key(&entry.stable_id) {
            errors.push(format!(
                "{NEGATIVE_FIXTURE_MATRIX_MD}: missing row for `{}`",
                entry.stable_id
            ));
        }
    }

    errors
}

fn validate_negative_fixture_entry(entry: &NegativeFixtureEntry, errors: &mut Vec<String>) {
    if !is_valid_negative_stable_id(&entry.stable_id) {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: stable_id `{}` must be lower snake_case and start with a letter",
            entry.stable_id
        ));
    }
    if entry.family.trim().is_empty() {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{}` family is required",
            entry.stable_id
        ));
    }
    if !is_valid_negative_status(&entry.status) {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{}` has invalid status `{}`",
            entry.stable_id, entry.status
        ));
    }
    for alias in &entry.aliases {
        if alias.trim().is_empty() {
            errors.push(format!(
                "{NEGATIVE_FIXTURES_TOML}: `{}` has an empty alias",
                entry.stable_id
            ));
        }
    }
    if let Some(claim) = &entry.claim
        && claim.trim().is_empty()
    {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{}` claim must not be empty when present",
            entry.stable_id
        ));
    }
    if entry.scanner_safe.is_none() {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{}` scanner_safe is required",
            entry.stable_id
        ));
    }
    if entry.runtime_material.is_none() {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{}` runtime_material is required",
            entry.stable_id
        ));
    }
    if entry.bundle_exposed.is_none() {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{}` bundle_exposed is required",
            entry.stable_id
        ));
    }
    if entry.bundle_exposed == Some(true) && entry.bundle_profiles.is_empty() {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{}` bundle_profiles is required when bundle_exposed=true",
            entry.stable_id
        ));
    }

    if entry.status == "implemented" {
        validate_required_text(
            entry.owner_crate.as_deref(),
            &entry.stable_id,
            "owner_crate",
            errors,
        );
        validate_required_text(
            entry.public_surface.as_deref(),
            &entry.stable_id,
            "public_surface",
            errors,
        );
        validate_required_vec(&entry.docs, &entry.stable_id, "docs", errors);
        validate_required_vec(&entry.tests, &entry.stable_id, "tests", errors);
        validate_required_vec(
            &entry.does_not_prove,
            &entry.stable_id,
            "does_not_prove",
            errors,
        );
        for doc in &entry.docs {
            if !Path::new(doc).exists() {
                errors.push(format!(
                    "{NEGATIVE_FIXTURES_TOML}: `{}` docs path `{doc}` does not exist",
                    entry.stable_id
                ));
            }
        }
        for command in &entry.tests {
            if !(command.starts_with("cargo test ") || command.starts_with("cargo xtask ")) {
                errors.push(format!(
                    "{NEGATIVE_FIXTURES_TOML}: `{}` test command `{command}` must start with `cargo test` or `cargo xtask`",
                    entry.stable_id
                ));
            }
        }
    } else {
        validate_required_text(entry.reason.as_deref(), &entry.stable_id, "reason", errors);
    }
}

fn validate_required_text(
    value: Option<&str>,
    stable_id: &str,
    field: &str,
    errors: &mut Vec<String>,
) {
    if value.is_none_or(|value| value.trim().is_empty()) {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{stable_id}` {field} is required"
        ));
    }
}

fn validate_required_vec(
    values: &[String],
    stable_id: &str,
    field: &str,
    errors: &mut Vec<String>,
) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        errors.push(format!(
            "{NEGATIVE_FIXTURES_TOML}: `{stable_id}` {field} must contain at least one non-empty value"
        ));
    }
}

fn validate_negative_fixture_matrix(
    policy: &NegativeFixturePolicy,
    matrix_rows: &BTreeMap<String, NegativeFixtureMatrixRow>,
    errors: &mut Vec<String>,
) {
    let ledger_ids = policy
        .negative
        .iter()
        .map(|entry| entry.stable_id.as_str())
        .collect::<BTreeSet<_>>();

    for matrix_id in matrix_rows.keys() {
        if !ledger_ids.contains(matrix_id.as_str()) {
            errors.push(format!(
                "{NEGATIVE_FIXTURE_MATRIX_MD}: row `{matrix_id}` is not present in {NEGATIVE_FIXTURES_TOML}"
            ));
        }
    }

    for entry in &policy.negative {
        let Some(row) = matrix_rows.get(&entry.stable_id) else {
            continue;
        };
        if row.status != entry.status {
            errors.push(format!(
                "{NEGATIVE_FIXTURE_MATRIX_MD}: `{}` status `{}` does not match ledger `{}`",
                entry.stable_id, row.status, entry.status
            ));
        }
        let expected_bundle = expected_negative_fixture_bundle_cell(entry);
        if row.bundle_exposed != expected_bundle {
            errors.push(format!(
                "{NEGATIVE_FIXTURE_MATRIX_MD}: `{}` bundle cell `{}` does not match ledger `{}`",
                entry.stable_id, row.bundle_exposed, expected_bundle
            ));
        }
        if entry.status == "implemented" {
            let surface = entry.public_surface.as_deref().unwrap_or_default();
            if row.public_surface != surface {
                errors.push(format!(
                    "{NEGATIVE_FIXTURE_MATRIX_MD}: `{}` public surface `{}` does not match ledger `{}`",
                    entry.stable_id, row.public_surface, surface
                ));
            }
            if !entry.tests.contains(&row.proof) {
                errors.push(format!(
                    "{NEGATIVE_FIXTURE_MATRIX_MD}: `{}` proof `{}` is not one of the ledger tests",
                    entry.stable_id, row.proof
                ));
            }
        }
    }
}

fn expected_negative_fixture_bundle_cell(entry: &NegativeFixtureEntry) -> String {
    if entry.bundle_exposed == Some(true) {
        entry.bundle_profiles.join(", ")
    } else {
        "no".to_string()
    }
}

fn validate_json_schema_file(path: &str, errors: &mut Vec<String>) {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) => {
            errors.push(format!("{path}: failed to read schema: {error}"));
            return;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(format!("{path}: failed to parse JSON schema: {error}"));
            return;
        }
    };
    if parsed
        .get("$schema")
        .and_then(|value| value.as_str())
        .is_none()
    {
        errors.push(format!("{path}: `$schema` is required"));
    }
    if parsed
        .get("title")
        .and_then(|value| value.as_str())
        .is_none()
    {
        errors.push(format!("{path}: `title` is required"));
    }
    if parsed.get("type").and_then(|value| value.as_str()) != Some("object") {
        errors.push(format!("{path}: root `type` must be `object`"));
    }
}

fn parse_negative_fixture_matrix(
    raw: &str,
) -> (BTreeMap<String, NegativeFixtureMatrixRow>, Vec<String>) {
    let mut rows = BTreeMap::new();
    let mut duplicate_ids = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if !line.starts_with('|') || !line.contains('`') {
            continue;
        }
        let columns = line
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if columns.len() < 5 {
            continue;
        }
        let Some(stable_id) = first_markdown_code(columns[0]) else {
            continue;
        };
        let Some(status) = first_markdown_code(columns[1]) else {
            continue;
        };
        let public_surface = first_markdown_code(columns[2]).unwrap_or_else(|| columns[2].into());
        let bundle_exposed = first_markdown_code(columns[3]).unwrap_or_else(|| columns[3].into());
        let proof = first_markdown_code(columns[4]).unwrap_or_else(|| columns[4].into());
        let row = NegativeFixtureMatrixRow {
            status,
            public_surface,
            bundle_exposed,
            proof,
        };
        if rows.insert(stable_id.clone(), row).is_some() {
            duplicate_ids.push(stable_id);
        }
    }
    (rows, duplicate_ids)
}

fn first_markdown_code(cell: &str) -> Option<String> {
    let start = cell.find('`')?;
    let rest = &cell[start + 1..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

fn is_valid_negative_stable_id(stable_id: &str) -> bool {
    let mut chars = stable_id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    let mut previous_underscore = false;
    for ch in chars {
        match ch {
            'a'..='z' | '0'..='9' => previous_underscore = false,
            '_' if !previous_underscore => previous_underscore = true,
            _ => return false,
        }
    }
    !previous_underscore
}

fn is_valid_negative_status(status: &str) -> bool {
    matches!(
        status,
        "implemented" | "accepted_planned" | "deferred" | "out_of_scope"
    )
}

fn count_negative_fixture_status(policy: &NegativeFixturePolicy, status: &str) -> usize {
    policy
        .negative
        .iter()
        .filter(|entry| entry.status == status)
        .count()
}

fn render_negative_fixture_policy_md(report: &NegativeFixturePolicyReport) -> String {
    let mut s = String::new();
    s.push_str("# Negative fixture policy report\n\n");
    s.push_str(&format!("- Schema version: `{}`\n", report.schema_version));
    s.push_str(&format!("- Owner: `{}`\n", report.owner));
    s.push_str(&format!("- Updated: `{}`\n", report.updated));
    s.push_str(&format!("- Entries: {}\n", report.total_entries));
    s.push_str(&format!("- Implemented: {}\n", report.implemented));
    s.push_str(&format!(
        "- Accepted/planned: {}\n",
        report.accepted_planned
    ));
    s.push_str(&format!("- Deferred: {}\n", report.deferred));
    s.push_str(&format!("- Out of scope: {}\n", report.out_of_scope));
    s.push_str(&format!("- Matrix rows: {}\n", report.matrix_rows));
    s.push_str(&format!("- Errors: {}\n\n", report.errors.len()));
    s.push_str("## Schemas Checked\n\n");
    for schema in &report.schemas_checked {
        s.push_str(&format!("- `{schema}`\n"));
    }
    s.push('\n');
    if !report.errors.is_empty() {
        s.push_str("## Errors\n\n");
        for error in &report.errors {
            s.push_str(&format!("- {error}\n"));
        }
        s.push('\n');
    }
    s
}

// =============================================================================
// lint-policy checker
// =============================================================================

#[derive(Debug, Deserialize)]
struct LintPolicy {
    msrv: String,
    #[expect(dead_code, reason = "schema validation; surfaced in reports later")]
    #[serde(default)]
    policy: LintPolicySettings,
    #[serde(default)]
    planned: Vec<PlannedLint>,
    #[serde(default)]
    forbidden_carveouts: ForbiddenCarveouts,
}

#[derive(Debug, Default, Deserialize)]
#[expect(dead_code, reason = "schema validation; surfaced in reports later")]
struct LintPolicySettings {
    #[serde(default)]
    suppression_style: Option<String>,
    #[serde(default)]
    allow_test_carveouts: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "schema validation; surfaced in reports later")]
struct PlannedLint {
    name: String,
    level: String,
    activate_when_msrv: String,
    reason: String,
}

#[derive(Debug, Default, Deserialize)]
struct ForbiddenCarveouts {
    #[serde(default)]
    keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClippyDebt {
    #[expect(
        dead_code,
        reason = "schema validation; surfaced in policy reports later"
    )]
    schema_version: Option<String>,
    #[serde(default)]
    debt: Vec<DebtEntry>,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "schema fields surfaced in policy reports later")]
struct DebtEntry {
    id: String,
    lint: String,
    scope: String,
    owner: String,
    reason: String,
    expires: String,
}

pub fn check_lint_policy() -> Result<()> {
    let lp: LintPolicy = read_toml(CLIPPY_LINTS_TOML)?;
    let mut errors: Vec<String> = Vec::new();

    // 1. MSRV alignment.
    let root_cargo = fs::read_to_string("Cargo.toml").context("read Cargo.toml")?;
    let root_msrv = parse_workspace_rust_version(&root_cargo);
    if let Some(rv) = &root_msrv {
        if rv != &lp.msrv {
            errors.push(format!(
                "MSRV mismatch: workspace `rust-version = \"{}\"` != policy msrv `\"{}\"`",
                rv, lp.msrv
            ));
        }
    } else {
        errors.push("could not find `[workspace.package].rust-version` in Cargo.toml".into());
    }

    // 2. clippy.toml carveouts.
    if let Ok(c) = fs::read_to_string("clippy.toml") {
        for key in &lp.forbidden_carveouts.keys {
            if c.contains(key) {
                errors.push(format!("clippy.toml contains forbidden key `{key}`"));
            }
        }
    }

    // 3. Every member crate has [lints] workspace = true.
    let members = list_workspace_members(&root_cargo);
    for m in &members {
        let cargo = format!("{m}/Cargo.toml");
        let raw = match fs::read_to_string(&cargo) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if !has_workspace_lints(&raw) {
            errors.push(format!("{cargo}: missing `[lints]\\nworkspace = true`"));
        }
    }

    // 4. Planned lints not yet activated.
    let active = parse_active_lints(&root_cargo);
    for pl in &lp.planned {
        if !msrv_reached(&lp.msrv, &pl.activate_when_msrv) && active.iter().any(|n| n == &pl.name) {
            errors.push(format!(
                "planned lint `{}` activated before MSRV {} (current {})",
                pl.name, pl.activate_when_msrv, lp.msrv
            ));
        }
    }

    // 5. clippy-debt entries valid.
    if Path::new(CLIPPY_DEBT_TOML).exists() {
        let debt: ClippyDebt = read_toml(CLIPPY_DEBT_TOML)?;
        let now = today();
        for e in &debt.debt {
            match NaiveDate::parse_from_str(&e.expires, "%Y-%m-%d") {
                Ok(d) if d < now => {
                    errors.push(format!("clippy-debt `{}` expired {}", e.id, e.expires))
                }
                Err(err) => errors.push(format!(
                    "clippy-debt `{}` invalid expires `{}`: {}",
                    e.id, e.expires, err
                )),
                _ => {}
            }
        }
    }

    // 6. Bare `#[allow(...)]` is a hard error: every suppression must carry
    //    `reason = "..."`. This matches the shape Clippy
    //    `allow_attributes_without_reason` will flag once it is promoted to
    //    `deny` in Stage C.
    let bare_allow = scan_bare_allow_in_crates(&members)?;
    let bare_allow_total: usize = bare_allow.iter().map(|(_, n)| *n).sum();
    for (file, count) in bare_allow.iter().take(20) {
        errors.push(format!(
            "{file}: {count} bare `#[allow(...)]` attribute(s); use `#[allow(..., reason = \"...\")]` (or `#[expect]`)"
        ));
    }
    if bare_allow.len() > 20 {
        errors.push(format!(
            "... and {} more files with bare `#[allow]`",
            bare_allow.len() - 20
        ));
    }

    let report = LintPolicyReport {
        msrv: lp.msrv.clone(),
        members: members.len(),
        errors: errors.clone(),
        bare_allow_files: bare_allow.len(),
        bare_allow_total,
        bare_allow_hits: bare_allow
            .iter()
            .map(|(f, n)| BareAllowHit {
                file: f.clone(),
                count: *n,
            })
            .collect(),
    };
    let md = render_lint_policy_md(&report);
    write_outputs("lint-policy", &serde_json::to_value(&report)?, &md)?;

    eprintln!(
        "lint-policy: msrv={} members={} errors={}",
        lp.msrv,
        members.len(),
        errors.len()
    );

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("  {e}");
        }
        bail!("lint-policy: {} error(s)", errors.len());
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct LintPolicyReport {
    msrv: String,
    members: usize,
    errors: Vec<String>,
    bare_allow_files: usize,
    bare_allow_total: usize,
    bare_allow_hits: Vec<BareAllowHit>,
}

#[derive(Debug, Serialize)]
struct BareAllowHit {
    file: String,
    count: usize,
}

fn render_lint_policy_md(report: &LintPolicyReport) -> String {
    let mut s = String::new();
    s.push_str("# Lint-policy report\n\n");
    s.push_str(&format!("- MSRV: `{}`\n", report.msrv));
    s.push_str(&format!("- Workspace members: {}\n", report.members));
    s.push_str(&format!("- Errors: **{}**\n", report.errors.len()));
    s.push_str(&format!(
        "- Bare `#[allow(...)]` (blocking): {} attribute(s) across {} file(s)\n\n",
        report.bare_allow_total, report.bare_allow_files,
    ));
    if !report.errors.is_empty() {
        s.push_str("## Errors\n\n");
        for e in &report.errors {
            s.push_str(&format!("- {e}\n"));
        }
        s.push('\n');
    }
    if !report.bare_allow_hits.is_empty() {
        s.push_str("## Bare-allow sites\n\n");
        for hit in &report.bare_allow_hits {
            s.push_str(&format!("- `{}`: {}\n", hit.file, hit.count));
        }
    }
    s
}

fn parse_workspace_rust_version(cargo: &str) -> Option<String> {
    let in_pkg = cargo
        .split("[workspace.package]")
        .nth(1)
        .unwrap_or("")
        .split('\n');
    for line in in_pkg {
        let l = line.trim();
        if l.starts_with('[') {
            break;
        }
        if let Some(rest) = l.strip_prefix("rust-version") {
            // form: rust-version = "1.92"
            if let Some(eq) = rest.find('=') {
                let val = rest[eq + 1..].trim().trim_matches('"');
                return Some(val.to_string());
            }
        }
    }
    None
}

fn list_workspace_members(cargo: &str) -> Vec<String> {
    let mut out = Vec::new();
    let after = match cargo.split("members").nth(1) {
        Some(s) => s,
        None => return out,
    };
    let body = match after.split_once('[') {
        Some((_, b)) => b,
        None => return out,
    };
    let body = match body.split_once(']') {
        Some((b, _)) => b,
        None => return out,
    };
    for line in body.lines() {
        let l = line.trim().trim_end_matches(',').trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let l = l.trim_matches('"');
        out.push(l.to_string());
    }
    out
}

fn has_workspace_lints(cargo: &str) -> bool {
    // Match `[lints]` followed by `workspace = true` (within a few lines).
    let needle = "[lints]";
    let idx = match cargo.find(needle) {
        Some(i) => i + needle.len(),
        None => return false,
    };
    cargo[idx..]
        .lines()
        .take(8)
        .any(|l| l.trim() == "workspace = true")
}

fn parse_active_lints(cargo: &str) -> Vec<String> {
    // Collect lint keys under [workspace.lints.*] sections. Only keys that
    // appear before the next top-level `[` are part of the section.
    let mut active = Vec::new();
    for section in ["[workspace.lints.rust]", "[workspace.lints.clippy]"] {
        if let Some(rest) = cargo.split(section).nth(1) {
            let group = rest.split("\n[").next().unwrap_or("");
            let prefix = if section.ends_with("clippy]") {
                "clippy::"
            } else {
                ""
            };
            for line in group.lines() {
                let l = line.trim();
                if l.is_empty() || l.starts_with('#') {
                    continue;
                }
                if let Some(eq) = l.find('=') {
                    let k = l[..eq].trim();
                    if !k.is_empty() {
                        active.push(format!("{prefix}{k}"));
                    }
                }
            }
        }
    }
    active
}

fn msrv_reached(current: &str, target: &str) -> bool {
    fn parts(s: &str) -> Vec<u32> {
        s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect()
    }
    let c = parts(current);
    let t = parts(target);
    let n = c.len().max(t.len());
    for i in 0..n {
        let ci = c.get(i).copied().unwrap_or(0);
        let ti = t.get(i).copied().unwrap_or(0);
        if ci > ti {
            return true;
        }
        if ci < ti {
            return false;
        }
    }
    true
}

fn scan_bare_allow_in_crates(_members: &[String]) -> Result<Vec<(String, usize)>> {
    let files = git_ls_files()?;
    // A bare `#[allow(...)]` is one with no `reason = "..."` clause. Suppressions
    // with a `reason` are policy-compliant (matches the shape
    // `clippy::allow_attributes_without_reason` flags).
    let mut hits: Vec<(String, usize)> = Vec::new();
    for f in files {
        if !f.ends_with(".rs") {
            continue;
        }
        let s = match fs::read_to_string(&f) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let count = count_bare_allow_attributes(&s);
        if count > 0 {
            hits.push((f, count));
        }
    }
    Ok(hits)
}

fn count_bare_allow_attributes(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if starts_with(bytes, i, b"//") {
            i = skip_line_comment(bytes, i);
            continue;
        }
        if starts_with(bytes, i, b"/*") {
            i = skip_block_comment(bytes, i);
            continue;
        }
        if let Some(next) = skip_raw_string(bytes, i) {
            i = next;
            continue;
        }
        if let Some(next) = skip_quoted_string(bytes, i) {
            i = next;
            continue;
        }
        if let Some((body_start, body_end, attr_end)) = parse_allow_attribute(bytes, i) {
            if !allow_body_has_reason(&bytes[body_start..body_end]) {
                count += 1;
            }
            i = attr_end;
            continue;
        }
        i += 1;
    }

    count
}

fn parse_allow_attribute(bytes: &[u8], i: usize) -> Option<(usize, usize, usize)> {
    if !starts_with(bytes, i, b"#[allow") {
        return None;
    }
    let mut j = i + b"#[allow".len();
    j = skip_ascii_ws(bytes, j);
    if bytes.get(j) != Some(&b'(') {
        return None;
    }

    let body_start = j + 1;
    let mut depth = 1usize;
    let mut k = body_start;
    while k < bytes.len() {
        if starts_with(bytes, k, b"//") {
            k = skip_line_comment(bytes, k);
            continue;
        }
        if starts_with(bytes, k, b"/*") {
            k = skip_block_comment(bytes, k);
            continue;
        }
        if let Some(next) = skip_raw_string(bytes, k) {
            k = next;
            continue;
        }
        if let Some(next) = skip_quoted_string(bytes, k) {
            k = next;
            continue;
        }

        match bytes[k] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let attr_end = skip_ascii_ws(bytes, k + 1);
                    return (bytes.get(attr_end) == Some(&b']')).then_some((
                        body_start,
                        k,
                        attr_end + 1,
                    ));
                }
            }
            _ => {}
        }
        k += 1;
    }
    None
}

fn allow_body_has_reason(body: &[u8]) -> bool {
    body.windows(b"reason".len()).enumerate().any(|(idx, w)| {
        let after_reason = idx + b"reason".len();
        w == b"reason"
            && idx
                .checked_sub(1)
                .and_then(|before| body.get(before))
                .is_none_or(|b| !is_ident_byte(*b))
            && body.get(after_reason).is_none_or(|b| !is_ident_byte(*b))
            && body.get(skip_ascii_ws(body, after_reason)) == Some(&b'=')
    })
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn starts_with(bytes: &[u8], i: usize, needle: &[u8]) -> bool {
    bytes.get(i..i.saturating_add(needle.len())) == Some(needle)
}

fn skip_ascii_ws(bytes: &[u8], mut i: usize) -> usize {
    while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
        i += 1;
    }
    i
}

fn skip_line_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

fn skip_block_comment(bytes: &[u8], mut i: usize) -> usize {
    let mut depth = 0usize;
    while i < bytes.len() {
        if starts_with(bytes, i, b"/*") {
            depth += 1;
            i += 2;
            continue;
        }
        if starts_with(bytes, i, b"*/") {
            depth = depth.saturating_sub(1);
            i += 2;
            if depth == 0 {
                break;
            }
            continue;
        }
        i += 1;
    }
    i
}

fn skip_quoted_string(bytes: &[u8], i: usize) -> Option<usize> {
    let quote = match bytes.get(i) {
        Some(b'"') => i,
        Some(b'b' | b'c') if bytes.get(i + 1) == Some(&b'"') => i + 1,
        _ => return None,
    };

    let mut j = quote + 1;
    let mut escaped = false;
    while j < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[j] == b'\\' {
            escaped = true;
        } else if bytes[j] == b'"' {
            return Some(j + 1);
        }
        j += 1;
    }
    Some(bytes.len())
}

fn skip_raw_string(bytes: &[u8], i: usize) -> Option<usize> {
    let mut r = i;
    if bytes.get(r) == Some(&b'b') {
        r += 1;
    }
    if bytes.get(r) != Some(&b'r') {
        return None;
    }

    let mut hash_count = 0usize;
    let mut quote = r + 1;
    while bytes.get(quote) == Some(&b'#') {
        hash_count += 1;
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return None;
    }

    let mut j = quote + 1;
    while j < bytes.len() {
        if bytes[j] == b'"'
            && j + 1 + hash_count <= bytes.len()
            && bytes[j + 1..j + 1 + hash_count].iter().all(|b| *b == b'#')
        {
            return Some(j + 1 + hash_count);
        }
        j += 1;
    }
    Some(bytes.len())
}

// =============================================================================
// aggregate report
// =============================================================================

pub fn policy_report() -> Result<()> {
    let mut summary: BTreeMap<&'static str, serde_json::Value> = BTreeMap::new();
    let mut failures: Vec<String> = Vec::new();
    for (name, run) in [
        ("no-panic", check_no_panic_family()),
        ("file-policy", check_file_policy()),
        ("negative-fixtures", check_negative_fixtures()),
        ("lint-policy", check_lint_policy()),
    ] {
        if let Err(e) = &run {
            failures.push(format!("{name}: {e}"));
        }
        summary.insert(
            name,
            serde_json::json!({
                "ok": run.is_ok(),
                "error": run.err().map(|e| e.to_string()),
            }),
        );
    }

    let summary = serde_json::Value::Object(
        summary
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    );
    fs::create_dir_all(TARGET_DIR).ok();
    fs::write(
        format!("{TARGET_DIR}/policy-report.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    let mut md = String::new();
    md.push_str("# Policy report (aggregate)\n\n");
    let obj = summary
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("policy-report summary was not an object"))?;
    for (k, v) in obj {
        let ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
        let symbol = if ok { "OK" } else { "FAIL" };
        md.push_str(&format!("## {k} — {symbol}\n\n"));
        if !ok && let Some(err) = v.get("error").and_then(|x| x.as_str()) {
            md.push_str(&format!("Error: `{err}`\n\n"));
        }
        md.push_str(&format!("See `target/{k}.md` for the full report.\n\n"));
    }
    fs::write(format!("{TARGET_DIR}/policy-report.md"), md)?;

    eprintln!("policy-report: target/policy-report.{{md,json}}");
    if !failures.is_empty() {
        bail!("policy-report: {} check(s) failed", failures.len());
    }
    Ok(())
}

// =============================================================================
// tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_double_star_matches_nested() {
        assert!(glob_match("crates/**/Cargo.toml", "crates/a/b/Cargo.toml"));
        assert!(glob_match("crates/**/Cargo.toml", "crates/a/Cargo.toml"));
        assert!(!glob_match("crates/**/Cargo.toml", "Cargo.toml"));
    }

    #[test]
    fn glob_single_star_only_path_segment() {
        assert!(glob_match("crates/*/Cargo.toml", "crates/a/Cargo.toml"));
        assert!(!glob_match("crates/*/Cargo.toml", "crates/a/b/Cargo.toml"));
    }

    #[test]
    fn glob_extension_matches() {
        assert!(glob_match(
            ".github/workflows/*.yml",
            ".github/workflows/ci.yml"
        ));
        assert!(!glob_match(
            ".github/workflows/*.yml",
            ".github/dependabot.yml"
        ));
    }

    #[test]
    fn msrv_reached_compares_versions() {
        assert!(msrv_reached("1.94", "1.94"));
        assert!(msrv_reached("1.95", "1.94"));
        assert!(!msrv_reached("1.92", "1.94"));
        assert!(!msrv_reached("1.93", "1.94"));
    }

    #[test]
    fn negative_fixture_stable_ids_are_lower_snake_case() {
        assert!(is_valid_negative_stable_id("jwt_missing_kid"));
        assert!(is_valid_negative_stable_id("x509_not_yet_valid_leaf"));
        assert!(!is_valid_negative_stable_id("JwtMissingKid"));
        assert!(!is_valid_negative_stable_id("jwt-missing-kid"));
        assert!(!is_valid_negative_stable_id("jwt_missing_kid_"));
    }

    #[test]
    fn negative_fixture_matrix_parser_reads_contract_rows() -> Result<()> {
        let (rows, duplicate_ids) = parse_negative_fixture_matrix(
            "| Stable ID | Status | Public surface | Bundle exposed | Proof |\n\
             | --- | --- | --- | --- | --- |\n\
             | `jwt_missing_kid` | `implemented` | `NegativeToken::MissingKid` | no | `cargo test -p uselesskey-token --all-features` |\n",
        );
        assert!(duplicate_ids.is_empty());
        let row = rows
            .get("jwt_missing_kid")
            .ok_or_else(|| anyhow::anyhow!("matrix row"))?;
        assert_eq!(row.status, "implemented");
        assert_eq!(row.public_surface, "NegativeToken::MissingKid");
        assert_eq!(row.bundle_exposed, "no");
        assert_eq!(row.proof, "cargo test -p uselesskey-token --all-features");
        Ok(())
    }

    #[test]
    fn negative_fixture_matrix_parser_reports_duplicate_rows() {
        let (_rows, duplicate_ids) = parse_negative_fixture_matrix(
            "| Stable ID | Status | Public surface | Bundle exposed | Proof |\n\
             | --- | --- | --- | --- | --- |\n\
             | `jwt_missing_kid` | `implemented` | `NegativeToken::MissingKid` | no | `cargo test -p uselesskey-token --all-features` |\n\
             | `jwt_missing_kid` | `implemented` | `NegativeToken::MissingKid` | no | `cargo test -p uselesskey-token --all-features` |\n",
        );
        assert_eq!(duplicate_ids, vec!["jwt_missing_kid"]);
    }

    #[test]
    fn implemented_negative_fixture_requires_public_contract_fields() {
        let entry = NegativeFixtureEntry {
            stable_id: "jwt_missing_kid".into(),
            family: "jwt_token".into(),
            status: "implemented".into(),
            scanner_safe: Some(true),
            runtime_material: Some(false),
            bundle_exposed: Some(false),
            ..NegativeFixtureEntry::default()
        };
        let mut errors = Vec::new();
        validate_negative_fixture_entry(&entry, &mut errors);
        assert!(errors.iter().any(|error| error.contains("owner_crate")));
        assert!(errors.iter().any(|error| error.contains("public_surface")));
        assert!(errors.iter().any(|error| error.contains("docs")));
        assert!(errors.iter().any(|error| error.contains("tests")));
        assert!(errors.iter().any(|error| error.contains("does_not_prove")));
    }

    #[test]
    fn workspace_lints_detection_works() {
        let cargo = "[lints]\nworkspace = true\n";
        assert!(has_workspace_lints(cargo));
        let cargo2 = "[lints]\n# no workspace\n";
        assert!(!has_workspace_lints(cargo2));
    }

    #[test]
    fn glob_escapes_dots_and_dashes_correctly() {
        // `.` in glob is a literal dot, not a regex any-char.
        assert!(!glob_match("foo.yml", "fooXyml"));
        assert!(glob_match("foo.yml", "foo.yml"));
    }

    #[test]
    fn glob_double_star_at_root_matches_anywhere() {
        assert!(glob_match("**/snapshots/**/*.snap", "x/snapshots/a/b.snap"));
        assert!(glob_match("**/snapshots/**/*.snap", "snapshots/a.snap"));
    }

    #[test]
    fn strip_line_comment_drops_trailing_comment() {
        assert_eq!(strip_line_comment("let x = 1; // .unwrap()"), "let x = 1; ");
        assert_eq!(strip_line_comment("plain code"), "plain code");
        assert_eq!(strip_line_comment("// only a comment"), "");
    }

    #[test]
    fn parse_workspace_rust_version_finds_value() {
        let cargo = "[workspace.package]\nrust-version = \"1.92\"\nedition = \"2024\"\n";
        assert_eq!(parse_workspace_rust_version(cargo).as_deref(), Some("1.92"));
    }

    #[test]
    fn parse_workspace_rust_version_returns_none_when_missing() {
        let cargo = "[workspace.package]\nedition = \"2024\"\n";
        assert!(parse_workspace_rust_version(cargo).is_none());
    }

    #[test]
    fn parse_active_lints_collects_from_clippy_section() {
        let cargo = "[workspace.lints.rust]\nfoo = \"deny\"\n\n[workspace.lints.clippy]\ndbg_macro = \"deny\"\n";
        let active = parse_active_lints(cargo);
        assert!(active.iter().any(|l| l == "foo"));
        assert!(active.iter().any(|l| l == "clippy::dbg_macro"));
    }

    #[test]
    fn bare_allow_counter_detects_single_line_and_multiline() {
        let source = r#"
#[allow(dead_code)]
fn single_line() {}

#[allow(
    dead_code
)]
fn multi_line() {}
"#;
        assert_eq!(count_bare_allow_attributes(source), 2);
    }

    #[test]
    fn bare_allow_counter_accepts_reasoned_suppressions() {
        let source = r#"
#[allow(dead_code, reason = "documented exception")]
fn single_line() {}

#[allow(
    dead_code,
    reason = "documented exception"
)]
fn multi_line() {}
"#;
        assert_eq!(count_bare_allow_attributes(source), 0);
    }

    #[test]
    fn bare_allow_counter_requires_reason_clause() {
        let source = r#"
#[allow(dead_code_reason)]
fn lint_name_contains_reason() {}

#[allow(dead_code /* reason */)]
fn comment_mentions_reason() {}
"#;
        assert_eq!(count_bare_allow_attributes(source), 2);
    }

    #[test]
    fn bare_allow_counter_ignores_comments_and_strings() {
        let source = r##"
// #[allow(dead_code)]
/* #[allow(dead_code)] */
const TEXT: &str = "- Bare `#[allow(...)]`";
const RAW: &str = r#"#[allow(dead_code)]"#;
const BYTES: &[u8] = b"#[allow(dead_code)]";
"##;
        assert_eq!(count_bare_allow_attributes(source), 0);
    }

    #[test]
    fn baseline_key_uses_path_family_callee_and_snippet() {
        let f = PanicFinding {
            path: "crates/x/src/lib.rs".into(),
            family: "unwrap".into(),
            line: 42,
            column: 10,
            selector_kind: "method_call".into(),
            selector_callee: "unwrap".into(),
            snippet: ".unwrap()".into(),
        };
        let key = baseline_key(&f);
        assert_eq!(
            key,
            BaselineKey {
                path: "crates/x/src/lib.rs".into(),
                family: "unwrap".into(),
                selector_kind: "method_call".into(),
                selector_callee: "unwrap".into(),
                snippet: ".unwrap()".into(),
            }
        );

        let entry = BaselineEntry::from_key(key.clone(), 1);
        assert_eq!(baseline_entry_key(&entry), key);
    }

    #[test]
    fn blank_string_literals_replaces_string_body_with_spaces() {
        let line = r#"let s = "use .unwrap()"; let other = ".expect()";"#;
        let blanked = blank_string_literals_on_line(line);
        // The string bodies are blanked (replaced with spaces) but the quotes
        // remain so the regex panic-family matchers don't fire.
        assert!(!blanked.contains(".unwrap()"));
        assert!(!blanked.contains(".expect()"));
        assert!(blanked.contains("let s = "));
    }

    #[test]
    fn baseline_entries_count_duplicate_finding_shapes() {
        let config = NoPanicConfig {
            schema_version: None,
            policy: NoPanicPolicy::default(),
            allow: Vec::new(),
        };
        let findings = [
            PanicFinding {
                path: "a.rs".into(),
                family: "unwrap".into(),
                line: 1,
                column: 1,
                selector_kind: "method_call".into(),
                selector_callee: "unwrap".into(),
                snippet: ".unwrap()".into(),
            },
            PanicFinding {
                path: "a.rs".into(),
                family: "unwrap".into(),
                line: 99,
                column: 1,
                selector_kind: "method_call".into(),
                selector_callee: "unwrap".into(),
                snippet: ".unwrap()".into(),
            },
        ];
        let entries = baseline_entries_from_findings(&findings, &config);
        assert_eq!(
            entries.len(),
            1,
            "two findings with same key collapse to one baseline entry",
        );
        assert_eq!(entries[0].count, 2);
    }

    #[test]
    fn new_baseline_debt_detects_count_increase() {
        let key = BaselineKey {
            path: "a.rs".into(),
            family: "unwrap".into(),
            selector_kind: "method_call".into(),
            selector_callee: "unwrap".into(),
            snippet: ".unwrap()".into(),
        };
        let existing = vec![BaselineEntry::from_key(key.clone(), 1)];
        let current = vec![BaselineEntry::from_key(key, 2)];
        let existing_counts = baseline_counts(&existing);
        let new = new_baseline_debt(&current, &existing_counts);
        assert_eq!(new.len(), 1);
        assert_eq!(new[0].1, 1);
    }
}
