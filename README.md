# AppImageXdg

Integrate AppImages into your Linux desktop with automatic desktop entries, icon extraction, and management commands. A Go reimplementation of the original [AppImage Desktop Integrator](https://github.com/8ByteSword/appimage-desktop-integrator) by [8ByteSword](https://github.com/8ByteSword).

## Features

- **Single binary**: Use `axdg` for all operations
- **Auto-discovery**: Finds AppImages in common locations (Downloads, Desktop, etc.)
- **Atomic desktop entries**: Files never get orphaned if integration fails
- **Smart Electron detection**: Auto-detects Electron apps needing `--no-sandbox`
- **Icon extraction**: Extracts and copies icons from AppImages
- **Case-insensitive matching**: All commands support case-insensitive name matching
- **Debug mode**: Run AppImages with verbose output, strace, and framework-specific debug flags
- **Interactive prompts**: Customizable app names, storage location selection

## Install

```sh
go install github.com/8ByteSword/AppImageXdg@latest
```

Or clone and build:

```sh
git clone https://github.com/8ByteSword/AppImageXdg.git
cd AppImageXdg
go build -o axdg .
```

## Usage

```
axdg                    Show help
axdg status             Show configuration and status
axdg find               Find AppImages on your system
axdg install [file]     Install AppImage(s) — prompts if no file given
axdg list               List integrated AppImages
axdg remove <name>      Remove an integrated AppImage
axdg run <name>         Run an AppImage with live output
axdg debug <name>       Run an AppImage with debug/verbose output
axdg desktop            Show .desktop files created
```

### Examples

```
# Find AppImages in common locations
axdg find

# Install a specific AppImage
axdg install ~/Downloads/Firefox.AppImage

# List integrated AppImages
axdg list

# Remove an integration
axdg remove Firefox

# Run with debug output
axdg debug Firefox
```

## Configuration

Configuration is stored at `~/.config/AppImageXdg/config.ini`:

```ini
icons_dir=~/.local/share/icons/AppImageXdg
update_dir=~/.local/share/applications
appimages_dirs=("$HOME/Applications" "$HOME/AppImages")
```

- **icons_dir** — Where extracted AppImage icons are stored
- **update_dir** — Where `.desktop` files are created
- **appimages_dirs** — Directories monitored for AppImages

## Original Project

This is a Go port of [appimage-desktop-integrator](https://github.com/8ByteSword/appimage-desktop-integrator), a shell script tool by [8ByteSword](https://github.com/8ByteSword) that automates the creation of desktop entries with icons for AppImage applications on Linux.

## License

MIT
