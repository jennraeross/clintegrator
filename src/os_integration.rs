use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum OsIntegrationStatus {
    Installed(PathBuf),
    NotInstalled,
    Outdated(PathBuf), // installed, but points to different binary path
}

pub struct OsIntegration;

impl OsIntegration {
    /// Returns current executable path or fallback
    pub fn current_executable() -> PathBuf {
        std::env::current_exe().unwrap_or_else(|_| PathBuf::from("clintegrator"))
    }

    /// Checks the OS integration status
    pub fn status() -> OsIntegrationStatus {
        #[cfg(target_os = "linux")]
        {
            let desktop_path = Self::linux_desktop_file_path();
            if desktop_path.exists() {
                if let Ok(content) = fs::read_to_string(&desktop_path) {
                    let current_exe = Self::current_executable();
                    let current_exe_str = current_exe.to_string_lossy();
                    if content.contains(&*current_exe_str) {
                        return OsIntegrationStatus::Installed(desktop_path);
                    } else {
                        return OsIntegrationStatus::Outdated(desktop_path);
                    }
                }
                return OsIntegrationStatus::Installed(desktop_path);
            }
            OsIntegrationStatus::NotInstalled
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, check registry or status
            OsIntegrationStatus::NotInstalled
        }

        #[cfg(target_os = "macos")]
        {
            OsIntegrationStatus::NotInstalled
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            OsIntegrationStatus::NotInstalled
        }
    }

    /// Linux desktop file path (~/.local/share/applications/clintegrator.desktop)
    pub fn linux_desktop_file_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("applications")
            .join("clintegrator.desktop")
    }

    /// Installs Linux desktop integration
    pub fn install_linux_desktop() -> Result<PathBuf, String> {
        let dest = Self::linux_desktop_file_path();
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let exe = Self::current_executable();
        let content = format!(
            r#"[Desktop Entry]
Type=Application
Name=Clintegrator
GenericName=Terminal File Opener
Comment=Open any filetype in your preferred terminal command
Exec="{}" %F
Icon=utilities-terminal
Terminal=false
Categories=Utility;System;Development;
StartupNotify=false
NoDisplay=false
# Broad file type and mime-type definitions so OS and file managers offer Clintegrator for all files:
MimeType=application/octet-stream;text/plain;text/markdown;text/x-shellscript;text/x-python;application/json;application/xml;application/x-zerosize;*/*;all/all;all/allfiles;
"#,
            exe.to_string_lossy()
        );

        fs::write(&dest, content).map_err(|e| format!("Failed to write desktop file: {}", e))?;

        // Update desktop database if available
        if let Some(parent) = dest.parent() {
            let _ = Command::new("update-desktop-database")
                .arg(parent)
                .output();
        }

        Ok(dest)
    }

    /// Uninstalls Linux desktop integration
    pub fn uninstall_linux_desktop() -> Result<(), String> {
        let dest = Self::linux_desktop_file_path();
        if dest.exists() {
            fs::remove_file(&dest).map_err(|e| format!("Failed to remove desktop file: {}", e))?;
            if let Some(parent) = dest.parent() {
                let _ = Command::new("update-desktop-database")
                    .arg(parent)
                    .output();
            }
        }
        Ok(())
    }

    /// Generates Windows Registry (.reg) content to associate Clintegrator with all files
    pub fn generate_windows_reg_content() -> String {
        let exe = Self::current_executable();
        let exe_escaped = exe.to_string_lossy().replace('\\', "\\\\");

        format!(
            r#"Windows Registry Editor Version 5.00

; Register Clintegrator for all files (*) in Explorer context menu and Open With
[HKEY_CURRENT_USER\Software\Classes\*\shell\Open with Clintegrator]
@="Open with Clintegrator"
"Icon"="\"{exe_escaped}\",0"

[HKEY_CURRENT_USER\Software\Classes\*\shell\Open with Clintegrator\command]
@="\"{exe_escaped}\" \"%1\""

; Register Clintegrator as a standard application
[HKEY_CURRENT_USER\Software\Classes\Applications\clintegrator.exe]
"FriendlyAppName"="Clintegrator"

[HKEY_CURRENT_USER\Software\Classes\Applications\clintegrator.exe\SupportedTypes]
"*"=""

[HKEY_CURRENT_USER\Software\Classes\Applications\clintegrator.exe\shell\open\command]
@="\"{exe_escaped}\" \"%1\""
"#,
            exe_escaped = exe_escaped
        )
    }

    /// Writes Windows Registry .reg file to desktop or user dir
    pub fn export_windows_reg_file(dest: &Path) -> Result<(), String> {
        let content = Self::generate_windows_reg_content();
        fs::write(dest, content).map_err(|e| format!("Failed to export .reg file: {}", e))?;
        Ok(())
    }

    /// Generates macOS Info.plist content for Clintegrator.app bundle
    #[allow(dead_code)]
    pub fn generate_macos_info_plist() -> String {
        let exe = Self::current_executable();
        let exe_name = exe.file_name().and_then(|n| n.to_str()).unwrap_or("clintegrator");

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>io.clintegrator.app</string>
    <key>CFBundleName</key>
    <string>Clintegrator</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>All Files</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.item</string>
                <string>public.data</string>
                <string>public.content</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#,
            exe_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_reg_content() {
        let content = OsIntegration::generate_windows_reg_content();
        assert!(content.contains("Windows Registry Editor Version 5.00"));
        assert!(content.contains(r"Software\Classes\*\shell\Open with Clintegrator"));
        assert!(content.contains(r"SupportedTypes"));
    }

    #[test]
    fn test_macos_info_plist() {
        let content = OsIntegration::generate_macos_info_plist();
        assert!(content.contains("CFBundleDocumentTypes"));
        assert!(content.contains("public.item"));
        assert!(content.contains("public.data"));
    }
}
