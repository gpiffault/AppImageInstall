# AppImageInstall

Integrate AppImages into your Linux desktop with automatic desktop entries, icon extraction, and stale entry cleanup.

## Features

- Finds AppImages in the given directory, adds new ones to the desktop launcher
- Removes desktop entries whose executables no longer exist
- Extracts and copies icons from AppImages

## Install

Download a release from https://github.com/gpiffault/AppImageInstall/releases

Or build from source with the Rust toolchain:

```sh
cargo install --git https://github.com/gpiffault/AppImageInstall
```

## Usage

```
AppImageInstall [path] [-y] [--cli]

  path       Directory or .AppImage file (defaults to current directory)
  -y         Answer yes to all prompts
  --cli      Run in command-line mode (default: GUI mode)
  -v, --version  Show version
  -h, --help    Show help
```

### GUI mode (default)

Opens a window listing all `.AppImage` files found in **path** and `~/Applications`.
Each entry shows an **Install** or **Remove** button depending on whether it already
has a desktop entry.

- **Install**: offers to move the file to `~/Applications` first, then creates the
  desktop entry and extracts its icon.
- **Remove**: removes the desktop entry and offers to delete the `.AppImage` file.

When launched with a single `.AppImage` file, an install prompt opens automatically.

### CLI mode (`--cli`)

Scans **path** for `.AppImage` files, creates desktop entries for unintegrated ones,
and cleans up stale entries whose executables no longer exist.

If **path** is an `.AppImage` file, it offers to move it to `~/Applications`
(or `$APPIMAGE_INSTALL_PATH`) then creates a desktop entry.

### Examples

```
# Open the graphical interface (default)
AppImageInstall

# Open the graphical interface for a directory
AppImageInstall ~/Downloads

# Open the graphical interface for a single file
AppImageInstall ./some-app.AppImage

# Clean up stale entries and install AppImages from current directory (CLI mode)
AppImageInstall --cli

# Same, from a specific directory, answering yes to all prompts
AppImageInstall ~/Applications -y --cli

# Integrate a single AppImage (optionally moving it to ~/Applications first)
AppImageInstall ./some-app.AppImage -y --cli
```

## Build

```sh
git clone https://github.com/gpiffault/AppImageInstall.git
cd AppImageInstall
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

AppImageInstall uses standard [XDG](https://specifications.freedesktop.org/basedir-spec/latest/) paths (respects `$XDG_DATA_HOME`, falls back to `~/.local/share`):

- `$XDG_DATA_HOME/applications/` — `.desktop` entries
- `$XDG_DATA_HOME/icons/AppImageInstall/` — extracted icons

`APPIMAGE_INSTALL_PATH` controls where single `.AppImage` files are moved to
during integration (default: `$HOME/Applications`).

## Original Project

This is originally a Go then Rust port of [appimage-desktop-integrator](https://github.com/8ByteSword/appimage-desktop-integrator), but there is not much left. It was a good starting point though.

## License

MIT
