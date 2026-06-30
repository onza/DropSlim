#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod desktop;

#[cfg(target_os = "macos")]
pub use macos::{pick_paths, pick_save_folder};

#[cfg(not(target_os = "macos"))]
pub use desktop::{pick_paths, pick_save_folder};
