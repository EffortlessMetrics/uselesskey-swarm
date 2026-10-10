use std::collections::BTreeSet;

use crate::workspace_graph::WorkspaceGraph;

#[derive(Debug, Clone)]
pub struct Plan {
    pub impacted_crates: BTreeSet<String>,
    pub directly_changed_crates: BTreeSet<String>,
    pub workspace_packages: BTreeSet<String>,
    pub run_fmt: bool,
    pub run_clippy: bool,
    pub run_tests: bool,
    pub run_feature_matrix: bool,
    pub run_dep_guard: bool,
    pub run_bdd: bool,
    /// Build-only PR-lite evidence; ordinary PR proof never executes fuzzing.
    pub run_fuzz_build: bool,
    pub run_no_blob: bool,
    pub run_publish_preflight: bool,
    pub run_root_tests: bool,
    pub run_xtask_tests: bool,
    pub control_paths: BTreeSet<String>,
    pub unclassified_paths: BTreeSet<String>,
    pub docs_only: bool,
}

pub fn build_plan(paths: &[String], workspace: &WorkspaceGraph) -> Plan {
    let mut crate_rust_changed = false;
    let mut xtask_changed = false;
    let mut xtask_cargo_changed = false;
    let mut fuzz_changed = false;
    let mut root_tests_changed = false;
    let mut examples_changed = false;
    let mut cargo_changed = false;
    let mut shared_cargo_changed = false;
    let mut bdd_feature_changed = false;
    let mut no_blob_trigger = false;
    let mut tests_cargo_changed = false;
    let mut changed_crates = BTreeSet::new();
    let mut source_changed_crates = BTreeSet::new();
    let mut control_paths = BTreeSet::new();
    let mut unclassified_paths = BTreeSet::new();
    let mut only_markdown = true;

    for path in paths {
        let path = normalize_path(path);
        let owner = workspace.owner(&path);
        let removed_crate = if owner.is_none() && workspace.path_is_absent(&path) {
            crate_from_path(&path)
        } else {
            None
        };
        let unowned_current_crate = path.starts_with("crates/")
            && owner.is_none()
            && removed_crate.is_none()
            && !path.ends_with(".md");
        let control = is_control_change(&path);
        if control {
            control_paths.insert(path.clone());
        }
        only_markdown &= path.ends_with(".md") && !control;

        if is_crate_rust_change(&path)
            || path.ends_with(".rs")
                && owner
                    .is_some_and(|name| name != "xtask" && name != "uselesskey-integration-tests")
        {
            crate_rust_changed = true;
            if let Some(crate_name) = owner
                .map(ToOwned::to_owned)
                .or_else(|| removed_crate.clone())
            {
                source_changed_crates.insert(crate_name);
            }
        }
        if is_xtask_change(&path) {
            xtask_changed = true;
        }
        if path == "xtask/Cargo.toml" {
            xtask_cargo_changed = true;
        }
        if path.starts_with("fuzz/") && !path.ends_with(".md") {
            fuzz_changed = true;
        }
        if is_root_tests_change(&path) {
            root_tests_changed = true;
        }
        if is_examples_change(&path) {
            examples_changed = true;
        }

        if path.ends_with("Cargo.toml") || path.ends_with("Cargo.lock") {
            cargo_changed = true;
        }
        shared_cargo_changed |= path == "Cargo.toml" || path == "Cargo.lock";
        if path == "tests/Cargo.toml" {
            tests_cargo_changed = true;
        }

        if path.starts_with("crates/uselesskey-bdd/")
            || path.starts_with("features/")
            || path.ends_with(".feature") && path.contains("features/")
        {
            bdd_feature_changed = true;
        }

        if is_no_blob_trigger(&path) {
            no_blob_trigger = true;
        }

        if let Some(crate_name) = owner
            .map(ToOwned::to_owned)
            .or_else(|| removed_crate.clone())
        {
            changed_crates.insert(crate_name);
        }

        if unowned_current_crate
            || !(control
                || owner.is_some()
                || path.ends_with(".md")
                || path.ends_with("Cargo.toml")
                || path.ends_with("Cargo.lock")
                || removed_crate.is_some()
                || path.starts_with("fuzz/")
                || is_root_tests_change(&path)
                || is_examples_change(&path)
                || is_no_blob_trigger(&path)
                || bdd_feature_path(&path))
        {
            unclassified_paths.insert(path);
        }
    }

    let workspace_packages = workspace.packages();
    let impacted_crates = if shared_cargo_changed {
        workspace_packages.clone()
    } else {
        workspace.expand(&changed_crates)
    };
    let directly_changed_crates = source_changed_crates;

    // any_rust_changed is the OR of all five classifiers (used for fmt/clippy)
    let any_rust_changed = crate_rust_changed
        || xtask_changed
        || fuzz_changed
        || root_tests_changed
        || examples_changed;

    let run_fmt = any_rust_changed || cargo_changed;
    let run_clippy = run_fmt;
    let run_tests = crate_rust_changed
        || root_tests_changed
        || impacted_crates.iter().any(|name| {
            !matches!(
                name.as_str(),
                "uselesskey-bdd" | "uselesskey-integration-tests" | "xtask"
            )
        });
    let run_feature_matrix = cargo_changed
        || paths.iter().any(|p| {
            let p = normalize_path(p);
            p.starts_with("crates/uselesskey/")
                || (p.starts_with("crates/uselesskey-bdd/") && p.ends_with(".feature"))
        });
    let run_dep_guard = cargo_changed;
    let run_bdd =
        crate_rust_changed || bdd_feature_changed || impacted_crates.contains("uselesskey-bdd");
    let run_fuzz_build = fuzz_changed;
    let run_no_blob = no_blob_trigger;
    let run_publish_preflight = cargo_changed;
    let run_root_tests = root_tests_changed
        || tests_cargo_changed
        || impacted_crates.contains("uselesskey-integration-tests");
    let run_xtask_tests = xtask_changed || xtask_cargo_changed || impacted_crates.contains("xtask");

    let run_any = run_fmt
        || run_clippy
        || run_tests
        || run_feature_matrix
        || run_dep_guard
        || run_bdd
        || run_fuzz_build
        || run_no_blob
        || run_publish_preflight
        || run_root_tests
        || run_xtask_tests;

    Plan {
        impacted_crates,
        directly_changed_crates,
        workspace_packages,
        run_fmt,
        run_clippy,
        run_tests,
        run_feature_matrix,
        run_dep_guard,
        run_bdd,
        run_fuzz_build,
        run_no_blob,
        run_publish_preflight,
        run_root_tests,
        run_xtask_tests,
        control_paths,
        unclassified_paths,
        docs_only: only_markdown && !run_any,
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn is_crate_rust_change(path: &str) -> bool {
    path.ends_with(".rs")
        && path.starts_with("crates/")
        && (path.contains("/src/") || path.contains("/tests/"))
}

fn is_xtask_change(path: &str) -> bool {
    path.ends_with(".rs") && path.starts_with("xtask/")
}

fn is_control_change(path: &str) -> bool {
    [
        ".github/",
        ".cargo/",
        ".githooks/",
        "policy/",
        ".rails/",
        ".uselesskey/",
    ]
    .iter()
    .any(|prefix| path.starts_with(prefix))
        || matches!(
            path,
            ".gitignore"
                | ".gitattributes"
                | "rust-toolchain.toml"
                | "rustfmt.toml"
                | "clippy.toml"
                | "deny.toml"
                | "lefthook.yml"
                | "codecov.yml"
        )
        || path.starts_with("scripts/")
            && [".py", ".sh", ".ps1", ".toml", ".json"]
                .iter()
                .any(|extension| path.ends_with(extension))
        || is_xtask_change(path)
        || path.starts_with("docs/schemas/") && path.ends_with(".json")
}

fn bdd_feature_path(path: &str) -> bool {
    path.starts_with("features/") || path.ends_with(".feature") && path.contains("features/")
}

fn is_root_tests_change(path: &str) -> bool {
    path.ends_with(".rs") && path.starts_with("tests/")
}

fn is_examples_change(path: &str) -> bool {
    path.ends_with(".rs") && path.starts_with("examples/")
}

fn is_no_blob_trigger(path: &str) -> bool {
    path.starts_with("tests/")
        || path.starts_with("fixtures/")
        || path.starts_with("testdata/")
        || (path.starts_with("crates/") && path.contains("/tests/"))
}

fn crate_from_path(path: &str) -> Option<String> {
    let mut parts = path.split('/');
    if parts.next()? != "crates" {
        return None;
    }
    parts.next().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn build_plan(paths: &[String]) -> Plan {
        static WORKSPACE: OnceLock<WorkspaceGraph> = OnceLock::new();
        let workspace = WORKSPACE.get_or_init(|| {
            WorkspaceGraph::load(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap(),
            )
            .expect("read the actual locked workspace graph")
        });
        super::build_plan(paths, workspace)
    }

    #[test]
    fn docs_only_changes_skip_all() {
        let paths = vec!["README.md".to_string(), "docs/architecture.md".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.docs_only);
        assert!(!plan.run_fmt);
        assert!(!plan.run_clippy);
        assert!(!plan.run_tests);
        assert!(!plan.run_feature_matrix);
        assert!(!plan.run_bdd);
        assert!(!plan.run_fuzz_build);
        assert!(!plan.run_no_blob);
        assert!(!plan.run_publish_preflight);
        assert!(plan.impacted_crates.is_empty());
    }

    #[test]
    fn core_change_expands_dependents() {
        let paths = vec!["crates/uselesskey-core/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = plan.impacted_crates;
        assert!(impacted.contains("uselesskey-core"));
        assert!(impacted.contains("uselesskey-rsa"));
        assert!(impacted.contains("uselesskey-ecdsa"));
        assert!(impacted.contains("uselesskey-ed25519"));
        assert!(impacted.contains("uselesskey-hmac"));
        assert!(impacted.contains("uselesskey-token"));
        assert!(impacted.contains("uselesskey-pgp"));
        assert!(impacted.contains("uselesskey-x509"));
        assert!(impacted.contains("uselesskey"));
        assert!(impacted.contains("uselesskey-bdd"));
        assert!(plan.run_bdd);
        assert!(!plan.run_fuzz_build);
    }

    #[test]
    fn examples_path_counts_as_rust_code_change() {
        let paths = vec!["examples/demo.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_fmt);
        assert!(plan.run_clippy);
        // examples-only changes don't trigger tests (no crate source or root test changes)
        assert!(!plan.run_tests);
    }

    #[test]
    fn xtask_only_change_skips_expensive_steps() {
        let paths = vec!["xtask/src/main.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_fmt);
        assert!(plan.run_clippy);
        assert!(!plan.run_tests);
        assert!(!plan.run_bdd);
        assert!(!plan.run_fuzz_build);
        assert!(!plan.run_root_tests);
        assert!(plan.run_xtask_tests);
        assert!(plan.directly_changed_crates.is_empty());
    }

    #[test]
    fn xtask_rs_change_triggers_xtask_tests() {
        let paths = vec!["xtask/src/main.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_xtask_tests);
    }

    #[test]
    fn xtask_cargo_change_triggers_xtask_tests() {
        let paths = vec!["xtask/Cargo.toml".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_xtask_tests);
    }

    #[test]
    fn fuzz_only_change_selects_build_only_evidence() {
        let paths = vec!["fuzz/fuzz_targets/pem_corrupt.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_fmt);
        assert!(plan.run_clippy);
        assert!(plan.run_fuzz_build);
        assert!(!plan.run_bdd);
    }

    #[test]
    fn root_tests_change_triggers_root_tests() {
        let paths = vec!["tests/governance.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_fmt);
        assert!(plan.run_clippy);
        assert!(plan.run_tests);
        assert!(plan.run_root_tests);
        assert!(!plan.run_bdd);
        assert!(!plan.run_fuzz_build);
    }

    #[test]
    fn tests_cargo_toml_triggers_root_tests() {
        let paths = vec!["tests/Cargo.toml".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_root_tests);
        assert!(!plan.run_bdd);
        assert!(!plan.run_fuzz_build);
    }

    #[test]
    fn workspace_cargo_changes_preserve_consumer_and_package_proof() {
        let paths = vec![
            "Cargo.toml".to_string(),
            "crates/uselesskey-core/Cargo.toml".to_string(),
            "crates/uselesskey-rsa/Cargo.toml".to_string(),
        ];
        let plan = build_plan(&paths);
        assert!(plan.run_fmt);
        assert!(plan.run_clippy);
        assert!(plan.run_tests); // impacted_crates is non-empty
        assert!(plan.run_dep_guard);
        assert!(plan.run_publish_preflight);
        assert!(plan.run_bdd);
        assert!(!plan.run_fuzz_build);
        assert!(plan.run_root_tests);
        assert!(plan.run_xtask_tests);
        assert_eq!(plan.impacted_crates, plan.workspace_packages);
        assert!(plan.directly_changed_crates.is_empty());
    }

    #[test]
    fn directly_changed_crates_tracks_source_changes() {
        let paths = vec![
            "crates/uselesskey-core/src/lib.rs".to_string(),
            "crates/uselesskey-rsa/Cargo.toml".to_string(),
        ];
        let plan = build_plan(&paths);
        // directly_changed_crates only includes crates with actual .rs changes (no expansion)
        assert!(plan.directly_changed_crates.contains("uselesskey-core"));
        assert!(!plan.directly_changed_crates.contains("uselesskey-rsa")); // NOT expanded
        // impacted_crates includes both source and Cargo.toml changes (with expansion)
        assert!(plan.impacted_crates.contains("uselesskey-rsa"));
        assert!(plan.impacted_crates.contains("uselesskey-core"));
    }

    #[test]
    fn no_blob_trigger_sets_flag() {
        let paths = vec!["tests/fixtures/secret.pem".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_no_blob);
    }

    #[test]
    fn bdd_feature_change_runs_bdd() {
        let paths = vec!["crates/uselesskey-bdd/features/rsa.feature".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_bdd);
    }

    #[test]
    fn fuzz_target_change_selects_build_only_evidence() {
        let paths = vec!["fuzz/fuzz_targets/pem_corrupt.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_fuzz_build);
    }

    #[test]
    fn jwk_change_expands_to_key_crates() {
        let paths = vec!["crates/uselesskey-jwk/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-jwk"));
        assert!(impacted.contains("uselesskey-rsa"));
        assert!(impacted.contains("uselesskey-ecdsa"));
        assert!(impacted.contains("uselesskey-ed25519"));
        assert!(impacted.contains("uselesskey-hmac"));
        assert!(impacted.contains("uselesskey"));
    }

    #[test]
    fn hmac_change_expands_to_facade_and_jwt() {
        let paths = vec!["crates/uselesskey-hmac/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-hmac"));
        assert!(impacted.contains("uselesskey"));
        assert!(impacted.contains("uselesskey-jsonwebtoken"));
    }

    #[test]
    fn token_change_expands_to_facade() {
        let paths = vec!["crates/uselesskey-token/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-token"));
        assert!(impacted.contains("uselesskey"));
    }

    #[test]
    fn x509_change_expands_to_tonic_adapter() {
        let paths = vec!["crates/uselesskey-x509/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-x509"));
        assert!(impacted.contains("uselesskey-rustls"));
        assert!(impacted.contains("uselesskey-tonic"));
    }

    #[test]
    fn pgp_change_expands_to_facade() {
        let paths = vec!["crates/uselesskey-pgp/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-pgp"));
        assert!(impacted.contains("uselesskey"));
    }

    #[test]
    fn product_source_does_not_select_fuzz_build() {
        let paths = vec!["crates/uselesskey-core/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_tests);
        assert!(plan.run_bdd);
        assert!(!plan.run_fuzz_build);
    }

    #[test]
    fn cargo_change_enables_publish_preflight() {
        let paths = vec!["Cargo.toml".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_publish_preflight);
    }

    #[test]
    fn cargo_lock_triggers_feature_matrix() {
        let paths = vec!["Cargo.lock".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_feature_matrix);
        assert!(plan.run_fmt);
        assert!(plan.run_clippy);
        assert_eq!(plan.impacted_crates, plan.workspace_packages);
        assert!(plan.run_root_tests && plan.run_xtask_tests && plan.run_bdd);
    }

    #[test]
    fn facade_change_triggers_feature_matrix() {
        let paths = vec!["crates/uselesskey/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_feature_matrix);
    }

    #[test]
    fn bdd_feature_file_triggers_feature_matrix() {
        let paths = vec!["crates/uselesskey-bdd/features/rsa.feature".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_feature_matrix);
    }

    #[test]
    fn windows_paths_normalized_for_feature_matrix() {
        let paths = vec!["crates\\uselesskey\\src\\lib.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(plan.run_feature_matrix);
    }

    #[test]
    fn bdd_steps_change_expands_to_bdd() {
        let paths = vec!["crates/uselesskey-bdd-steps/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-bdd-steps"));
        assert!(impacted.contains("uselesskey-bdd"));
    }

    #[test]
    fn feature_grid_change_expands_to_test_grid_and_bdd() {
        let paths = vec!["crates/uselesskey-feature-grid/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-feature-grid"));
        assert!(impacted.contains("uselesskey-test-grid"));
        assert!(impacted.contains("uselesskey-bdd"));
    }

    #[test]
    fn interop_tests_change_is_self_only() {
        let paths = vec!["crates/uselesskey-interop-tests/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-interop-tests"));
        assert_eq!(impacted.len(), 1);
    }

    #[test]
    fn test_grid_change_expands_to_bdd() {
        let paths = vec!["crates/uselesskey-test-grid/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        let impacted = &plan.impacted_crates;
        assert!(impacted.contains("uselesskey-test-grid"));
        assert!(impacted.contains("uselesskey-bdd"));
    }

    #[test]
    fn test_empty_paths_docs_only() {
        let paths: Vec<String> = vec![];
        let plan = build_plan(&paths);
        assert!(
            plan.docs_only,
            "empty paths should produce a docs_only plan"
        );
        assert!(!plan.run_fmt);
        assert!(!plan.run_clippy);
        assert!(!plan.run_tests);
        assert!(!plan.run_feature_matrix);
        assert!(!plan.run_dep_guard);
        assert!(!plan.run_bdd);
        assert!(!plan.run_fuzz_build);
        assert!(!plan.run_no_blob);
        assert!(!plan.run_publish_preflight);
        assert!(!plan.run_root_tests);
        assert!(!plan.run_xtask_tests);
        assert!(plan.impacted_crates.is_empty());
        assert!(plan.directly_changed_crates.is_empty());
    }

    #[test]
    fn test_crate_rust_change() {
        let paths = vec!["crates/uselesskey-rsa/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(!plan.docs_only);
        assert!(plan.run_tests, "crate .rs change should trigger tests");
        assert!(plan.run_clippy, "crate .rs change should trigger clippy");
        assert!(plan.run_fmt, "crate .rs change should trigger fmt");
        assert!(plan.run_bdd, "crate .rs change should trigger bdd");
        assert!(
            !plan.run_fuzz_build,
            "crate source does not select fuzz work"
        );
        // feature_matrix only triggers for facade crate, .feature files, or Cargo changes
        assert!(
            !plan.run_feature_matrix,
            "non-facade crate .rs change alone should not trigger feature_matrix"
        );
        assert!(
            plan.directly_changed_crates.contains("uselesskey-rsa"),
            "directly_changed_crates should include uselesskey-rsa"
        );
        assert!(
            plan.impacted_crates.contains("uselesskey-rsa"),
            "impacted_crates should include uselesskey-rsa"
        );
        // Transitive dependents of uselesskey-rsa should also be impacted
        assert!(
            plan.impacted_crates.contains("uselesskey-jsonwebtoken"),
            "jsonwebtoken should be impacted via uselesskey-rsa"
        );
        assert!(
            plan.impacted_crates.contains("uselesskey"),
            "facade should be impacted via uselesskey-rsa"
        );
    }

    #[test]
    fn test_xtask_change() {
        let paths = vec!["xtask/src/main.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(!plan.docs_only);
        assert!(
            plan.run_xtask_tests,
            "xtask .rs change should trigger xtask_tests"
        );
        assert!(plan.run_fmt, "xtask .rs change should trigger fmt");
        assert!(plan.run_clippy, "xtask .rs change should trigger clippy");
        assert!(
            !plan.run_tests,
            "xtask-only change should not trigger crate tests"
        );
        assert!(!plan.run_bdd, "xtask-only change should not trigger bdd");
    }

    #[test]
    fn test_bdd_feature_change() {
        let paths = vec!["crates/uselesskey-bdd/features/rsa.feature".to_string()];
        let plan = build_plan(&paths);
        assert!(!plan.docs_only);
        assert!(plan.run_bdd, ".feature file change should trigger bdd");
        assert!(
            plan.run_feature_matrix,
            ".feature file under uselesskey-bdd should trigger feature_matrix"
        );
        assert!(
            !plan.run_fuzz_build,
            "bdd feature-only change should not trigger fuzz"
        );
    }

    #[test]
    fn test_cargo_toml_change() {
        let paths = vec!["Cargo.toml".to_string()];
        let plan = build_plan(&paths);
        assert!(!plan.docs_only);
        assert!(
            plan.run_dep_guard,
            "Cargo.toml change should trigger dep_guard"
        );
        assert!(plan.run_fmt, "Cargo.toml change should trigger fmt");
        assert!(plan.run_clippy, "Cargo.toml change should trigger clippy");
        assert!(
            plan.run_publish_preflight,
            "Cargo.toml change should trigger publish_preflight"
        );
        assert!(
            plan.run_feature_matrix,
            "Cargo.toml change should trigger feature_matrix"
        );
    }

    #[test]
    fn test_fuzz_change() {
        let paths = vec!["fuzz/fuzz_targets/pem_corrupt.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(!plan.docs_only);
        assert!(
            plan.run_fuzz_build,
            "fuzz .rs change selects build-only PR-lite evidence"
        );
        assert!(plan.run_fmt, "fuzz .rs change should trigger fmt");
        assert!(plan.run_clippy, "fuzz .rs change should trigger clippy");
        assert!(!plan.run_bdd, "fuzz-only change should not trigger bdd");

        assert!(
            !plan.run_tests,
            "fuzz-only change should not trigger crate tests"
        );
    }

    #[test]
    fn test_root_tests_change() {
        let paths = vec!["tests/governance.rs".to_string()];
        let plan = build_plan(&paths);
        assert!(!plan.docs_only);
        assert!(
            plan.run_root_tests,
            "tests/ .rs change should trigger root_tests"
        );
        assert!(plan.run_tests, "tests/ .rs change should trigger tests");
        assert!(plan.run_fmt, "tests/ .rs change should trigger fmt");
        assert!(plan.run_clippy, "tests/ .rs change should trigger clippy");
        assert!(plan.run_no_blob, "tests/ path should trigger no_blob");
        assert!(
            !plan.run_bdd,
            "root tests-only change should not trigger bdd"
        );
    }

    #[test]
    fn test_no_blob_trigger() {
        // Various fixture-like paths that should trigger no_blob
        for path in &[
            "tests/fixtures/secret.pem",
            "fixtures/something.json",
            "testdata/key.der",
            "crates/uselesskey-rsa/tests/snapshot.rs",
        ] {
            let paths = vec![path.to_string()];
            let plan = build_plan(&paths);
            assert!(plan.run_no_blob, "path '{}' should trigger no_blob", path);
        }

        // Paths that should NOT trigger no_blob
        for path in &[
            "crates/uselesskey-rsa/src/lib.rs",
            "xtask/src/main.rs",
            "README.md",
        ] {
            let paths = vec![path.to_string()];
            let plan = build_plan(&paths);
            assert!(
                !plan.run_no_blob,
                "path '{}' should NOT trigger no_blob",
                path
            );
        }
    }

    #[test]
    fn test_multiple_changes() {
        let paths = vec![
            "crates/uselesskey-rsa/src/lib.rs".to_string(),
            "xtask/src/plan.rs".to_string(),
            "fuzz/fuzz_targets/pem_corrupt.rs".to_string(),
            "Cargo.toml".to_string(),
            "tests/governance.rs".to_string(),
            "crates/uselesskey-bdd/features/rsa.feature".to_string(),
            "fixtures/something.pem".to_string(),
        ];
        let plan = build_plan(&paths);
        assert!(
            !plan.docs_only,
            "multiple real changes should not be docs_only"
        );
        // Crate .rs change flags
        assert!(plan.run_tests);
        assert!(plan.run_bdd);
        // Fuzz change
        assert!(plan.run_fuzz_build);
        // Xtask change
        assert!(plan.run_xtask_tests);
        // Cargo.toml change
        assert!(plan.run_dep_guard);
        assert!(plan.run_publish_preflight);
        assert!(plan.run_feature_matrix);
        // Root tests change
        assert!(plan.run_root_tests);
        // Fixture path
        assert!(plan.run_no_blob);
        // Common flags
        assert!(plan.run_fmt);
        assert!(plan.run_clippy);
        // Impacted crates should include uselesskey-rsa and its dependents
        assert!(plan.impacted_crates.contains("uselesskey-rsa"));
        assert!(
            plan.impacted_crates.contains("uselesskey"),
            "facade should be impacted via uselesskey-rsa"
        );
        // directly_changed_crates should only include crates with .rs source changes
        assert!(plan.directly_changed_crates.contains("uselesskey-rsa"));
        assert!(
            !plan.directly_changed_crates.contains("uselesskey"),
            "facade should NOT be in directly_changed (no direct .rs change)"
        );
    }

    #[test]
    fn test_impacted_crates_includes_dependents() {
        // Changing uselesskey-core should transitively impact many downstream crates
        let paths = vec!["crates/uselesskey-core/src/lib.rs".to_string()];
        let plan = build_plan(&paths);
        // Direct dependents of uselesskey-core
        assert!(plan.impacted_crates.contains("uselesskey-core"));
        assert!(plan.impacted_crates.contains("uselesskey-rsa"));
        assert!(plan.impacted_crates.contains("uselesskey-ecdsa"));
        assert!(plan.impacted_crates.contains("uselesskey-ed25519"));
        assert!(plan.impacted_crates.contains("uselesskey-hmac"));
        assert!(plan.impacted_crates.contains("uselesskey-token"));
        assert!(plan.impacted_crates.contains("uselesskey-pgp"));
        assert!(plan.impacted_crates.contains("uselesskey-x509"));
        assert!(plan.impacted_crates.contains("uselesskey"));
        assert!(plan.impacted_crates.contains("uselesskey-bdd"));
        // Transitive dependents (via uselesskey-rsa -> uselesskey-jsonwebtoken, etc.)
        assert!(
            plan.impacted_crates.contains("uselesskey-jsonwebtoken"),
            "jsonwebtoken should be transitively impacted via uselesskey-rsa"
        );
        assert!(
            plan.impacted_crates.contains("uselesskey-rustls"),
            "rustls should be transitively impacted via uselesskey-rsa or uselesskey-x509"
        );
        assert!(
            plan.impacted_crates.contains("uselesskey-ring"),
            "ring should be transitively impacted via uselesskey-rsa"
        );
        assert!(
            plan.impacted_crates.contains("uselesskey-rustcrypto"),
            "rustcrypto should be transitively impacted via uselesskey-rsa"
        );
        assert!(
            plan.impacted_crates.contains("uselesskey-aws-lc-rs"),
            "aws-lc-rs should be transitively impacted via uselesskey-rsa"
        );
        assert!(
            plan.impacted_crates.contains("uselesskey-tonic"),
            "tonic should be transitively impacted via uselesskey-x509"
        );
        // directly_changed_crates should only have the core crate
        assert_eq!(plan.directly_changed_crates.len(), 1);
        assert!(plan.directly_changed_crates.contains("uselesskey-core"));
    }

    #[test]
    fn control_paths_are_not_documentation_only() {
        for path in [
            ".github/workflows/required.yml",
            "scripts/seal-development-proof.py",
            "policy/non-rust-allowlist.toml",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            ".githooks/pre-commit",
            ".rails/index.toml",
            "docs/schemas/receipt.json",
        ] {
            let plan = build_plan(&[path.to_string()]);
            assert!(!plan.docs_only, "control path: {path}");
            assert!(plan.control_paths.contains(path), "control path: {path}");
            assert!(
                plan.unclassified_paths.is_empty(),
                "known control path: {path}"
            );
            assert!(!plan.run_fuzz_build);
        }
    }

    #[test]
    fn unknown_paths_do_not_acquire_documentation_proof() {
        for path in [
            "unknown.asset",
            " scripts/test_fake.py",
            "scripts/control.rb",
            "src/new.rs",
        ] {
            let plan = build_plan(&[path.to_string()]);
            assert!(!plan.docs_only, "unknown path: {path}");
            assert!(
                plan.unclassified_paths.contains(path),
                "unknown path: {path}"
            );
        }
    }

    #[test]
    fn current_unowned_crates_refuse_but_removed_packages_keep_conservative_routing()
    -> anyhow::Result<()> {
        use std::fs;

        let directory = tempfile::tempdir()?;
        let root = directory.path();
        let manifest = root.join("Cargo.toml");
        fs::write(
            &manifest,
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/original-dir\", \"crates/consumer\", \"crates/removed\"]\nexclude = [\"crates/excluded\"]\n",
        )?;
        for (directory, name, dependencies) in [
            ("original-dir", "new-owner-name", ""),
            (
                "consumer",
                "consumer",
                "[dependencies]\ninput = { package = \"new-owner-name\", path = \"../original-dir\" }\n",
            ),
            ("removed", "removed", ""),
            ("excluded", "excluded", ""),
        ] {
            let package = root.join("crates").join(directory);
            fs::create_dir_all(package.join("src"))?;
            fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n")?;
            fs::write(
                package.join("Cargo.toml"),
                format!(
                    "[package]\nname = {name:?}\nversion = \"0.1.0\"\nedition = \"2024\"\n{dependencies}"
                ),
            )?;
        }

        let workspace = WorkspaceGraph::load(root)?;
        for path in ["crates/excluded/src/lib.rs", "crates/excluded/Cargo.toml"] {
            let plan = super::build_plan(&[path.to_string()], &workspace);
            assert_eq!(plan.unclassified_paths, BTreeSet::from([path.to_string()]));
            assert!(!plan.impacted_crates.contains("excluded"));
            assert!(!plan.docs_only);
        }
        #[cfg(unix)]
        {
            let dangling = "crates/excluded/src/dangling.rs";
            std::os::unix::fs::symlink("absent-target", root.join(dangling))?;
            let plan = super::build_plan(&[dangling.to_string()], &workspace);
            assert!(plan.unclassified_paths.contains(dangling));
        }

        let renamed =
            super::build_plan(&["crates/original-dir/src/lib.rs".to_string()], &workspace);
        assert!(renamed.unclassified_paths.is_empty());
        assert_eq!(
            renamed.directly_changed_crates,
            BTreeSet::from(["new-owner-name".to_string()])
        );
        assert_eq!(
            renamed.impacted_crates,
            BTreeSet::from(["new-owner-name".to_string(), "consumer".to_string()])
        );

        fs::remove_dir_all(root.join("crates/removed"))?;
        fs::write(
            &manifest,
            "[workspace]\nresolver = \"2\"\nmembers = [\"crates/original-dir\", \"crates/consumer\"]\nexclude = [\"crates/excluded\"]\n",
        )?;
        let workspace = WorkspaceGraph::load(root)?;
        let removed = super::build_plan(
            &[
                "Cargo.toml".to_string(),
                "crates/removed/Cargo.toml".to_string(),
                "crates/removed/src/lib.rs".to_string(),
            ],
            &workspace,
        );
        assert!(removed.unclassified_paths.is_empty());
        assert!(removed.directly_changed_crates.contains("removed"));
        assert_eq!(removed.impacted_crates, workspace.packages());
        assert!(removed.run_tests && removed.run_publish_preflight && removed.run_dep_guard);
        assert!(removed.run_feature_matrix);
        Ok(())
    }

    #[test]
    fn docs_do_not_mask_an_unknown_or_control_path() {
        let plan = build_plan(&["README.md".to_string(), "unknown.asset".to_string()]);
        assert!(!plan.docs_only);
        assert_eq!(
            plan.unclassified_paths,
            BTreeSet::from(["unknown.asset".to_string()])
        );
        let plan = build_plan(&["README.md".to_string(), "scripts/tool.py".to_string()]);
        assert!(!plan.docs_only);
        assert_eq!(
            plan.control_paths,
            BTreeSet::from(["scripts/tool.py".to_string()])
        );
    }

    #[test]
    fn fuzz_owned_manifests_and_corpus_select_only_the_build_obligation() {
        for path in [
            "fuzz/Cargo.toml",
            "fuzz/Cargo.lock",
            "fuzz/corpus/pem_corrupt/sample",
        ] {
            let plan = build_plan(&[path.to_string()]);
            assert!(plan.run_fuzz_build, "fuzz path: {path}");
            assert!(!plan.docs_only);
            assert!(!plan.run_tests);
            assert!(!plan.run_bdd);
        }
        let plan = build_plan(&["fuzz/README.md".to_string()]);
        assert!(plan.docs_only);
        assert!(!plan.run_fuzz_build);
    }

    #[test]
    fn reduced_plan_preserves_package_and_consumer_obligations() {
        let paths = [
            "crates/uselesskey-rsa/Cargo.toml".to_string(),
            "crates/uselesskey-rsa/src/lib.rs".to_string(),
            "crates/uselesskey-rsa/tests/regression.rs".to_string(),
            "xtask/src/plan.rs".to_string(),
            "tests/governance.rs".to_string(),
        ];
        let plan = build_plan(&paths);
        assert!(plan.run_fmt && plan.run_clippy && plan.run_tests);
        assert!(plan.run_feature_matrix && plan.run_dep_guard && plan.run_publish_preflight);
        assert!(plan.run_bdd && plan.run_root_tests && plan.run_xtask_tests && plan.run_no_blob);
        assert!(plan.impacted_crates.contains("uselesskey-jsonwebtoken"));
        assert!(plan.impacted_crates.contains("uselesskey-tonic"));
        assert!(!plan.run_fuzz_build);
        let reversed = build_plan(&paths.into_iter().rev().collect::<Vec<_>>());
        assert_eq!(plan.impacted_crates, reversed.impacted_crates);
        assert_eq!(plan.control_paths, reversed.control_paths);
    }

    #[test]
    fn core_change_reaches_consumers_missing_from_the_old_table() {
        let plan = build_plan(&["crates/uselesskey-core/src/lib.rs".to_string()]);
        for name in ["uselesskey-axum", "uselesskey-ssh", "uselesskey-webhook"] {
            assert!(
                plan.impacted_crates.contains(name),
                "declared consumer: {name}"
            );
        }
    }
}
