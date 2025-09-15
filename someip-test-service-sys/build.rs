//! Builds SOME/IP test service.

use std::env::var as env_var;
use std::path::PathBuf;

use anyhow::Context;

const CAPICXX_SOMEIP_ENV_KEY: &str = "DEP_CAPICXX_SOMEIP_INSTALL_PATH";

fn main() -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        // CommonAPI C++ SOME/IP Framework supports only Linux.
        build_test_service()?;
    } else {
        // We don't panic here because we don't want to fail
        // the build for other platforms. Instead we provide
        // a stub implementation of the test service.
        build_test_service_stub()?;
    }
    generate_bindings()?;
    setup_rebuild_rules()?;
    Ok(())
}

fn build_test_service() -> anyhow::Result<()> {
    build_cmake_project(PathBuf::from("cpp/service").canonicalize()?)?;
    link_libraries(vec![
        "CommonAPI",
        "CommonAPI-SomeIP",
        "someip-test-service",
        "someip-test-service-someip",
    ]);
    Ok(())
}

fn build_test_service_stub() -> anyhow::Result<()> {
    build_cmake_project(PathBuf::from("cpp/service_stub").canonicalize()?)?;
    link_libraries(vec!["someip-test-service"]);
    Ok(())
}

fn build_cmake_project(project_path: PathBuf) -> anyhow::Result<()> {
    let capicxx_someip_install_path = env_var(CAPICXX_SOMEIP_ENV_KEY).context(format!(
        "{CAPICXX_SOMEIP_ENV_KEY} environment variable is not set"
    ))?;
    let prefix_path = format!("-DCMAKE_PREFIX_PATH={capicxx_someip_install_path}");
    let install_path = cmake::Config::new(project_path)
        .configure_arg(prefix_path)
        .build();
    let install_lib_path = format!("{}/lib", install_path.display());
    println!("cargo:rustc-link-search=native={install_lib_path}");
    Ok(())
}

fn link_libraries(libraries: Vec<&str>) {
    for library in libraries {
        println!("cargo:rustc-link-lib={library}");
    }
}

fn generate_bindings() -> anyhow::Result<()> {
    bindgen::Builder::default()
        .header("cpp/interface.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .context("failed to generate bindings")?
        .write_to_file(PathBuf::from(env_var("OUT_DIR")?).join("bindings.rs"))?;
    Ok(())
}

fn setup_rebuild_rules() -> anyhow::Result<()> {
    let rerun_if_changed = vec![
        PathBuf::from("cpp").canonicalize()?,
        PathBuf::from("src").canonicalize()?,
    ];
    for path in rerun_if_changed {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    Ok(())
}
