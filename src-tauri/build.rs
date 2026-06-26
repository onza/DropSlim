fn main() {
    println!("cargo:rerun-if-changed=../assets/icon/icon-1024.png");
    println!("cargo:rerun-if-changed=../scripts/build-icons.mjs");
    println!("cargo:rerun-if-changed=icons/icon.png");
    #[cfg(target_os = "macos")]
    println!("cargo:rerun-if-changed=icons/icon.icns");
    tauri_build::build()
}
