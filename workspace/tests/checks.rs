#![allow(missing_docs, reason = "this is a test crate")]

use std::process::Command;
use std::str::FromStr;

use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use libtest_mimic::Trial;

/// All environment variables that should not be inherited from the parent `cargo` invocation.
const NON_INHERITABLE_ENV_VARS: &[&str] = &[
    "LIB_FREERTOS_NAME",
    "LIB_FREERTOS_SEARCH_PATH",
    "FREERTOS_CONFIG_INCLUDE_PATH",
    "FREERTOS_KERNEL_INCLUDE_PATH",
    "FREERTOS_KERNEL_PORTMACRO_INCLUDE_PATH",
    "FREERTOS_HEAP_FILE_PATH",
    "FREERTOS_ADDITIONAL_INCLUDE_PATHS",
    "FREERTOS_ADDITIONAL_INCLUDE_PATHS_BASE",
    "FREERTOS_BINDINGS_LOCATION",
    "BINDINGS_WRAPPER_PREPEND_EXTENSION_PATH",
];

trait CommandExt {
    /// Runs a command, maybe capturing output from it and returning as `Failed`.
    fn run_as_test(&mut self, capture: bool) -> Result<(), libtest_mimic::Failed>;
    /// Removes all environment variables listed in `NON_INHERITABLE_ENV_VARS` from the command.
    fn clear_non_inheritable_env_vars(&mut self) -> &mut Self;
}

impl CommandExt for std::process::Command {
    fn run_as_test(&mut self, capture: bool) -> Result<(), libtest_mimic::Failed> {
        let (status, stdout, stderr) = if capture {
            let output = self.output()?;

            let stdout = String::from_utf8(output.stdout)?;
            let stderr = String::from_utf8(output.stderr)?;

            (output.status, stdout, stderr)
        } else {
            (self.status()?, String::new(), String::new())
        };

        status.success().then_some(()).ok_or_else(|| {
            let mut message = format!("Running {self:?} {status}");

            if !stdout.is_empty() {
                message.push_str("\n==== stdout ====\n");
                message.push_str(&stdout);
            }
            if !stderr.is_empty() {
                message.push_str("\n==== stderr ====\n");
                message.push_str(&stderr);
            }

            message.into()
        })
    }

    fn clear_non_inheritable_env_vars(&mut self) -> &mut Self {
        for env_var in NON_INHERITABLE_ENV_VARS {
            self.env_remove(env_var);
        }
        self
    }
}

fn get_workspace_lints(manifest: &toml::Table) -> Option<bool> {
    manifest.get("lints")?.get("workspace")?.as_bool()
}

/// Creates trials for the various `cargo` checks we want to run for each `package` that is in the root workspace.
fn make_root_package_trials(base: &Utf8Path, package: &Package, capture: bool) -> Vec<Trial> {
    let dir = package.manifest_path.parent().unwrap();
    let package_relative = dir.strip_prefix(base).unwrap();
    let relative = format_args!("veecle_os::{package_relative}");

    let mut trials = vec![Trial::test(format!("{relative}::workspace-lints"), {
        let manifest_path = package.manifest_path.to_owned();
        move || {
            let text = std::fs::read_to_string(&manifest_path)?;
            let manifest = toml::Table::from_str(&text)?;
            get_workspace_lints(&manifest)
                .unwrap_or(false)
                .then_some(())
                .ok_or_else(|| "Missing `workspace.lints = true`".into())
        }
    })];

    if package
        .metadata
        .pointer("/workspace-checks/miri")
        .and_then(|miri| miri.as_bool())
        .unwrap_or(true)
    {
        trials.push(
            Trial::test(format!("{relative}::miri"), {
                let dir = package.manifest_path.parent().unwrap().to_owned();
                move || {
                    Command::new("cargo")
                        .args([
                            "miri",
                            "nextest",
                            "run",
                            "--no-fail-fast",
                            "--no-tests=warn",
                        ])
                        .current_dir(dir)
                        .clear_non_inheritable_env_vars()
                        .run_as_test(capture)
                }
            })
            .with_ignored_flag(true),
        );
    }

    trials
}

/// Gets the package definitions for all members of the workspace.
fn workspace_packages(metadata: &Metadata) -> impl Iterator<Item = &Package> {
    metadata
        .workspace_members
        .iter()
        .map(|id| metadata.packages.iter().find(|p| &p.id == id).unwrap())
}

