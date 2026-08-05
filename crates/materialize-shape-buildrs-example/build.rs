use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=uselesskey-fixtures.toml");

    let manifest_dir = required_env_path("CARGO_MANIFEST_DIR")?;
    let manifest_path = manifest_dir.join("uselesskey-fixtures.toml");
    let out_dir = required_env_path("OUT_DIR")?;
    let module_path = out_dir.join("fixtures.rs");

    let manifest = uselesskey_cli::load_materialize_manifest(&manifest_path).map_err(|error| {
        io::Error::other(format!(
            "failed to load materialize manifest {}: {error}",
            manifest_path.display()
        ))
    })?;
    uselesskey_cli::materialize_manifest_to_dir(&manifest, &out_dir, false)
        .map_err(|error| io::Error::other(format!("failed to materialize fixtures: {error}")))?;
    uselesskey_cli::emit_include_bytes_module(&manifest, &out_dir, &module_path).map_err(
        |error| {
            io::Error::other(format!(
                "failed to emit include-bytes module {}: {error}",
                module_path.display()
            ))
        },
    )?;

    Ok(())
}

fn required_env_path(name: &str) -> Result<PathBuf, io::Error> {
    env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        io::Error::other(format!(
            "required build-script environment variable {name} is missing"
        ))
    })
}
