use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use glob::glob;

pub struct AppImageMount {
    pub child: Child,
    pub mount_point: String,
}

impl Drop for AppImageMount {
    fn drop(&mut self) {
        let _ = self.child.kill();
        thread::sleep(Duration::from_millis(500));
        let _ = self.child.wait();
        for cmd in &["fusermount", "fusermount3"] {
            let _ = Command::new(cmd).args(["-u", &self.mount_point]).output();
        }
    }
}

pub fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        return meta.permissions().mode() & 0o111 != 0;
    }

    if !path.contains(std::path::MAIN_SEPARATOR) && look_path(path).is_some() {
        return true;
    }

    false
}

fn look_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").ok()?;
    for dir in path_var.split(':') {
        let full = Path::new(dir).join(name);
        if full.is_file() {
            if let Ok(meta) = fs::metadata(&full) {
                if meta.permissions().mode() & 0o111 != 0 {
                    return Some(full);
                }
            }
        }
    }
    None
}

pub fn mount_appimage(path: &str) -> Result<AppImageMount, String> {
    if !is_executable(path) {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("cannot make executable: {}", e))?;
    }

    let mut child = Command::new(path)
        .arg("--appimage-mount")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("start mount: {}", e))?;

    thread::sleep(Duration::from_secs(1));

    let stdout = child.stdout.take().ok_or_else(|| {
        let _ = child.kill();
        "stdout pipe unavailable".to_string()
    })?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| {
        let _ = child.kill();
        format!("read mount point: {}", e)
    })?;

    let mount_point = line.trim().to_string();
    if mount_point.is_empty() {
        let _ = child.kill();
        return Err("empty mount point".to_string());
    }

    Ok(AppImageMount {
        child,
        mount_point,
    })
}

pub fn extract_desktop_file(mount_point: &str) -> Result<(String, String), String> {
    let pattern = format!("{}/*.desktop", mount_point);
    let mut paths = glob(&pattern)
        .map_err(|e| format!("glob error: {}", e))?;

    let first_path = paths
        .next()
        .ok_or_else(|| "no .desktop file found in AppImage".to_string())?
        .map_err(|e| format!("read glob entry: {}", e))?;

    let data = fs::read_to_string(&first_path)
        .map_err(|e| format!("read .desktop file: {}", e))?;

    let icon_in_mount = find_icon(mount_point).unwrap_or_default();

    Ok((data, icon_in_mount))
}

pub fn find_icon(mount_point: &str) -> Option<String> {
    let patterns = [
        format!("{}/*256*icon*.png", mount_point),
        format!("{}/*128*icon*.png", mount_point),
        format!("{}/*64*icon*.png", mount_point),
        format!("{}/*48*icon*.png", mount_point),
        format!("{}/*icon*.png", mount_point),
        format!("{}/*icon*.svg", mount_point),
        format!("{}/*256*.png", mount_point),
        format!("{}/*128*.png", mount_point),
        format!("{}/*.png", mount_point),
        format!("{}/*.svg", mount_point),
    ];

    for pattern in &patterns {
        if let Ok(mut paths) = glob::glob(pattern) {
            if let Some(Ok(path)) = paths.next() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    None
}

pub fn copy_icon(src_file: &str, dst_dir: &str) -> io::Result<String> {
    if src_file.is_empty() {
        return Ok(String::new());
    }

    fs::create_dir_all(dst_dir)?;

    let file_name = Path::new(src_file)
        .file_name()
        .unwrap_or_default();
    let dst_path = Path::new(dst_dir).join(file_name);
    fs::copy(src_file, &dst_path)?;

    Ok(dst_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_is_executable_executable_file() {
        let dir = std::env::temp_dir().join("appimage_xdg_test_exec");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_exec");
        fs::write(&path, "#!/bin/sh\necho test").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(is_executable(&path.to_string_lossy()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_executable_non_executable_file() {
        let dir = std::env::temp_dir().join("appimage_xdg_test_noexec");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_noexec");
        fs::write(&path, "test").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(!is_executable(&path.to_string_lossy()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_executable_nonexistent() {
        assert!(!is_executable("/tmp/nonexistent_file_98765"));
    }

    #[test]
    fn test_copy_icon() {
        let dir = std::env::temp_dir().join("appimage_xdg_test_copy");
        let _ = fs::remove_dir_all(&dir);
        let src_dir = dir.join("src");
        let dst_dir = dir.join("dst");
        fs::create_dir_all(&src_dir).unwrap();
        let src = src_dir.join("icon.png");
        fs::write(&src, "fake-icon-data").unwrap();
        let result = copy_icon(&src.to_string_lossy(), &dst_dir.to_string_lossy()).unwrap();
        assert_eq!(result, dst_dir.join("icon.png").to_string_lossy().to_string());
        assert!(dst_dir.join("icon.png").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_copy_icon_empty_src() {
        let result = copy_icon("", "/tmp/dst").unwrap();
        assert_eq!(result, "");
    }
}
