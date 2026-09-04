fn main() {
    let helper = std::path::Path::new("binaries/cyvra-purge-helper.exe");
    println!("cargo:rerun-if-changed={}", helper.display());
    if !helper.exists() {
        std::fs::create_dir_all("binaries").expect("CYVRA binaries directory");
        // Placeholder so cargo check can run before the helper is staged.
        // npm run stage:purge-helper (beforeBuildCommand) replaces this with the real helper.
        std::fs::write(helper, []).expect("CYVRA helper placeholder");
    }
    tauri_build::build();
}