/// Creates trials for the various `cargo` checks we want to run for each workspace.
fn make_workspace_trials(base: &Utf8Path, workspace: &Metadata, capture: bool) -> Vec<Trial> {
    let workspace_directory = &workspace.workspace_root;
    let mut relative = workspace_directory.strip_prefix(base).unwrap().to_owned();
    // Root workspace (`veecle-os`) if `relative` is empty.
    if relative.as_str() == "" {
        relative = Utf8PathBuf::from("veecle-os");
    }

    let workspace_packages = workspace_packages(workspace).collect::<Vec<_>>();
    let mut trials = vec![
        Trial::test(format!("{relative}::clippy"), {
            let dir = workspace_directory.to_owned();
            move || {
                Command::new("cargo")
                    .args([
                        "clippy",
                        "--all-targets",
                        "--workspace",
                        "--all-features",
                        "--color=always",
                        "--keep-going",
                        "--",
                        "-Dwarnings",
                    ])
                    .current_dir(dir)
                    .clear_non_inheritable_env_vars()
                    .run_as_test(capture)
            }
        }),
        Trial::test(format!("{relative}::doc"), {
            let dir = workspace_directory.to_owned();
            move || {
                Command::new("cargo")
                    .args([
                        "doc",
                        "--no-deps",
                        "--workspace",
                        "--all-features",
                        "--color=always",
                        "--keep-going",
                    ])
                    .env("RUSTDOCFLAGS", "-Dwarnings --document-private-items")
                    .current_dir(dir)
                    .clear_non_inheritable_env_vars()
                    .run_as_test(capture)
            }
        }),
        Trial::test(format!("{relative}::licenses"), {
            let dir = workspace_directory.to_owned();
            move || {
                Command::new("cargo")
                    .args(["deny", "--workspace", "check", "licenses"])
                    // not all licenses are used in every workspace
                    .args([
                        "--allow=license-exception-not-encountered",
                        "--allow=license-not-encountered",
                    ])
                    .current_dir(dir)
                    .clear_non_inheritable_env_vars()
                    .run_as_test(capture)
            }
        }),
        Trial::test(format!("{relative}::advisories"), {
            let dir = workspace_directory.to_owned();
            move || {
                Command::new("cargo")
                    .args(["deny", "--workspace", "check", "advisories"])
                    .current_dir(dir)
                    .clear_non_inheritable_env_vars()
                    .run_as_test(capture)
            }
        }),
    ];

    for package_manifest_path in workspace_packages
        .iter()
        .map(|package| package.manifest_path.clone())
    {
        let package_relative = package_manifest_path
            .strip_prefix(base)
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let relative = format_args!("{relative}::{package_relative}");

        trials.push(Trial::test(format!("{relative}::cargo-config-toml"), {
            let workspace_root = workspace_directory.to_owned();
            let mut manifest_directory = package_manifest_path.parent().unwrap().to_owned();

            move || {
                // Search the package manifest directory and all parent directories for ".cargo/config.toml" until the
                // workspace root is reached.
                while manifest_directory != workspace_root {
                    if manifest_directory.join(".cargo/config.toml").exists() {
                        return Err(format!(
                            r#"".cargo/config.toml" only allowed at the workspace root level: "{manifest_directory}/.cargo/config.toml""#
                        )
                        .into());
                    }
                    if !manifest_directory.pop() {
                        panic!("crate directory does not seem to be a sub-directory of the workspace");
                    }
                }

                Ok(())
            }
        }));

        // Run `cargo fmt` per crate to include which crate failed the formatting check in the test summary.
        trials.push(Trial::test(format!("{relative}::fmt"), {
            let dir = package_manifest_path.parent().unwrap().to_owned();
            move || {
                Command::new("cargo")
                    .args(["fmt", "--check"])
                    .current_dir(dir)
                    .clear_non_inheritable_env_vars()
                    .run_as_test(capture)
            }
        }));
    }

    // Run doc-tests for each crate as `nextest` does not include them itself.
    for package in workspace_packages
        .iter()
        .filter(|package| package.targets.iter().any(|target| target.is_lib()))
    {
        let package_relative = package
            .manifest_path
            .strip_prefix(base)
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let relative = format_args!("{relative}::{package_relative}");
        let package_name = package.name.clone();

        trials.push(Trial::test(format!("{relative}::doc-test"), {
            let dir = workspace_directory.to_owned();
            move || {
                Command::new("cargo")
                    .args(["test", "--doc", "-p", &package_name])
                    .current_dir(dir)
                    .clear_non_inheritable_env_vars()
                    .run_as_test(capture)
            }
        }));
    }

    // Build binaries for each crate to verify they compile successfully.
    for package in workspace_packages
        .iter()
        .filter(|package| package.targets.iter().any(|target| target.is_bin()))
    {
        let package_relative = package
            .manifest_path
            .strip_prefix(base)
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let relative = format_args!("{relative}::{package_relative}");
        let package_name = package.name.clone();

        trials.push(Trial::test(format!("{relative}::build-bins"), {
            let dir = workspace_directory.to_owned();
            move || {
                Command::new("cargo")
                    .args(["build", "--bins", "-p", &package_name])
                    .current_dir(dir)
                    .clear_non_inheritable_env_vars()
                    .run_as_test(capture)
            }
        }));
    }

    // Creates a test for each publishable package (`package.publish.is_none()`), to verify that it does not depend on non-publishable packages (`is_some()`).
    for package in workspace_packages
        .iter()
        .filter(|package| package.publish.is_none())
        .copied()
    {
        let package_manifest_path = package.manifest_path.clone();
        let package_relative = package_manifest_path
            .strip_prefix(base)
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let relative = format_args!("{relative}::{package_relative}");
        let package = package.clone();
        let workspace_packages: Vec<Package> =
            workspace_packages.iter().copied().cloned().collect();

        trials.push(Trial::test(format!("{relative}::publishable"), {
            move || {
                let mut unpublishable_dependencies = vec![];
                for dependency in &package.dependencies {
                    let Some(workspace_dependency) = workspace_packages
                        .iter()
                        .find(|workspace_package| *workspace_package.name == dependency.name)
                    else {
                        continue;
                    };

                    if workspace_dependency.publish.is_some() {
                        unpublishable_dependencies.push(dependency.name.clone());
                    }
                }
                if unpublishable_dependencies.is_empty() {
                    Ok(())
                } else {
                    Err(
                        format!("has non-publish dependencies: {unpublishable_dependencies:?}")
                            .into(),
                    )
                }
            }
        }));
    }

    trials
}

