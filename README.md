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
AppImageXdg [path] [-y]

  path       Directory or .AppImage file (defaults to current directory)
  -y         Answer yes to all prompts
  --version  Show version
  -h, --help Show help
```

If **path** is a directory, AppImageXdg scans for `.AppImage` files and creates
desktop entries for any not yet integrated. It also removes stale desktop entries
whose executables no longer exist.

If **path** is an `.AppImage` file, it offers to move the file to `~/Applications`
(or `$APPIMAGE_INSTALL_PATH`) if not already there, then creates a desktop entry
for it.

### Examples

```
# Clean up stale entries and install AppImages from current directory
AppImageXdg

# Same, from a specific directory, answering yes to all prompts
AppImageXdg ~/Applications -y

# Integrate a single AppImage (optionally moving it to ~/Applications first)
AppImageXdg ./some-app.AppImage -y
```

## Build

```sh
git clone https://github.com/gpiffault/AppImageXdg.git
cd AppImageXdg
cargo build --release
```

## XDG Directories

AppImageXdg uses standard [XDG](https://specifications.freedesktop.org/basedir-spec/latest/) paths (respects `$XDG_DATA_HOME`, falls back to `~/.local/share`):

- `$XDG_DATA_HOME/applications/` — `.desktop` entries
- `$XDG_DATA_HOME/icons/AppImageXdg/` — extracted icons

## Original Project

This is originally a Go then Rust port of [appimage-desktop-integrator](https://github.com/8ByteSword/appimage-desktop-integrator), but there is not much left. It was a good starting point though.

## License

MIT
