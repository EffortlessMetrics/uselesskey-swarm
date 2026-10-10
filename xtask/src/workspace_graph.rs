//! Workspace owners and consumers from Cargo's declared package graph.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct WorkspaceGraph {
    root: PathBuf,
    owners: BTreeMap<PathBuf, String>,
    consumers: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Deserialize)]
struct Metadata {
    workspace_root: PathBuf,
    workspace_members: Vec<String>,
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    name: String,
    path: Option<PathBuf>,
}

impl WorkspaceGraph {
    pub fn load(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .context("resolve workspace graph root")?;
        let output = Command::new("cargo")
            .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
            .current_dir(&root)
            .output()
            .context("run locked Cargo metadata for PR selection")?;
        if !output.status.success() {
            bail!(
                "locked Cargo metadata failed for PR selection: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        let metadata: Metadata = serde_json::from_slice(&output.stdout)
            .context("parse Cargo metadata for PR selection")?;
        if metadata.workspace_root != root {
            bail!("Cargo metadata names a different workspace root");
        }
        Self::from_metadata(metadata)
    }

    fn from_metadata(metadata: Metadata) -> Result<Self> {
        let members = metadata
            .workspace_members
            .into_iter()
            .collect::<BTreeSet<_>>();
        let packages = metadata
            .packages
            .into_iter()
            .filter(|package| members.contains(&package.id))
            .collect::<Vec<_>>();
        let present = packages
            .iter()
            .map(|package| package.id.clone())
            .collect::<BTreeSet<_>>();
        if present != members || packages.len() != members.len() || members.is_empty() {
            bail!("Cargo metadata does not enumerate every workspace member exactly once");
        }
        let mut owners = BTreeMap::new();
        let mut absolute_owners = BTreeMap::new();
        let mut consumers = BTreeMap::new();
        for package in &packages {
            let directory = package
                .manifest_path
                .parent()
                .context("package manifest has no parent")?;
            let relative = directory
                .strip_prefix(&metadata.workspace_root)
                .context("workspace package lies outside the repository")?;
            if owners
                .insert(relative.to_path_buf(), package.name.clone())
                .is_some()
                || consumers
                    .insert(package.name.clone(), BTreeSet::new())
                    .is_some()
            {
                bail!("Cargo metadata repeats a package name or directory");
            }
            absolute_owners.insert(directory.to_path_buf(), package.name.clone());
        }
        for package in &packages {
            // Cargo metadata uses the package name even when the dependency is
            // renamed. Retain optional, target-specific, dev and build edges;
            // an inactive default feature is not grounds to drop a consumer.
            for dependency in &package.dependencies {
                if dependency.path.is_none() && consumers.contains_key(&dependency.name) {
                    bail!(
                        "declaration-only Cargo metadata cannot establish whether dependency '{}' of '{}' resolves to its workspace namesake; use an explicit workspace path or qualify the resolved graph before PR selection",
                        dependency.name,
                        package.name
                    );
                }
                let Some(name) = dependency
                    .path
                    .as_ref()
                    .and_then(|path| absolute_owners.get(path))
                else {
                    continue;
                };
                if name != &dependency.name {
                    bail!("workspace dependency name disagrees with its manifest path");
                }
                consumers
                    .get_mut(name)
                    .context("missing workspace dependency owner")?
                    .insert(package.name.clone());
            }
        }
        Ok(Self {
            root: metadata.workspace_root,
            owners,
            consumers,
        })
    }

    pub fn owner(&self, path: &str) -> Option<&str> {
        let path = Path::new(path);
        self.owners
            .iter()
            .filter(|(directory, _)| path.starts_with(directory))
            .max_by_key(|(directory, _)| directory.components().count())
            .map(|(_, name)| name.as_str())
    }

    pub fn packages(&self) -> BTreeSet<String> {
        self.consumers.keys().cloned().collect()
    }

    pub fn path_is_absent(&self, path: &str) -> bool {
        // Dangling symlinks are still candidate entries. Permission and other
        // inspection errors must not turn a current path into a deleted one.
        std::fs::symlink_metadata(self.root.join(path))
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
    }

    pub fn expand(&self, directly_changed: &BTreeSet<String>) -> BTreeSet<String> {
        let mut selected = BTreeSet::new();
        let mut queue = directly_changed.iter().cloned().collect::<Vec<_>>();
        while let Some(name) = queue.pop() {
            if selected.insert(name.clone())
                && let Some(consumers) = self.consumers.get(&name)
            {
                queue.extend(consumers.iter().cloned());
            }
        }
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_package(root: &Path, directory: &str, name: &str, dependencies: &str) -> Result<()> {
        let directory = root.join(directory);
        fs::create_dir_all(directory.join("src"))?;
        fs::write(directory.join("src/lib.rs"), "pub fn fixture() {}\n")?;
        fs::write(
            directory.join("Cargo.toml"),
            format!(
                "[package]\nname = {name:?}\nversion = \"0.1.0\"\nedition = \"2024\"\n{dependencies}"
            ),
        )?;
        Ok(())
    }

    #[test]
    fn new_renamed_optional_dev_build_and_target_consumers_are_selected() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        fs::write(
            root.join("Cargo.toml"),
            r#"[workspace]
resolver = "2"
members = ["producer", "new-place/consumer", "build-consumer", "test-consumer", "target-consumer"]
"#,
        )?;
        write_package(root, "producer", "producer", "")?;
        write_package(
            root,
            "new-place/consumer",
            "renamed-consumer",
            r#"[dependencies]
renamed_input = { package = "producer", path = "../../producer", optional = true }
"#,
        )?;
        write_package(
            root,
            "build-consumer",
            "build-consumer",
            r#"[build-dependencies]
renamed-consumer = { path = "../new-place/consumer" }
"#,
        )?;
        write_package(
            root,
            "test-consumer",
            "test-consumer",
            r#"[dev-dependencies]
producer = { path = "../producer" }
"#,
        )?;
        write_package(
            root,
            "target-consumer",
            "target-consumer",
            r#"[target.'cfg(windows)'.dependencies]
producer = { path = "../producer" }
"#,
        )?;
        let graph = WorkspaceGraph::load(root)?;
        assert_eq!(
            graph.owner("new-place/consumer/src/lib.rs"),
            Some("renamed-consumer")
        );
        assert_eq!(graph.owner("new-place/consumer-extra/src/lib.rs"), None);
        assert_eq!(
            graph.expand(&BTreeSet::from(["producer".to_string()])),
            graph.packages()
        );
        assert!(
            !root.join("Cargo.lock").exists(),
            "no-deps selection must not generate a lockfile"
        );
        Ok(())
    }

    #[test]
    fn cycles_terminate_and_unrelated_registry_dependencies_are_ignored() -> Result<()> {
        let metadata: Metadata = serde_json::from_value(serde_json::json!({
            "workspace_root": "/workspace", "workspace_members": ["a-id", "b-id", "other-id"],
            "packages": [
                {"id":"a-id", "name":"a", "manifest_path":"/workspace/a/Cargo.toml",
                 "dependencies":[{"name":"b", "path":"/workspace/b"}]},
                {"id":"b-id", "name":"b", "manifest_path":"/workspace/b/Cargo.toml",
                 "dependencies":[{"name":"a", "path":"/workspace/a"}]},
                {"id":"other-id", "name":"other", "manifest_path":"/workspace/other/Cargo.toml",
                 "dependencies":[{"name":"registry-input", "path":null}]}
            ]
        }))?;
        let graph = WorkspaceGraph::from_metadata(metadata)?;
        assert_eq!(
            graph.expand(&BTreeSet::from(["a".to_string()])),
            BTreeSet::from(["a".to_string(), "b".to_string()])
        );
        Ok(())
    }

    #[test]
    fn ambiguous_registry_workspace_names_refuse_selection() -> Result<()> {
        // A registry or git declaration can be patched to a workspace member.
        // --no-deps omits the resolved graph; a missing path cannot rule out an edge.
        let metadata: Metadata = serde_json::from_value(serde_json::json!({
            "workspace_root": "/workspace", "workspace_members": ["a-id", "consumer-id"],
            "packages": [
                {"id":"a-id", "name":"a", "manifest_path":"/workspace/a/Cargo.toml",
                 "dependencies":[]},
                {"id":"consumer-id", "name":"consumer", "manifest_path":"/workspace/consumer/Cargo.toml",
                 "dependencies":[{"name":"a", "path":null}]}
            ]
        }))?;
        let error = WorkspaceGraph::from_metadata(metadata)
            .expect_err("an unresolved workspace namesake must not silently lose its consumer");
        assert!(error.to_string().contains("dependency 'a' of 'consumer'"));
        assert!(error.to_string().contains("qualify the resolved graph"));
        Ok(())
    }

    #[test]
    fn missing_workspace_member_fails_instead_of_narrowing_the_graph() -> Result<()> {
        let metadata: Metadata = serde_json::from_value(serde_json::json!({
            "workspace_root":"/workspace", "workspace_members":["missing-id"], "packages":[]
        }))?;
        assert!(WorkspaceGraph::from_metadata(metadata).is_err());
        Ok(())
    }

    #[test]
    fn invalid_workspace_refuses_selection() -> Result<()> {
        let directory = tempfile::tempdir()?;
        fs::write(
            directory.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"missing\"]\n",
        )?;
        let error = WorkspaceGraph::load(directory.path())
            .expect_err("missing workspace member must refuse");
        assert!(error.to_string().contains("locked Cargo metadata failed"));
        Ok(())
    }
}