/// Creates trials for the `cargo` checks we want to run for each non-root workspace.
fn make_non_root_workspace_trials(
    base: &Utf8Path,
    workspace: &Metadata,
    capture: bool,
) -> Vec<Trial> {
    let workspace_directory = &workspace.workspace_root;
    let relative = workspace_directory.strip_prefix(base).unwrap();

    vec![Trial::test(format!("{relative}::test"), {
        let dir = workspace_directory.to_owned();
        move || {
            Command::new("cargo")
                .args([
                    "nextest",
                    "run",
                    "--workspace",
                    "--no-fail-fast",
                    "--no-tests=warn",
                ])
                .current_dir(dir)
                .clear_non_inheritable_env_vars()
                .run_as_test(capture)
        }
    })]
}

/// Get the `cargo metadata` output for a workspace in `dir`.
fn load_metadata(dir: &Utf8Path) -> Metadata {
    MetadataCommand::new()
        .manifest_path(dir.join("Cargo.toml"))
        .exec()
        .unwrap()
}

fn main() -> std::process::ExitCode {
    let args = libtest_mimic::Arguments::from_args();
    let capture = !args.nocapture;

    let manifest_dir = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let root_workspace = load_metadata(manifest_dir);

    let serialized_workspaces = include_str!(concat!(env!("OUT_DIR"), "/workspaces.json"));
    let non_root_workspaces: Vec<Metadata> = serde_json::from_str(serialized_workspaces).unwrap();

    // For all non-root workspaces generate trials running tests from within the respective workspace root directory.
    // The root workspace itself is assumed to be running concurrently via `cargo test --workspace`.
    let non_root_workspace_trials = non_root_workspaces
        .iter()
        .flat_map(|workspace| make_non_root_workspace_trials(manifest_dir, workspace, capture));

    // Generate trials checking every workspace including the root workspace.
    let all_workspaces_trials = if cfg!(coverage) {
        vec![]
    } else {
        make_workspace_trials(manifest_dir, &root_workspace, capture)
            .into_iter()
            .chain(
                non_root_workspaces
                    .iter()
                    .flat_map(|workspace| make_workspace_trials(manifest_dir, workspace, capture)),
            )
            .collect()
    };

    // For the packages in the root workspace we have some extra trials.
    let root_package_trials = if cfg!(coverage) {
        vec![]
    } else {
        workspace_packages(&root_workspace)
            .flat_map(|package| make_root_package_trials(manifest_dir, package, capture))
            .collect()
    };

    // Trials that run on the repository as a whole.
    // Skipped in coverage runs.
    let global_trials = if cfg!(coverage) {
        vec![]
    } else {
        vec![
            Trial::test("veecle_os::tombi::fmt", move || {
                Command::new("tombi")
                    .args(["format", "--offline", "--check"])
                    .current_dir(manifest_dir)
                    .run_as_test(capture)
            }),
            Trial::test("veecle_os::tombi::lint", move || {
                Command::new("tombi")
                    .args(["lint", "--offline"])
                    .current_dir(manifest_dir)
                    .run_as_test(capture)
            }),
        ]
    };

    // Trials that run on the root workspace.
    // Skipped in coverage runs.
    let root_workspace_trials = if cfg!(coverage) {
        vec![]
    } else {
        vec![Trial::test("veecle_os::vet", {
            move || {
                Command::new("cargo")
                    .args(["vet", "check", "--locked"])
                    .current_dir(manifest_dir)
                    .run_as_test(capture)
            }
        })]
    };

    let trials: Vec<Trial> = non_root_workspace_trials
        .chain(all_workspaces_trials)
        .chain(root_package_trials)
        .chain(global_trials)
        .chain(root_workspace_trials)
        .collect();

    libtest_mimic::run(&args, trials).exit_code()
}
