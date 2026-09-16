#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK's DMABUF renderer fails to initialize on NVIDIA + Wayland
    // (GBM/EGL allocation errors, blank or crashed windows). Force the
    // stable rendering path on Linux; harmless elsewhere.
    // Escalation if still broken: also set WEBKIT_DISABLE_COMPOSITING_MODE=1,
    // or run under XWayland with GDK_BACKEND=x11.
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    gm_toolkit_lib::run();
}
