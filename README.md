# AppImageXdg

Integrate AppImages into your Linux desktop with automatic desktop entries, icon extraction, and stale entry cleanup.

## Features

- **Single binary**: Use `AppImageXdg` for all operations
- **Stale entry cleanup**: Removes desktop entries whose executables no longer exist
- **Auto-discovery**: Finds AppImages in the given directory
- **Atomic desktop entries**: Files never get orphaned if integration fails
- **Icon extraction**: Extracts and copies icons from AppImages

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
AppImageXdg [dirPath] [-y]

  dirPath    Directory containing .AppImage files (defaults to current directory)
  -y         Answer yes to all prompts
  --version  Show version
  -h, --help Show help
```

AppImageXdg performs two operations:

1. **Cleanup**: Checks all desktop entries in `$XDG_DATA_HOME/applications` and removes any whose `Exec` line points to a non-existent executable.
2. **Install**: Finds `.AppImage` files in `dirPath` not yet referenced by any desktop entry and creates desktop entries for them.

### Examples

```
# Clean up stale entries and install AppImages from current directory
AppImageXdg

# Same, from a specific directory, answering yes to all prompts
AppImageXdg ~/Downloads -y

# Show version
AppImageXdg --version
```

## XDG Directories

AppImageXdg uses standard [XDG](https://specifications.freedesktop.org/basedir-spec/latest/) paths (respects `$XDG_DATA_HOME`, falls back to `~/.local/share`):

- `$XDG_DATA_HOME/applications/` — `.desktop` entries
- `$XDG_DATA_HOME/icons/AppImageXdg/` — extracted icons

No configuration file needed.

## Original Project

This is originally a Go port of [appimage-desktop-integrator](https://github.com/8ByteSword/appimage-desktop-integrator), but there is not much left. It was a good starting point though.

## License

MIT
