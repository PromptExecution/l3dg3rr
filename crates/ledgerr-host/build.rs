fn main() {
    #[cfg(target_os = "windows")]
    tauri_build::build();

    // Bake build counter into binary at compile time
    let counter_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join(".build_counter"))
        .unwrap_or_else(|| std::path::PathBuf::from(".build_counter"));

    let build_num = if counter_path.exists() {
        std::fs::read_to_string(&counter_path)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0)
    } else {
        0
    };

    println!("cargo:rustc-env=TAURI_BUILD_NUMBER={}", build_num);
}
