use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::target_output;

const DEFAULT_OUT: &str = "target/source-of-truth/merge-queue-check.json";
const LOCK_DIR: &str = "target/check-merge-queue.lock";

#[derive(Debug, Clone)]
pub(crate) struct MergeQueueOptions {
    pub(crate) repo: Option<String>,
    pub(crate) main_branch: String,
    pub(crate) runs_json: Option<PathBuf>,
    pub(crate) out: PathBuf,
    pub(crate) strict: bool,
    pub(crate) urgent_ci_repair: bool,
}

impl Default for MergeQueueOptions {
    fn default() -> Self {
        Self {
            repo: None,
            main_branch: "main".to_string(),
            runs_json: None,
            out: PathBuf::from(DEFAULT_OUT),
            strict: false,
            urgent_ci_repair: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct WorkflowRun {
    #[serde(rename = "databaseId")]
    pub(crate) database_id: u64,
    #[serde(rename = "workflowName")]
    pub(crate) workflow_name: String,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) conclusion: String,
    #[serde(default)]
    pub(crate) event: String,
    #[serde(rename = "headSha", default)]
    pub(crate) head_sha: String,
    #[serde(default)]
    pub(crate) url: String,
    #[serde(rename = "createdAt", default)]
    pub(crate) created_at: String,
    #[serde(rename = "updatedAt", default)]
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MergeQueueDecision {
    Pass,
    Hold,
    Investigate,
    Unknown,
}

impl MergeQueueDecision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Hold => "hold",
            Self::Investigate => "investigate",
            Self::Unknown => "unknown",
        }
    }

    fn is_strict_failure(self) -> bool {
        !matches!(self, Self::Pass)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MergeQueueReceipt {
    schema_version: &'static str,
    generated_by: &'static str,
    checked_at: String,
    repository: Option<String>,
    main_branch: String,
    urgent_ci_repair: bool,
    decision: MergeQueueDecision,
    reason: String,
    required_action: String,
    latest_main_rust_run: Option<WorkflowRun>,
    merge_blockers: Vec<String>,
    boundaries: Vec<&'static str>,
}

pub(crate) fn run(root: &Path, options: MergeQueueOptions) -> Result<()> {
    ensure_target_path(&options.out)?;
    let _lock = target_output::acquire_lock(root, LOCK_DIR, "check-merge-queue")?;

    let runs = match &options.runs_json {
        Some(path) => read_runs_json(path)?,
        None => fetch_runs(&options)?,
    };
    let receipt = evaluate_runs(&runs, &options);

    let out_path = root.join(&options.out);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&out_path, serde_json::to_string_pretty(&receipt)? + "\n")
        .with_context(|| format!("failed to write {}", out_path.display()))?;

    println!(
        "merge-queue: {} - {}",
        receipt.decision.as_str(),
        receipt.reason
    );
    println!("merge-queue: wrote {}", options.out.display());

    if options.strict && receipt.decision.is_strict_failure() {
        bail!(
            "merge-queue strict check failed: {} ({})",
            receipt.decision.as_str(),
            receipt.required_action
        );
    }

    Ok(())
}

fn read_runs_json(path: &Path) -> Result<Vec<WorkflowRun>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    parse_runs_json(&text).with_context(|| format!("failed to parse {}", path.display()))
}

