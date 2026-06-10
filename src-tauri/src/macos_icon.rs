/// Dev-only: set dock icon from the macOS-formatted `icon.png` (82% art grid + rounded corners).
#[cfg(all(target_os = "macos", debug_assertions))]
pub fn set_dock_icon() {
    use objc2::AnyThread;
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    const ICON: &[u8] = include_bytes!("../icons/icon.png");

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let data = NSData::with_bytes(ICON);
    let icon = NSImage::initWithData(NSImage::alloc(), &data).expect("dock icon png");

    unsafe {
        app.setApplicationIconImage(Some(&icon));
    }
}

#[cfg(not(all(target_os = "macos", debug_assertions)))]
pub fn set_dock_icon() {}
