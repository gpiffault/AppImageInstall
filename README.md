# AppImageXdg

Integrate AppImages into your Linux desktop with automatic desktop entries, icon extraction, and management commands. A Go reimplementation of the original [AppImage Desktop Integrator](https://github.com/8ByteSword/appimage-desktop-integrator) by [8ByteSword](https://github.com/8ByteSword).

## Features

- **Single binary**: Use `AppImageXdg` for all operations
- **Auto-discovery**: Finds AppImages in the current directory
- **Atomic desktop entries**: Files never get orphaned if integration fails
- **Smart Electron detection**: Auto-detects Electron apps needing `--no-sandbox`
- **Icon extraction**: Extracts and copies icons from AppImages
- **Case-insensitive matching**: All commands support case-insensitive name matching
- **Debug mode**: Run AppImages with verbose output, strace, and framework-specific debug flags
- **Interactive prompts**: Customizable app names

## Install

```sh
go install github.com/gpiffault/AppImageXdg@latest
```

Or clone and build:

```sh
git clone https://github.com/gpiffault/AppImageXdg.git
cd AppImageXdg
go build -ldflags "-X main.version=$(git describe --tags --always)" .
```

## Usage

```
AppImageXdg                    Show help
AppImageXdg status             Show status
AppImageXdg find               Find AppImages in current directory
AppImageXdg install [file]     Install AppImage(s) — prompts if no file given
AppImageXdg list               List integrated AppImages
AppImageXdg remove <name>      Remove an integrated AppImage
AppImageXdg run <name>         Run an AppImage with live output
AppImageXdg debug <name>       Run an AppImage with debug/verbose output
AppImageXdg desktop            Show .desktop files created
```

### Examples

```
# Find AppImages in current directory
AppImageXdg find

# Install a specific AppImage
AppImageXdg install ~/Downloads/Firefox.AppImage

# List integrated AppImages
AppImageXdg list

# Remove an integration
AppImageXdg remove Firefox

# Run with debug output
AppImageXdg debug Firefox
```

## XDG Directories

AppImageXdg uses standard [XDG](https://specifications.freedesktop.org/basedir-spec/latest/) paths (respects `$XDG_DATA_HOME`, falls back to `~/.local/share`):

- `$XDG_DATA_HOME/applications/` — `.desktop` entries
- `$XDG_DATA_HOME/icons/AppImageXdg/` — extracted icons

No configuration file needed.

## Original Project

This is a Go port of [appimage-desktop-integrator](https://github.com/8ByteSword/appimage-desktop-integrator), a shell script tool by [8ByteSword](https://github.com/8ByteSword) that automates the creation of desktop entries with icons for AppImage applications on Linux.

## License

MIT
