use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

const OUTPUT_LOCK_TIMEOUT: Duration = Duration::from_mins(2);
const OUTPUT_LOCK_RETRY: Duration = Duration::from_millis(250);

pub(crate) struct TargetOutputLock {
    path: PathBuf,
}

impl Drop for TargetOutputLock {
    fn drop(&mut self) {
        match fs::remove_dir(&self.path) {
            Ok(()) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }
}

pub(crate) fn acquire_lock(
    root: &Path,
    lock_dir: impl AsRef<Path>,
    command_name: &str,
) -> Result<TargetOutputLock> {
    let lock_dir = lock_dir.as_ref();
    if lock_dir.is_absolute()
        || lock_dir
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        bail!(
            "{command_name} output lock path must be a relative target/ child: {}",
            lock_dir.display()
        );
    }

    let lock_dir = root.join(lock_dir);
    ensure_target_child(root, &lock_dir, command_name)?;
    let target_dir = root.join("target");
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("failed to create {}", target_dir.display()))?;

    let start = Instant::now();
    loop {
        match fs::create_dir(&lock_dir) {
            Ok(()) => return Ok(TargetOutputLock { path: lock_dir }),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                if start.elapsed() >= OUTPUT_LOCK_TIMEOUT {
                    bail!(
                        "timed out waiting for {command_name} output lock at {}; remove this directory only if no {command_name} run is active",
                        lock_dir.display()
                    );
                }
                thread::sleep(OUTPUT_LOCK_RETRY);
            }
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to create {}", lock_dir.display()));
            }
        }
    }
}

fn ensure_target_child(root: &Path, path: &Path, command_name: &str) -> Result<()> {
    let absolute = absolute_path(root, path);
    let target_root = absolute_path(root, &root.join("target"));
    if !absolute.starts_with(&target_root) {
        bail!(
            "{command_name} refuses to write outside target/: {}",
            path.display()
        );
    }
    Ok(())
}

fn absolute_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_output_lock_rejects_non_target_lock_dir() -> Result<()> {
        let root = std::env::temp_dir().join("uselesskey-target-output-test");
        let err = match acquire_lock(&root, "../outside-target-output.lock", "test-command") {
            Ok(_) => bail!("non-target output lock was accepted"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("relative target/ child"));
        Ok(())
    }

    #[test]
    fn target_output_lock_rejects_target_parent_traversal() -> Result<()> {
        let root = std::env::temp_dir().join("uselesskey-target-output-test");
        let err = match acquire_lock(
            &root,
            "target/../outside-target-output.lock",
            "test-command",
        ) {
            Ok(_) => bail!("traversing output lock was accepted"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("relative target/ child"));
        Ok(())
    }

    #[test]
    fn target_output_lock_serializes_access() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let first_lock = acquire_lock(dir.path(), "target/test-command.lock", "test-command")?;
        let (tx, rx) = std::sync::mpsc::channel();
        let root = dir.path().to_path_buf();
        let waiter = std::thread::spawn(move || -> Result<()> {
            let _second_lock = acquire_lock(&root, "target/test-command.lock", "test-command")?;
            tx.send(())
                .expect("lock waiter can report successful acquisition");
            Ok(())
        });

        match rx.recv_timeout(Duration::from_millis(300)) {
            Ok(()) => bail!("second target output lock acquired too early"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                bail!("lock waiter disconnected before acquiring the output lock")
            }
        }

        drop(first_lock);
        rx.recv_timeout(Duration::from_secs(5))
            .context("second target output lock did not acquire after release")?;
        waiter
            .join()
            .expect("lock waiter thread panicked while acquiring output lock")?;
        Ok(())
    }
}
