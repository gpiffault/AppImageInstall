use std::io;

pub fn xdg_data_home() -> String {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        if !dir.is_empty() {
            return dir;
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{}/.local/share", home)
}

pub fn icons_dir() -> String {
    format!("{}/icons/AppImageXdg", xdg_data_home())
}

pub fn applications_dir() -> String {
    format!("{}/applications", xdg_data_home())
}

pub fn install_path() -> String {
    if let Ok(dir) = std::env::var("APPIMAGE_INSTALL_PATH") {
        if !dir.is_empty() {
            return dir;
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{}/Applications", home)
}

pub fn ensure_dirs() -> io::Result<()> {
    std::fs::create_dir_all(applications_dir())?;
    std::fs::create_dir_all(icons_dir())?;
    Ok(())
}
