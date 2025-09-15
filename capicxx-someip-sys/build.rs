//! Builds CommonAPI C++ SOME/IP Framework C/C++ libraries & installs them into the OUT_DIR.

use std::env::var as env_var;
use std::path::PathBuf;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        // vsomeip supports only Linux.
        // We don't panic here because
        // we wont fail the build for
        // unsupported systems.
        build_cmake_projects(vec![
            path_from_env_var("VSOMEIP_PATH")?,
            path_from_env_var("COMMON_API_CORE_PATH")?,
            path_from_env_var("COMMON_API_SOMEIP_PATH")?,
        ])?;
    }
    println!("cargo::metadata=install-path={}", env_var("OUT_DIR")?);
    Ok(())
}

fn build_cmake_projects(paths: Vec<PathBuf>) -> anyhow::Result<()> {
    for project_path in paths {
        let prefix_path = format!("-DCMAKE_PREFIX_PATH={}", env_var("OUT_DIR")?);
        let install_path = cmake::Config::new(project_path)
            .configure_arg(prefix_path)
            .build();

        let install_lib_path = format!("{}/lib", install_path.display());
        println!("cargo:rustc-link-search={install_lib_path}");
    }
    Ok(())
}

fn path_from_env_var(env_var_name: &str) -> anyhow::Result<PathBuf> {
    println!("cargo::rerun-if-env-changed={env_var_name}");

    let env_var_value =
        env_var(env_var_name).context(format!("{env_var_name} environment variable is not set"))?;
    let canonicalized_path = PathBuf::new()
        .join(env_var("CARGO_MANIFEST_DIR")?)
        .join(&env_var_value)
        .canonicalize()
        .context(format!(
            "path provided in {env_var_name} environment variable doesn't exist"
        ))?;
    Ok(canonicalized_path)
}
