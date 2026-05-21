mod appimage;
mod config;
mod desktop;
mod gui;

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use glob::glob;

use appimage::*;
use config::*;
use desktop::*;
use gui::{AppImageEntry, run_gui};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut auto_yes = false;
    let mut cli_mode = false;
    let mut dir_path = String::new();

    for arg in &args {
        match arg.as_str() {
            "-v" | "--version" => {
                println!("AppImageInstall version {}", VERSION);
                return;
            }
            "-h" | "--help" => {
                show_help();
                return;
            }
            "-y" => auto_yes = true,
            "--cli" => cli_mode = true,
            _ => {
                if !arg.starts_with('-') && dir_path.is_empty() {
                    dir_path = arg.clone();
                }
            }
        }
    }

    if dir_path.is_empty() {
        dir_path = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
    }

    if cli_mode {
        if is_single_app_image(&dir_path) {
            cleanup_stale_entries(auto_yes);
            install_single_app_image(&dir_path, auto_yes);
            return;
        }

        cleanup_stale_entries(auto_yes);
        install_unintegrated_app_images(&dir_path, auto_yes);
        return;
    }

    let mut entries: Vec<AppImageEntry> = if is_single_app_image(&dir_path) {
        vec![AppImageEntry {
            path: dir_path.clone(),
            name: app_base_name(&dir_path),
            integrated: is_appimage_referenced(&dir_path),
        }]
    } else {
        let pattern = format!("{}/*.AppImage", dir_path);
        match glob(&pattern) {
            Ok(paths) => paths
                .filter_map(|e| e.ok())
                .map(|p| {
                    let p = p.to_string_lossy().to_string();
                    AppImageEntry {
                        integrated: is_appimage_referenced(&p),
                        name: app_base_name(&p),
                        path: p,
                    }
                })
                .collect(),
            Err(e) => {
                eprintln!("Error scanning directory: {}", e);
                Vec::new()
            }
        }
    };

    let home_apps = install_path();
    if home_apps != dir_path {
        let pattern = format!("{}/*.AppImage", home_apps);
        if let Ok(paths) = glob(&pattern) {
            for p in paths.filter_map(|e| e.ok()) {
                let p = p.to_string_lossy().to_string();
                if !entries.iter().any(|e| e.path == p) {
                    entries.push(AppImageEntry {
                        integrated: is_appimage_referenced(&p),
                        name: app_base_name(&p),
                        path: p,
                    });
                }
            }
        }
    }

    let explicit = if is_single_app_image(&dir_path) {
        Some(dir_path.clone())
    } else {
        None
    };
    let self_installed = is_self_installed();
    run_gui(entries, explicit, self_installed);
}

fn is_single_app_image(path: &str) -> bool {
    if let Ok(meta) = std::fs::metadata(path) {
        return meta.is_file() && path.ends_with(".AppImage");
    }
    false
}

fn show_help() {
    println!(
        "AppImageInstall - Manage AppImage desktop integration\n\n\
Usage:\n  \
  AppImageInstall [path] [-y] [--cli]\n\n  \
  path       Directory containing .AppImage files, or a single .AppImage file\n             \
   (defaults to current directory)\n  \
  -y         Answer yes to all prompts\n  \
  --cli      Run in command-line mode (default: GUI mode)\n  \
  -v, --version  Show version\n  \
  -h, --help    Show this help\n\n\
AppImageInstall performs two operations:\n  \
   1. Removes stale desktop entries whose executables no longer exist\n  \
   2. Creates desktop entries for AppImage files not yet integrated\n\n\
When a single .AppImage file is provided, AppImageInstall can optionally move\n\
it to ~/Applications before integrating."
    );
}

fn cleanup_stale_entries(auto_yes: bool) {
    let entries = match list_all_desktop_entries() {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in &entries {
        let exec_path = exec_line_to_path(&entry.exec);
        if exec_path.is_empty() {
            continue;
        }
        if !is_executable(&exec_path) {
            println!("Stale entry: '{}' -> {}", entry.name, exec_path);
            if auto_yes || prompt_yes_no("Remove this entry?") {
                match remove_desktop_entry(entry) {
                    Ok(_) => println!("  Removed."),
                    Err(e) => eprintln!("  Error: {}", e),
                }
            }
        }
    }
}

fn install_unintegrated_app_images(dir_path: &str, auto_yes: bool) {
    let pattern = format!("{}/*.AppImage", dir_path);
    let app_images: Vec<String> = match glob(&pattern) {
        Ok(paths) => paths.filter_map(|e| e.ok()).map(|p| p.to_string_lossy().to_string()).collect(),
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            return;
        }
    };

    if app_images.is_empty() {
        return;
    }

    for app in &app_images {
        if !is_appimage_referenced(app) {
            let base = app_base_name(app);
            println!("Unintegrated: {}", base);
            if auto_yes || prompt_yes_no("Install it?") {
                install_app_image(app);
            }
        }
    }
}

fn install_single_app_image(app_image_path: &str, auto_yes: bool) {
    if is_appimage_referenced(app_image_path) {
        println!("Already integrated: {}", app_base_name(app_image_path));
        return;
    }

    let home_applications = install_path();

    let mut dest_path = app_image_path.to_string();
    let dir = Path::new(app_image_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if dir != home_applications {
        let base = app_base_name(app_image_path);
        println!("Not in ~/Applications: {}", base);
        if auto_yes || prompt_yes_no("Move it to ~/Applications?") {
            let _ = fs::create_dir_all(&home_applications);
            let new_path = format!("{}/{}", home_applications, base);
            match fs::rename(app_image_path, &new_path) {
                Ok(_) => {
                    println!("  Moved to {}", new_path);
                    dest_path = new_path;
                }
                Err(e) => eprintln!("  Error moving file: {}", e),
            }
        }
    }

    println!("Unintegrated: {}", app_base_name(&dest_path));
    if auto_yes || prompt_yes_no("Install it?") {
        install_app_image(&dest_path);
    }
}

pub(crate) fn app_base_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn install_app_image(app_image_path: &str) {
    if ensure_dirs().is_err() {
        return;
    }
    if let Err(e) = process_app_image(app_image_path) {
        eprintln!("  Failed: {}", e);
    }
}

pub(crate) fn process_app_image(app_image_path: &str) -> Result<(), String> {
    let _ = fs::set_permissions(app_image_path, fs::Permissions::from_mode(0o755));

    println!("  Processing {}...", app_base_name(app_image_path));

    let mount = mount_appimage(app_image_path)
        .map_err(|e| format!("mount failed: {}", e))?;

    let (desktop_content, icon_in_mount) = extract_desktop_file(&mount.mount_point)
        .map_err(|e| format!("extract desktop file: {}", e))?;

    let icon_path = if !icon_in_mount.is_empty() {
        copy_icon(&icon_in_mount, &icons_dir()).unwrap_or_default()
    } else {
        String::new()
    };

    let desktop_content = modify_desktop_content(&desktop_content, app_image_path, &icon_path);

    write_desktop_entry(&desktop_content)
        .map_err(|e| format!("write desktop entry: {}", e))?;

    let name = desktop_field(&desktop_content, "Name");
    println!("  Installed: {}", name);
    Ok(())
}

fn prompt_yes_no(prompt: &str) -> bool {
    print!("{} (y/n): ", prompt);
    let _ = io::stdout().flush();
    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        let answer = line.trim().to_lowercase();
        answer == "y" || answer == "yes"
    } else {
        false
    }
}
