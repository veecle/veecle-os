//! Build script used to detect all workspaces within the repository and makes them available for the test generation.
//!
//! `nextest` executes the test binary multiple times.
//! To avoid having to parse the directory structure every time the binary is executed, workspace detection is done at
//! compile-time.

use std::path::Path;
use std::{env, fs};

use camino::Utf8Path;
use cargo_metadata::{Metadata, MetadataCommand};
use walkdir::{DirEntry, WalkDir};

/// Directory names to skip in workspace discovery.
const SKIP_FOR_WORKSPACE_DISCOVERY: &[&str] = &["target", "build", "submodules"];

/// Returns `false` for directories that should not be recursed into when discovering workspaces.
///
/// Skips:
/// - non UTF8-directories
/// - hidden directories
/// - SKIP_FOR_WORKSPACE_DISCOVERY
fn workspace_discovery_filter(directory_entry: &DirEntry) -> bool {
    let Some(directory_name) = directory_entry.file_name().to_str() else {
        return false;
    };

    if directory_name.starts_with(".") {
        return false;
    }

    if SKIP_FOR_WORKSPACE_DISCOVERY.contains(&directory_name) {
        return false;
    }

    true
}

/// Recursively finds all workspaces in the repository ignoring the root directory.
fn find_workspaces(root: &Utf8Path) -> Vec<Metadata> {
    let mut workspaces = vec![];
    for manifest_path in WalkDir::new(root)
        .into_iter()
        .filter_entry(workspace_discovery_filter)
        .filter_map(|directory_entry| directory_entry.ok())
        .filter(|directory_entry| directory_entry.file_type().is_file())
        .filter(|directory_entry| directory_entry.file_name() == "Cargo.toml")
    {
        let manifest_metadata = MetadataCommand::new()
            .manifest_path(manifest_path.path())
            .exec()
            .expect("\"Cargo.toml\" should contain valid manifest data");

        // Packages that are part of a workspace will yield the same workspace root.
        // We only add each workspace root once to avoid duplicates.
        // The root workspace is skipped to avoid recursion when testing workspaces.
        if !workspaces
                .iter()
                .any(|workspace: &Metadata| workspace.workspace_root == manifest_metadata.workspace_root)
                && manifest_metadata.workspace_root != root.as_std_path()
                // Ensure that the current manifest is the workspace root manifest.
                && manifest_metadata.workspace_root == manifest_path.path().parent().unwrap().to_str().unwrap()
        {
            // This will emit duplicates for packages from the repository used in multiple workspaces, but that does
            // not do harm.
            for package in &manifest_metadata.packages {
                // Only emit rerun-if-changed for packages within the repository.
                if package.manifest_path.starts_with(root) {
                    // Emit rerun-if-changed for every package within the workspace.
                    println!("cargo::rerun-if-changed={}", package.manifest_path);
                }
            }
            // Emit rerun-if-changed for the root workspace as that might not be a package itself.
            println!(
                "cargo::rerun-if-changed={}",
                manifest_path.path().to_str().unwrap()
            );
            workspaces.push(manifest_metadata);
        }
    }
    workspaces
}

/// Detects all workspaces within the repository and stores them in `OUT_DIR/workspaces.json`.
pub fn main() {
    let manifest_dir = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let non_root_workspaces = find_workspaces(manifest_dir);

    // Print workspaces for sanity checking when running verbose builds.
    non_root_workspaces
        .iter()
        .for_each(|workspace| println!("Workspace considered in checking: {workspace:?}"));

    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).join("workspaces.json");
    fs::write(
        out_dir,
        serde_json::to_string(&non_root_workspaces).unwrap(),
    )
    .unwrap();
}