fn fetch_runs(options: &MergeQueueOptions) -> Result<Vec<WorkflowRun>> {
    let mut cmd = Command::new("gh");
    cmd.args([
        "run",
        "list",
        "--branch",
        &options.main_branch,
        "--limit",
        "10",
        "--json",
        "databaseId,workflowName,status,conclusion,createdAt,updatedAt,headSha,url,event",
    ]);
    if let Some(repo) = &options.repo {
        cmd.args(["--repo", repo]);
    }

    let output = cmd
        .output()
        .context("failed to invoke `gh run list` for merge queue check")?;
    if !output.status.success() {
        bail!(
            "`gh run list` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8(output.stdout).context("gh output was not UTF-8")?;
    parse_runs_json(&stdout)
}

fn parse_runs_json(text: &str) -> Result<Vec<WorkflowRun>> {
    serde_json::from_str(text).context("run list JSON must be an array of workflow runs")
}

fn evaluate_runs(runs: &[WorkflowRun], options: &MergeQueueOptions) -> MergeQueueReceipt {
    let latest = runs
        .iter()
        .find(|run| run.workflow_name == "EM CI Routed Rust" && run.event == "push")
        .cloned();

    let (decision, reason, required_action, merge_blockers) = match latest.as_ref() {
        None => (
            MergeQueueDecision::Unknown,
            "no main EM CI Routed Rust push run was found".to_string(),
            "inspect GitHub Actions before merging another main-changing PR".to_string(),
            vec!["missing main routed Rust proof".to_string()],
        ),
        Some(run) if is_live(&run.status) && options.urgent_ci_repair => (
            MergeQueueDecision::Pass,
            format!(
                "newest main routed Rust run is {}; urgent CI repair override is set",
                run.status
            ),
            "merge only the narrow CI repair and verify the replacement main run".to_string(),
            Vec::new(),
        ),
        Some(run) if is_live(&run.status) => (
            MergeQueueDecision::Hold,
            format!("newest main routed Rust run is {}", run.status),
            "wait for Uselesskey Main Full Gate and Uselesskey Rust Small Result to finish"
                .to_string(),
            vec!["live main full-gate proof is unresolved".to_string()],
        ),
        Some(run) if is_success(run) => (
            MergeQueueDecision::Pass,
            "newest main routed Rust proof is green".to_string(),
            "safe to consider the next merge candidate, subject to PR checks and scope".to_string(),
            Vec::new(),
        ),
        Some(run) if run.status == "completed" => (
            MergeQueueDecision::Investigate,
            format!(
                "newest main routed Rust run completed with conclusion `{}`",
                display_conclusion(&run.conclusion)
            ),
            "triage or repair main proof before merging unrelated work".to_string(),
            vec!["latest main routed Rust proof is not green".to_string()],
        ),
        Some(run) => (
            MergeQueueDecision::Unknown,
            format!(
                "newest main routed Rust run has unrecognized status `{}`",
                run.status
            ),
            "inspect GitHub Actions before merging another main-changing PR".to_string(),
            vec!["main routed Rust proof status is unknown".to_string()],
        ),
    };

    MergeQueueReceipt {
        schema_version: "1.0",
        generated_by: "cargo xtask check-merge-queue",
        checked_at: Utc::now().to_rfc3339(),
        repository: options.repo.clone(),
        main_branch: options.main_branch.clone(),
        urgent_ci_repair: options.urgent_ci_repair,
        decision,
        reason,
        required_action,
        latest_main_rust_run: latest,
        merge_blockers,
        boundaries: vec![
            "This is advisory queue evidence unless a caller runs it with --strict.",
            "It does not replace Uselesskey Rust Small Result or Source of Truth Advisory.",
            "It does not move release, publish, signing, tag, GitHub release, crates.io, or source-sync authority.",
        ],
    }
}

fn is_live(status: &str) -> bool {
    matches!(
        status,
        "queued" | "in_progress" | "requested" | "waiting" | "pending"
    )
}

fn is_success(run: &WorkflowRun) -> bool {
    run.status == "completed" && run.conclusion == "success"
}

fn display_conclusion(conclusion: &str) -> &str {
    if conclusion.is_empty() {
        "<empty>"
    } else {
        conclusion
    }
}

fn ensure_target_path(path: &Path) -> Result<()> {
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        || !path.starts_with("target")
    {
        bail!(
            "check-merge-queue output path must be a relative target/ child: {}",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(status: &str, conclusion: &str) -> WorkflowRun {
        WorkflowRun {
            database_id: 42,
            workflow_name: "EM CI Routed Rust".to_string(),
            status: status.to_string(),
            conclusion: conclusion.to_string(),
            event: "push".to_string(),
            head_sha: "abc123".to_string(),
            url: "https://example.invalid/run".to_string(),
            created_at: "2026-06-02T12:00:00Z".to_string(),
            updated_at: "2026-06-02T12:01:00Z".to_string(),
        }
    }

    fn options() -> MergeQueueOptions {
        MergeQueueOptions {
            repo: Some("EffortlessMetrics/uselesskey-swarm".to_string()),
            ..MergeQueueOptions::default()
        }
    }

    #[test]
    fn holds_when_latest_main_proof_is_live() {
        let receipt = evaluate_runs(&[run("in_progress", "")], &options());

        assert_eq!(receipt.decision, MergeQueueDecision::Hold);
        assert_eq!(
            receipt.merge_blockers,
            vec!["live main full-gate proof is unresolved"]
        );
        assert!(
            receipt
                .required_action
                .contains("wait for Uselesskey Main Full Gate")
        );
    }

    #[test]
    fn urgent_ci_repair_override_allows_live_main_proof() {
        let mut opts = options();
        opts.urgent_ci_repair = true;
        let receipt = evaluate_runs(&[run("in_progress", "")], &opts);

        assert_eq!(receipt.decision, MergeQueueDecision::Pass);
        assert!(receipt.merge_blockers.is_empty());
        assert!(receipt.reason.contains("urgent CI repair override"));
    }

    #[test]
    fn passes_when_latest_main_proof_is_green() {
        let receipt = evaluate_runs(&[run("completed", "success")], &options());

        assert_eq!(receipt.decision, MergeQueueDecision::Pass);
        assert!(receipt.merge_blockers.is_empty());
    }

    #[test]
    fn investigates_when_latest_main_proof_failed() {
        let receipt = evaluate_runs(&[run("completed", "failure")], &options());

        assert_eq!(receipt.decision, MergeQueueDecision::Investigate);
        assert_eq!(
            receipt.merge_blockers,
            vec!["latest main routed Rust proof is not green"]
        );
    }

    #[test]
    fn ignores_pull_request_runs_when_finding_main_push_proof() {
        let mut pr_run = run("in_progress", "");
        pr_run.event = "pull_request".to_string();
        let receipt = evaluate_runs(&[pr_run, run("completed", "success")], &options());

        assert_eq!(receipt.decision, MergeQueueDecision::Pass);
        assert_eq!(receipt.latest_main_rust_run.unwrap().event, "push");
    }

    #[test]
    fn rejects_non_target_output_paths() {
        let err = ensure_target_path(Path::new("docs/merge-queue.json")).unwrap_err();
        assert!(err.to_string().contains("relative target/ child"));
    }
}
