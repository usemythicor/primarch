// Prevents additional console window on Windows, DO NOT REMOVE!!
#![windows_subsystem = "windows"]

fn main() {
    // On macOS, the single-instance plugin is not supported, so we use a
    // Unix domain socket to forward --cwd intents to an already-running
    // instance. This check must happen BEFORE Tauri initializes.
    #[cfg(target_os = "macos")]
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(cwd) = primarch_lib::shell_integration::parse_cwd_arg(&args) {
            if primarch_lib::shell_integration::ipc::try_forward(&cwd) {
                // Successfully delivered to the running instance — exit.
                return;
            }
            // No running instance found; fall through and start normally.
        }
    }

    primarch_lib::run()
}
