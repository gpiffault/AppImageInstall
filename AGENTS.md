# AGENTS.md

## Build & Test

```sh
cargo build --release          # release build
cargo build                    # debug build
cargo test                     # all tests (GUI tests skip without display)
cargo test --test integration  # integration tests only
```

- Requires GTK4 dev libs: `libgtk-4-dev libgraphene-1.0-dev` (Debian/Ubuntu) or `gtk4-devel graphene-devel` (Fedora).
- Integration tests create fake AppImages as bash scripts and need the binary built first.
- Binary name is `AppImageInstall` (note capital letters), not `appimage-install`.

## Architecture

Single-binary Rust crate. No library target — the binary is the whole product.

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI arg parsing, orchestration, Y/N prompting |
| `src/appimage.rs` | Mount (`--appimage-mount`), extract `.desktop` + icon |
| `src/config.rs` | XDG path resolution, `APPIMAGE_INSTALL_PATH` |
| `src/desktop.rs` | `.desktop` file parse/write/remove, exec-line parsing |
| `src/gui.rs` | GTK4 + relm4 GUI (ColumnView, Install/Remove buttons) |

### Operational flow

1. **Mount**: Runs `./appimage --appimage-mount`, reads the mount-point path from stdout.
2. **Extract**: Globs `*.desktop` from the mount, finds icons by resolution preference.
3. **Rewrite desktop**: Replaces `Exec=` (rewrites `AppRun` → full path), strips `TryExec=`, updates `Icon=` to extracted path.
4. **Write entry**: Atomic write (temp file → rename) to `$XDG_DATA_HOME/applications/`.
5. **Unmount**: `fusermount -u` or `fusermount3 -u` on the mount point.
6. **Update DB**: Calls `update-desktop-database` after any entry change.

### Path resolution

- Desktop entries: `$XDG_DATA_HOME/applications/` (default `~/.local/share/applications/`)
- Icons: `$XDG_DATA_HOME/icons/AppImageInstall/`
- Install path: `$APPIMAGE_INSTALL_PATH` or `~/Applications/`

## GUI quirks

- The `relm4` GUI uses a `ColumnView` with GObject subclass `RowData` binding properties (`name`, `path`, `integrated`).
- `gui_yes_no()` pumps the GTK main loop synchronously via `MainContext::default().iteration(true)` — blocks until user clicks Yes/No.
- Self-install banner (first launch): creates a desktop entry for AppImageInstall itself when the user clicks "Add to desktop".
- GUI tests require a display (`gtk4::init()` succeeds); they silently skip otherwise.
- Integration tests also invoke the binary via `Command`, setting `HOME`, `XDG_DATA_HOME`, and `APPIMAGE_INSTALL_PATH` to temp dirs.
