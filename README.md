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
AppImageXdg [dirPath] [-y]

  dirPath    Directory containing .AppImage files (defaults to current directory)
  -y         Answer yes to all prompts
  --version  Show version
  -h, --help Show help
```

AppImageXdg performs two operations:

1. **Install**: Finds `.AppImage` files in `dirPath` not yet referenced by any desktop entry and creates desktop entries for them.
2. **Cleanup**: Checks all desktop entries in `$XDG_DATA_HOME/applications` and removes any whose `Exec` line points to a non-existent executable.

### Examples

```
# Clean up stale entries and install AppImages from current directory
AppImageXdg

# Same, from a specific directory, answering yes to all prompts
AppImageXdg ~/Applications -y
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
