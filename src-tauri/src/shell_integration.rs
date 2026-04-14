/// Shell integration — "Open in Primarch" context menu for Windows and macOS.
///
/// Windows: Registers shell verb entries in HKCU registry via reg.exe.
/// macOS:   Creates an Automator Quick Action workflow in ~/Library/Services/.

/// Parse `--cwd <path>` from a list of command-line arguments.
pub fn parse_cwd_arg(args: &[String]) -> Option<String> {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "--cwd" {
            return iter.next().cloned();
        }
        if let Some(path) = arg.strip_prefix("--cwd=") {
            return Some(path.to_string());
        }
    }
    None
}

// =============================================================================
// Windows
// =============================================================================

#[cfg(windows)]
pub mod platform {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const VERB_KEY: &str = "Primarch";

    // -- Registry helpers (thin wrappers around reg.exe) ----------------------

    fn reg_add(key: &str, value_name: Option<&str>, data: &str) -> Result<(), String> {
        let mut args = vec!["add", key, "/f"];
        match value_name {
            Some(name) => {
                args.push("/v");
                args.push(name);
            }
            None => args.push("/ve"),
        }
        args.push("/d");
        args.push(data);

        let output = std::process::Command::new("reg")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run reg.exe: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Registry write failed: {}", stderr.trim()));
        }
        Ok(())
    }

    fn reg_delete(key: &str) -> Result<(), String> {
        let _ = std::process::Command::new("reg")
            .args(["delete", key, "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output();
        Ok(()) // Ignore errors — key might not exist
    }

    fn reg_query(key: &str) -> bool {
        std::process::Command::new("reg")
            .args(["query", key])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    // -- Public API -----------------------------------------------------------

    /// Register "Open in Primarch" context menu entries.
    ///
    /// Creates entries for:
    /// - Right-clicking a directory
    /// - Right-clicking the background of an open directory
    pub fn install() -> Result<(), String> {
        let exe = std::env::current_exe()
            .map_err(|e| format!("Cannot determine executable path: {}", e))?;
        let exe_str = exe.to_string_lossy().to_string();
        let command = format!("\"{}\" --cwd \"%V\"", exe_str);
        let icon = format!("{},0", exe_str);

        // Right-click on a directory
        let dir_key = format!(r"HKCU\Software\Classes\Directory\shell\{}", VERB_KEY);
        reg_add(&dir_key, None, "Open in Primarch")?;
        reg_add(&dir_key, Some("Icon"), &icon)?;
        reg_add(&format!(r"{}\command", dir_key), None, &command)?;

        // Right-click on the background of an open directory
        let bg_key = format!(
            r"HKCU\Software\Classes\Directory\Background\shell\{}",
            VERB_KEY
        );
        reg_add(&bg_key, None, "Open in Primarch")?;
        reg_add(&bg_key, Some("Icon"), &icon)?;
        reg_add(&format!(r"{}\command", bg_key), None, &command)?;

        // Right-click on a drive letter
        let drive_key = format!(r"HKCU\Software\Classes\Drive\shell\{}", VERB_KEY);
        reg_add(&drive_key, None, "Open in Primarch")?;
        reg_add(&drive_key, Some("Icon"), &icon)?;
        reg_add(&format!(r"{}\command", drive_key), None, &command)?;

        Ok(())
    }

    /// Remove all "Open in Primarch" context menu entries.
    pub fn uninstall() -> Result<(), String> {
        reg_delete(&format!(
            r"HKCU\Software\Classes\Directory\shell\{}",
            VERB_KEY
        ))?;
        reg_delete(&format!(
            r"HKCU\Software\Classes\Directory\Background\shell\{}",
            VERB_KEY
        ))?;
        reg_delete(&format!(
            r"HKCU\Software\Classes\Drive\shell\{}",
            VERB_KEY
        ))?;
        Ok(())
    }

    /// Check whether the context menu entries are currently registered.
    pub fn is_installed() -> bool {
        reg_query(&format!(
            r"HKCU\Software\Classes\Directory\shell\{}\command",
            VERB_KEY
        ))
    }
}

// =============================================================================
// macOS
// =============================================================================

#[cfg(target_os = "macos")]
pub mod platform {
    use std::path::PathBuf;

    const WORKFLOW_NAME: &str = "Open in Primarch.workflow";

    fn workflow_base() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
        Ok(home.join("Library/Services").join(WORKFLOW_NAME))
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn build_document_wflow(binary_path: &str) -> String {
        let shell_script = format!(
            r#"BINARY="{binary}"
if [ ! -f "$BINARY" ]; then
    for loc in "/Applications/Primarch.app" "$HOME/Applications/Primarch.app"; do
        [ -f "$loc/Contents/MacOS/Primarch" ] && BINARY="$loc/Contents/MacOS/Primarch" && break
    done
fi
if [ ! -f "$BINARY" ]; then
    APP=$(mdfind "kMDItemCFBundleIdentifier == 'sh.primarch.terminal'" 2>/dev/null | head -1)
    [ -n "$APP" ] && BINARY="$APP/Contents/MacOS/Primarch"
fi
if [ -f "$BINARY" ]; then
    for f in "$@"; do
        "$BINARY" --cwd "$f" &
    done
fi"#,
            binary = binary_path,
        );

        let escaped_script = xml_escape(&shell_script);

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key>
	<string>523</string>
	<key>AMApplicationVersion</key>
	<string>2.10</string>
	<key>AMDocumentVersion</key>
	<string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Optional</key>
					<true/>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>AMActionVersion</key>
				<string>2.0.3</string>
				<key>AMApplication</key>
				<array>
					<string>Automator</string>
				</array>
				<key>AMBundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>AMCategory</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>AMIconName</key>
				<string>TerminalAction</string>
				<key>AMName</key>
				<string>Run Shell Script</string>
				<key>AMProvides</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>ActionBundlePath</key>
				<string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key>
				<string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key>
					<string>{script}</string>
					<key>CheckedForUserDefaultShell</key>
					<true/>
					<key>inputMethod</key>
					<integer>1</integer>
					<key>shell</key>
					<string>/bin/zsh</string>
					<key>source</key>
					<string></string>
				</dict>
				<key>BundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key>
				<string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key>
				<false/>
				<key>CanShowWhenRun</key>
				<true/>
				<key>Category</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>Class Name</key>
				<string>RunShellScriptAction</string>
				<key>InputUUID</key>
				<string>A1B2C3D4-E5F6-7890-ABCD-EF1234567890</string>
				<key>Keywords</key>
				<array>
					<string>Shell</string>
					<string>Script</string>
				</array>
				<key>OutputUUID</key>
				<string>B2C3D4E5-F6A7-8901-BCDE-F12345678901</string>
				<key>UUID</key>
				<string>C3D4E5F6-A7B8-9012-CDEF-123456789012</string>
				<key>UnlocalizedApplications</key>
				<array>
					<string>Automator</string>
				</array>
				<key>arguments</key>
				<dict>
					<key>0</key>
					<dict>
						<key>default value</key>
						<integer>0</integer>
						<key>name</key>
						<string>inputMethod</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>0</string>
					</dict>
					<key>1</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>source</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>1</string>
					</dict>
					<key>2</key>
					<dict>
						<key>default value</key>
						<false/>
						<key>name</key>
						<string>CheckedForUserDefaultShell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>2</string>
					</dict>
					<key>3</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>COMMAND_STRING</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>3</string>
					</dict>
					<key>4</key>
					<dict>
						<key>default value</key>
						<string>/bin/zsh</string>
						<key>name</key>
						<string>shell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>4</string>
					</dict>
				</dict>
				<key>isViewVisible</key>
				<true/>
				<key>location</key>
				<string>449.000000:620.000000</string>
				<key>nibPath</key>
				<string>/System/Library/Automator/Run Shell Script.action/Contents/Resources/English.lproj/main.nib</string>
			</dict>
			<key>isViewVisible</key>
			<true/>
		</dict>
	</array>
	<key>connectors</key>
	<dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>applicationBundleIDsByPath</key>
		<dict/>
		<key>applicationPaths</key>
		<array/>
		<key>inputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject.folder</string>
		<key>outputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>presentationMode</key>
		<integer>15</integer>
		<key>processesInput</key>
		<integer>0</integer>
		<key>serviceApplicationGroupName</key>
		<string>Folder</string>
		<key>serviceApplicationPath</key>
		<string>/System/Library/CoreServices/Finder.app</string>
		<key>serviceInputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject.folder</string>
		<key>serviceOutputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key>
		<integer>0</integer>
		<key>systemImageName</key>
		<string>NSActionTemplate</string>
		<key>useAutomaticInputType</key>
		<integer>0</integer>
		<key>workflowTypeIdentifier</key>
		<string>com.apple.Automator.servicesMenu</string>
	</dict>
</dict>
</plist>"#,
            script = escaped_script,
        )
    }

    const INFO_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSServices</key>
	<array>
		<dict>
			<key>NSMenuItem</key>
			<dict>
				<key>default</key>
				<string>Open in Primarch</string>
			</dict>
			<key>NSMessage</key>
			<string>runWorkflowAsService</string>
			<key>NSSendFileTypes</key>
			<array>
				<string>public.folder</string>
			</array>
		</dict>
	</array>
</dict>
</plist>"#;

    /// Install the "Open in Primarch" Finder Quick Action.
    pub fn install() -> Result<(), String> {
        let exe = std::env::current_exe()
            .map_err(|e| format!("Cannot determine executable path: {}", e))?;
        let binary_path = exe.to_string_lossy().to_string();

        let contents_dir = workflow_base()?.join("Contents");
        std::fs::create_dir_all(&contents_dir)
            .map_err(|e| format!("Failed to create workflow directory: {}", e))?;

        let wflow = build_document_wflow(&binary_path);
        std::fs::write(contents_dir.join("document.wflow"), wflow)
            .map_err(|e| format!("Failed to write workflow definition: {}", e))?;

        std::fs::write(contents_dir.join("Info.plist"), INFO_PLIST)
            .map_err(|e| format!("Failed to write Info.plist: {}", e))?;

        // Nudge LaunchServices so Finder picks up the new service immediately.
        let _ = std::process::Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
            .args(["-kill", "-r", "-domain", "local", "-domain", "user"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        Ok(())
    }

    /// Remove the "Open in Primarch" Finder Quick Action.
    pub fn uninstall() -> Result<(), String> {
        let base = workflow_base()?;
        if base.exists() {
            std::fs::remove_dir_all(&base)
                .map_err(|e| format!("Failed to remove workflow: {}", e))?;
        }
        Ok(())
    }

    /// Check whether the Quick Action is installed.
    pub fn is_installed() -> bool {
        workflow_base()
            .map(|p| p.join("Contents/document.wflow").exists())
            .unwrap_or(false)
    }
}

// =============================================================================
// macOS single-instance IPC via Unix domain socket
// =============================================================================

#[cfg(target_os = "macos")]
pub mod ipc {
    use std::io::{Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;

    fn socket_path() -> PathBuf {
        let user = std::env::var("USER").unwrap_or_else(|_| "default".to_string());
        PathBuf::from(format!("/tmp/primarch-{}.sock", user))
    }

    /// Try to forward the given CWD to an already-running Primarch instance.
    /// Returns `true` if successful (caller should exit).
    pub fn try_forward(cwd: &str) -> bool {
        let path = socket_path();
        if let Ok(mut stream) = UnixStream::connect(&path) {
            // Set a short timeout so we don't hang if the socket is stale.
            let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));
            let msg = serde_json::json!({ "cwd": cwd });
            if stream.write_all(msg.to_string().as_bytes()).is_ok() {
                return true;
            }
        }
        false
    }

    /// Start a background listener that receives open-directory intents from
    /// other Primarch processes. Emits `open-directory` events and focuses the
    /// main window.
    pub fn start_listener(app: tauri::AppHandle) {
        let path = socket_path();
        // Remove stale socket from a previous crash.
        let _ = std::fs::remove_file(&path);

        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("IPC listener failed to bind: {}", e);
                return;
            }
        };

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let _ =
                    stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
                let mut buf = String::new();
                if stream.read_to_string(&mut buf).is_err() {
                    continue;
                }
                let Ok(val) = serde_json::from_str::<serde_json::Value>(&buf) else {
                    continue;
                };
                if let Some(cwd) = val.get("cwd").and_then(|v| v.as_str()) {
                    use tauri::{Emitter, Manager};
                    let _ = app.emit("open-directory", cwd.to_string());
                    // Bring window to front.
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.unminimize();
                        let _ = win.set_focus();
                        let _ = win.show();
                    }
                }
            }
        });
    }
}

// =============================================================================
// Fallback — platforms without shell integration
// =============================================================================

#[cfg(not(any(windows, target_os = "macos")))]
pub mod platform {
    pub fn install() -> Result<(), String> {
        Err("Shell integration is not supported on this platform".into())
    }

    pub fn uninstall() -> Result<(), String> {
        Err("Shell integration is not supported on this platform".into())
    }

    pub fn is_installed() -> bool {
        false
    }
}
