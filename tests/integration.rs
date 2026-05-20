use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TestEnv {
    dir: PathBuf,
    home: PathBuf,
    data_home: PathBuf,
    install_path: PathBuf,
    appimages_dir: PathBuf,
}

impl TestEnv {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("appimage_install_int_{}", name));
        let _ = fs::remove_dir_all(&dir);

        let home = dir.join("home");
        let data_home = dir.join("data");
        let install_path = dir.join("Applications");
        let appimages_dir = dir.join("appimages");

        let apps_dir = data_home.join("applications");
        let icons_dir = data_home.join("icons").join("AppImageInstall");

        for d in &[&home, &apps_dir, &icons_dir, &install_path, &appimages_dir] {
            fs::create_dir_all(d).unwrap();
        }

        for d in &[&dir, &home, &data_home, &install_path, &appimages_dir] {
            let mut perms = fs::metadata(d).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(d, perms).unwrap();
        }

        TestEnv { dir, home, data_home, install_path, appimages_dir }
    }

    fn create_mock_appimage(&self, name: &str, display_name: &str) -> PathBuf {
        let path = self.appimages_dir.join(name);
        let mount_dir = self.dir.join(format!("mount_{}", display_name));

        let script = format!(
            "#!/bin/bash\n\
             if [ \"$1\" = \"--appimage-mount\" ]; then\n\
                 mkdir -p '{}'\n\
                 cat > '{}/app.desktop' <<'EOF'\n\
[Desktop Entry]\n\
Name={}\n\
Exec=AppRun\n\
Type=Application\n\
Categories=Utility;\n\
EOF\n\
                 touch '{}/icon_256.png'\n\
                 echo '{}'\n\
                 exec sleep 300\n\
             fi\n",
            mount_dir.display(),
            mount_dir.display(),
            display_name,
            mount_dir.display(),
            mount_dir.display(),
        );

        fs::write(&path, script).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    fn binary_path() -> PathBuf {
        if let Ok(p) = env::var("CARGO_BIN_EXE_AppImageInstall") {
            let pb = PathBuf::from(&p);
            if pb.exists() { return pb; }
        }
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if cfg!(debug_assertions) {
            manifest.join("target/debug/AppImageInstall")
        } else {
            manifest.join("target/release/AppImageInstall")
        }
    }

    fn run(&self, args: &[&str], cur_dir: &Path) -> std::process::Output {
        Command::new(Self::binary_path())
            .args(args)
            .env("HOME", &self.home)
            .env("XDG_DATA_HOME", &self.data_home)
            .env("APPIMAGE_INSTALL_PATH", &self.install_path)
            .current_dir(cur_dir)
            .output()
            .expect("failed to run binary")
    }

    fn desktop_files(&self) -> Vec<PathBuf> {
        let apps_dir = self.data_home.join("applications");
        if !apps_dir.exists() {
            return vec![];
        }
        fs::read_dir(&apps_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("desktop"))
            .collect()
    }

    fn icon_files(&self) -> Vec<PathBuf> {
        let icons_dir = self.data_home.join("icons").join("AppImageInstall");
        if !icons_dir.exists() {
            return vec![];
        }
        fs::read_dir(&icons_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect()
    }

    fn write_fake_desktop_entry(&self, name: &str, exec: &str) -> PathBuf {
        let path = self.data_home
            .join("applications")
            .join(format!("{}.desktop", name));
        let content = format!(
            "[Desktop Entry]\nName={}\nExec={}\nType=Application\n",
            name, exec
        );
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = Command::new("pkill")
            .args(["-f", "mock_mount_"])
            .output();
        std::thread::sleep(std::time::Duration::from_millis(300));
        let _ = fs::remove_dir_all(&self.dir);
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn test_install_single_appimage() {
    let env = TestEnv::new("install_single");
    let appimage = env.create_mock_appimage("TestApp.AppImage", "Test App");

    // Copy to install_path so it won't prompt about moving
    let dest = env.install_path.join("TestApp.AppImage");
    fs::copy(&appimage, &dest).unwrap();

    let output = env.run(&[dest.to_str().unwrap(), "-y"], &env.install_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Installed"),
        "should print Installed. stdout: {}\nstderr: {}", stdout, stderr
    );

    // Should have created a .desktop file
    let desktop = env.desktop_files();
    assert!(!desktop.is_empty(), "should have at least one .desktop file");

    // Should have copied an icon
    let icons = env.icon_files();
    assert!(!icons.is_empty(), "should have at least one icon file");
}

#[test]
fn test_already_integrated_is_skipped() {
    let env = TestEnv::new("already_integrated");
    let appimage = env.create_mock_appimage("TestApp.AppImage", "Test App");

    let dest = env.install_path.join("TestApp.AppImage");
    fs::copy(&appimage, &dest).unwrap();

    // First run: install
    let output = env.run(&[dest.to_str().unwrap(), "-y"], &env.install_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Installed"), "first run should install. got: {}", stdout);
    assert_eq!(env.desktop_files().len(), 1, "should have one desktop entry");

    // Second run: should detect as already integrated
    let output2 = env.run(&[dest.to_str().unwrap(), "-y"], &env.install_path);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("Already integrated"),
        "should say Already integrated. got: {}", stdout2
    );
    assert!(
        !stdout2.contains("Unintegrated"),
        "should NOT say Unintegrated. got: {}", stdout2
    );
}

#[test]
fn test_directory_install_skips_integrated() {
    let env = TestEnv::new("dir_skip");
    env.create_mock_appimage("AppA.AppImage", "App A");
    env.create_mock_appimage("AppB.AppImage", "App B");

    // Copy both to install_path
    fs::copy(
        env.appimages_dir.join("AppA.AppImage"),
        env.install_path.join("AppA.AppImage"),
    ).unwrap();
    fs::copy(
        env.appimages_dir.join("AppB.AppImage"),
        env.install_path.join("AppB.AppImage"),
    ).unwrap();

    // First run: install all from directory
    let output = env.run(&[env.install_path.to_str().unwrap(), "-y"], &env.install_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Installed"), "first run should install. got: {}", stdout);
    assert_eq!(env.desktop_files().len(), 2, "should have two desktop entries");

    // Second run: should skip both as already integrated
    let output2 = env.run(&[env.install_path.to_str().unwrap(), "-y"], &env.install_path);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        !stdout2.contains("Unintegrated"),
        "should NOT show any Unintegrated. got: {}", stdout2
    );
}

#[test]
fn test_cleanup_stale_entry() {
    let env = TestEnv::new("cleanup_stale");

    // Write a fake desktop entry pointing to a non-existent executable
    let stale_path = env.write_fake_desktop_entry(
        "stale_app",
        "/tmp/nonexistent_fake_app_123456",
    );
    assert!(stale_path.exists(), "fake desktop entry should exist");
    assert_eq!(env.desktop_files().len(), 1);

    // Run on empty directory: cleanup runs, no AppImages to install
    let empty_dir = env.dir.join("empty");
    fs::create_dir_all(&empty_dir).unwrap();
    let output = env.run(&[empty_dir.to_str().unwrap(), "-y"], &empty_dir);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Stale entry"),
        "should detect stale entry. got: {}", stdout
    );

    // The stale entry should be removed (since -y answers yes)
    assert!(
        !stale_path.exists(),
        "stale desktop entry should be removed.\nstdout: {}\ndesktop files: {:?}",
        stdout,
        env.desktop_files(),
    );
    assert_eq!(env.desktop_files().len(), 0, "no desktop files should remain");
}

#[test]
fn test_cleanup_keeps_valid_entry() {
    let env = TestEnv::new("cleanup_keep");
    let appimage = env.create_mock_appimage("MyApp.AppImage", "My App");

    let dest = env.install_path.join("MyApp.AppImage");
    fs::copy(&appimage, &dest).unwrap();

    // Install: creates a valid desktop entry
    let output = env.run(&[dest.to_str().unwrap(), "-y"], &env.install_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Installed"), "should install. got: {}", stdout);
    assert_eq!(env.desktop_files().len(), 1, "should have one desktop entry");

    // Run again: cleanup should NOT remove the valid entry
    let output2 = env.run(&[dest.to_str().unwrap(), "-y"], &env.install_path);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        !stdout2.contains("Stale entry"),
        "should NOT find stale entries. got: {}", stdout2
    );
    assert_eq!(env.desktop_files().len(), 1, "valid desktop entry should remain");
}

#[test]
fn test_install_path_env_var() {
    let env = TestEnv::new("install_path_env");
    let appimage = env.create_mock_appimage("CustomApp.AppImage", "Custom App");

    // Copy to a location outside the install path
    let outside = env.dir.join("outside");
    fs::create_dir_all(&outside).unwrap();
    let src = outside.join("CustomApp.AppImage");
    fs::copy(&appimage, &src).unwrap();

    // Run single-file mode: should detect "Not in ~/Applications" and move (via -y)
    let output = env.run(&[src.to_str().unwrap(), "-y"], &outside);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Moved to"),
        "should move to install path. got: {}", stdout
    );

    // Verify it was moved to install_path
    let moved = env.install_path.join("CustomApp.AppImage");
    assert!(moved.exists(), "AppImage should have been moved to install path");
}
