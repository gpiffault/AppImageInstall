# AppImageXdg

Integrate AppImages into your Linux desktop with automatic desktop entries, icon extraction, and stale entry cleanup.

## Features

- Finds AppImages in the given directory, adds new ones to the desktop launcher
- Removes desktop entries whose executables no longer exist
- Extracts and copies icons from AppImages

## Install

Download a release from https://github.com/gpiffault/AppImageXdg/releases

Or build from source with the Rust toolchain:

```sh
cargo install --git https://github.com/gpiffault/AppImageXdg
```

## Usage

```
AppImageXdg [path] [-y] [--gui]

  path       Directory or .AppImage file (defaults to current directory)
  -y         Answer yes to all prompts
  --gui      Open the graphical interface
  -v, --version  Show version
  -h, --help    Show help
```

### Terminal mode (default)

Scans **path** for `.AppImage` files, creates desktop entries for unintegrated ones,
and cleans up stale entries whose executables no longer exist.

If **path** is an `.AppImage` file, it offers to move it to `~/Applications`
(or `$APPIMAGE_INSTALL_PATH`) then creates a desktop entry.

### GUI mode (`--gui`)

Opens a window listing all `.AppImage` files found in **path**. Each entry shows
an **Install** or **Remove** button depending on whether it already has a desktop entry.

- **Install**: offers to move the file to `~/Applications` first, then creates the
  desktop entry and extracts its icon.
- **Remove**: removes the desktop entry and offers to delete the `.AppImage` file.

When launched with a single `.AppImage` file, an install prompt opens automatically.

### Examples

```
# Clean up stale entries and install AppImages from current directory
AppImageXdg

# Same, from a specific directory, answering yes to all prompts
AppImageXdg ~/Applications -y

# Integrate a single AppImage (optionally moving it to ~/Applications first)
AppImageXdg ./some-app.AppImage -y

# Open the graphical interface for a directory
AppImageXdg ~/Downloads --gui

# Open the graphical interface for a single file
AppImageXdg ./some-app.AppImage --gui
```

## Build

```sh
git clone https://github.com/gpiffault/AppImageXdg.git
cd AppImageXdg
cargo build --release
```

The GUI requires GTK4 development libraries. On Debian/Ubuntu:

```sh
sudo apt install libgtk-4-dev libgraphene-1.0-dev
```

On Fedora:

```sh
sudo dnf install gtk4-devel graphene-devel
```

## Directories

AppImageXdg uses standard [XDG](https://specifications.freedesktop.org/basedir-spec/latest/) paths (respects `$XDG_DATA_HOME`, falls back to `~/.local/share`):

- `$XDG_DATA_HOME/applications/` — `.desktop` entries
- `$XDG_DATA_HOME/icons/AppImageXdg/` — extracted icons

`APPIMAGE_INSTALL_PATH` controls where single `.AppImage` files are moved to
during integration (default: `$HOME/Applications`).

## Original Project

This is originally a Go then Rust port of [appimage-desktop-integrator](https://github.com/8ByteSword/appimage-desktop-integrator), but there is not much left. It was a good starting point though.

## License

MIT
