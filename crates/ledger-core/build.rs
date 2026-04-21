use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    if env::var_os("CARGO_FEATURE_LEGAL_Z3").is_none() {
        return;
    }

    println!("cargo:rerun-if-env-changed=Z3_LIBRARY_PATH_OVERRIDE");
    println!("cargo:rerun-if-env-changed=LIBRARY_PATH");
    println!("cargo:rerun-if-env-changed=LD_LIBRARY_PATH");

    if z3_available() {
        return;
    }

    let os_hint = if cfg!(target_os = "linux") && ubuntu_like() {
        " On Ubuntu/WSL install it with: sudo apt install -y libz3-dev"
    } else if cfg!(target_os = "linux") {
        " Install the system Z3 development package for your distribution."
    } else if cfg!(target_os = "windows") {
        " On Windows, install Z3 with vcpkg or provide Z3_LIBRARY_PATH_OVERRIDE."
    } else if cfg!(target_os = "macos") {
        " On macOS, install it with Homebrew: brew install z3."
    } else {
        " Install native Z3 development libraries or provide Z3_LIBRARY_PATH_OVERRIDE."
    };

    println!(
        "cargo:warning=ledger-core legal-z3 feature is enabled, but native libz3 was not found. Linked tests/builds may fail with `unable to find library -lz3`.{os_hint}"
    );
}

fn z3_available() -> bool {
    env_path_has_lib("Z3_LIBRARY_PATH_OVERRIDE")
        || env_path_has_lib("LIBRARY_PATH")
        || env_path_has_lib("LD_LIBRARY_PATH")
        || pkg_config_finds_z3()
        || common_lib_path_has_z3()
}

fn env_path_has_lib(var: &str) -> bool {
    let Some(paths) = env::var_os(var) else {
        return false;
    };

    env::split_paths(&paths).any(|path| has_z3_library(&path))
}

fn pkg_config_finds_z3() -> bool {
    Command::new("pkg-config")
        .args(["--exists", "z3"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn common_lib_path_has_z3() -> bool {
    [
        "/usr/lib",
        "/usr/lib64",
        "/usr/local/lib",
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
    ]
    .iter()
    .map(Path::new)
    .any(has_z3_library)
}

fn has_z3_library(path: &Path) -> bool {
    ["libz3.so", "libz3.a", "z3.lib", "libz3.dylib"]
        .iter()
        .any(|name| path.join(name).exists())
}

fn ubuntu_like() -> bool {
    let Ok(os_release) = std::fs::read_to_string("/etc/os-release") else {
        return false;
    };

    os_release.contains("ID=ubuntu") || os_release.contains("ID_LIKE=ubuntu")
}
