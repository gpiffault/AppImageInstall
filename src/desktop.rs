use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{applications_dir, icons_dir};

#[derive(Debug)]
pub struct DesktopEntry {
    pub path: PathBuf,
    pub name: String,
    pub exec: String,
    pub icon: String,
}

pub fn write_desktop_entry(content: &str) -> Result<(), String> {
    let name = desktop_field(content, "Name");
    if name.is_empty() {
        return Err("no Name= found in desktop content".to_string());
    }

    let safe_name = name.replace(' ', "_");
    let desktop_path = format!("{}/{}.desktop", applications_dir(), safe_name);
    let temp_path = format!("{}.tmp", desktop_path);

    fs::write(&temp_path, content)
        .map_err(|e| format!("write temp desktop file: {}", e))?;

    fs::rename(&temp_path, &desktop_path).inspect_err(|_| {
        let _ = fs::remove_file(&temp_path);
    }).map_err(|e| format!("rename temp to final: {}", e))?;

    let _ = Command::new("update-desktop-database")
        .arg(applications_dir())
        .output();

    Ok(())
}

pub fn desktop_field(content: &str, key: &str) -> String {
    let prefix = format!("{}=", key);
    for line in content.lines() {
        if let Some(value) = line.strip_prefix(&prefix) {
            return value.to_string();
        }
    }
    String::new()
}

pub fn modify_desktop_content(desktop_content: &str, app_image_path: &str, icon_path: &str) -> String {
    let mut out = String::new();

    for line in desktop_content.lines() {
        if let Some(val) = line.strip_prefix("Exec=") {
            let new_val = if val.starts_with("AppRun") {
                val.replacen("AppRun", &format!("\"{}\"", app_image_path), 1)
            } else {
                format!("\"{}\" %U", app_image_path)
            };
            out.push_str(&format!("Exec={}\n", new_val));
        } else if line.starts_with("TryExec=") {
            continue;
        } else if line.starts_with("Icon=") && !icon_path.is_empty() {
            out.push_str(&format!("Icon={}\n", icon_path));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

pub fn list_all_desktop_entries() -> io::Result<Vec<DesktopEntry>> {
    let mut entries = Vec::new();
    let dir = applications_dir();

    if let Ok(dir_entries) = fs::read_dir(&dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                if let Some(e) = parse_desktop_entry(&path) {
                    entries.push(e);
                }
            }
        }
    }

    Ok(entries)
}

fn parse_desktop_entry(path: &Path) -> Option<DesktopEntry> {
    let content = fs::read_to_string(path).ok()?;
    let mut entry = DesktopEntry {
        path: path.to_path_buf(),
        name: String::new(),
        exec: String::new(),
        icon: String::new(),
    };

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("Name=") {
            entry.name = value.to_string();
        } else if let Some(value) = line.strip_prefix("Exec=") {
            entry.exec = value.to_string();
        } else if let Some(value) = line.strip_prefix("Icon=") {
            entry.icon = value.to_string();
        }
    }

    Some(entry)
}

pub fn is_appimage_referenced(app_image_path: &str) -> bool {
    let entries = list_all_desktop_entries().unwrap_or_default();
    let app_file_name = Path::new(app_image_path)
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    for entry in &entries {
        let exec_path = exec_line_to_path(&entry.exec);
        let exec_file_name = Path::new(&exec_path)
            .file_name()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        if exec_file_name == app_file_name {
            return true;
        }
    }
    false
}

pub fn remove_desktop_entry(entry: &DesktopEntry) -> io::Result<()> {
    if !entry.icon.is_empty() && entry.icon.starts_with(&icons_dir()) {
        let _ = fs::remove_file(&entry.icon);
    }

    fs::remove_file(&entry.path)?;

    let _ = Command::new("update-desktop-database")
        .arg(applications_dir())
        .output();

    Ok(())
}

pub fn exec_line_to_path(exec_line: &str) -> String {
    let mut result = String::new();
    let mut in_quote = false;

    for ch in exec_line.chars() {
        if ch == '"' {
            in_quote = !in_quote;
            continue;
        }
        if ch == ' ' && !in_quote {
            if !result.is_empty() {
                break;
            }
            continue;
        }
        result.push(ch);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_field_name() {
        let content = "[Desktop Entry]\nName=Firefox\nExec=firefox %U\n";
        assert_eq!(desktop_field(content, "Name"), "Firefox");
    }

    #[test]
    fn test_desktop_field_exec() {
        let content = "[Desktop Entry]\nName=App\nExec=AppRun\n";
        assert_eq!(desktop_field(content, "Exec"), "AppRun");
    }

    #[test]
    fn test_desktop_field_missing() {
        let content = "[Desktop Entry]\nName=App\n";
        assert_eq!(desktop_field(content, "Icon"), "");
    }

    #[test]
    fn test_desktop_field_empty() {
        assert_eq!(desktop_field("", "Name"), "");
    }

    #[test]
    fn test_modify_desktop_content_apprun() {
        let content = "[Desktop Entry]\nName=Test App\nExec=AppRun\nIcon=old\nTryExec=old\n";
        let result = modify_desktop_content(content, "/path/to/app.AppImage", "/path/to/icon.png");
        assert!(result.contains("Exec=\"/path/to/app.AppImage\""));
        assert!(!result.contains("AppRun"));
        assert!(!result.contains("TryExec="));
        assert!(result.contains("Icon=/path/to/icon.png"));
        assert!(result.contains("Name=Test App"));
    }

    #[test]
    fn test_modify_desktop_content_non_apprun() {
        let content = "[Desktop Entry]\nName=App\nExec=/usr/bin/app\n";
        let result = modify_desktop_content(content, "/path/to/app.AppImage", "");
        assert!(result.contains("Exec=\"/path/to/app.AppImage\" %U"));
        assert!(!result.contains("/usr/bin/app"));
    }

    #[test]
    fn test_modify_desktop_content_keeps_icon_when_empty() {
        let content = "[Desktop Entry]\nIcon=original\n";
        let result = modify_desktop_content(content, "/path/to/app.AppImage", "");
        assert!(result.contains("Icon=original"));
    }

    #[test]
    fn test_exec_line_to_path_quoted() {
        assert_eq!(exec_line_to_path("\"/path/to/app\" --arg"), "/path/to/app");
    }

    #[test]
    fn test_exec_line_to_path_unquoted() {
        assert_eq!(exec_line_to_path("/path/to/app %U"), "/path/to/app");
    }

    #[test]
    fn test_exec_line_to_path_no_args() {
        assert_eq!(exec_line_to_path("/path/to/app"), "/path/to/app");
    }

    #[test]
    fn test_exec_line_to_path_apprun() {
        assert_eq!(exec_line_to_path("AppRun --flag"), "AppRun");
    }

    #[test]
    fn test_exec_line_to_path_leading_spaces() {
        assert_eq!(exec_line_to_path("  /path/to/app"), "/path/to/app");
    }

    #[test]
    fn test_exec_line_to_path_empty() {
        assert_eq!(exec_line_to_path(""), "");
    }
}
