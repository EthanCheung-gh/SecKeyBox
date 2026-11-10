//! Application lifecycle commands.

/// Restart the app after an update.
/// On Linux AppImage the old mount's LD_LIBRARY_PATH/APPDIR leaks into the
/// relaunch and makes /usr/bin/env load an incompatible libsystemd. Clear the
/// AppImage environment before restarting; the new AppImage runtime sets its
/// own values.
#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("APPIMAGE").is_some() {
            for key in ["LD_LIBRARY_PATH", "LD_PRELOAD", "APPDIR", "APPIMAGE", "ARGV0"] {
                std::env::remove_var(key);
            }
        }
    }
    app.restart();
}
