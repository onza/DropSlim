fn main() {
    println!("cargo:rerun-if-changed=../assets/icon/dropslim-icon.svg");
    println!("cargo:rerun-if-changed=icons/icon.png");
    println!("cargo:rerun-if-changed=icons/icon.icns");
    tauri_build::build()
}
