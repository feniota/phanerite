# Phanerite

This is the GUI part of Phanerite Launcher.

The application uses `org.feniota.phanerite` as its stable system identifier
and `Phanerite Launcher` as its native window title. The gallery uses the same
identity and window configuration.

## Linux (Wayland and X11)

Build from the workspace root, then install the binary, desktop entry and icon
theme assets for the current user:

```sh
cargo build -p phanerite --release
make -C phanerite-app install-linux PREFIX="$HOME/.local"
```

Keep `$HOME/.local/bin` on `PATH`. If `XDG_DATA_HOME` is customized, also pass
`DATADIR="$XDG_DATA_HOME"` to `make`. Packagers can set `PREFIX=/usr` and
`DESTDIR` for a staged installation; `BINARY` selects a different build output.

Wayland sets the toplevel `app_id` to the identifier. Compositors resolve its
name and icon from the installed `org.feniota.phanerite.desktop` file and the
matching hicolor icon. Running an uninstalled binary alone cannot supply a
Wayland launcher icon. X11 sets both `WM_CLASS` fields to the same identifier
and also supplies a bundled 256 px RGBA icon through `_NET_WM_ICON`.

## Windows

`cargo build -p phanerite --release` embeds the multi-resolution ICO as resource
ID **1**, which GPUI loads for its native windows, and adds executable name and
version metadata. A Windows resource compiler (`rc.exe`, `llvm-rc` or MinGW
`windres`) is required, including when cross-compiling. Startup registers
`org.feniota.phanerite` as the process AppUserModelID for taskbar grouping.
GPUI owns the internal Win32 window class; it is not the application's shell
identifier. Installer shortcuts should use the same AppUserModelID.

## macOS

On macOS, use `cargo-bundle` to produce the application bundle:

```sh
cargo install cargo-bundle --locked
cd phanerite-app
cargo bundle --release --format osx
```

The Cargo bundle metadata supplies the `Phanerite` application name,
`org.feniota.phanerite` bundle identifier and multi-resolution ICNS icon for
Finder, the Dock and the application switcher. Launch the resulting
`Phanerite.app`; `cargo run` does not create or register an application bundle.
The native window title is also set when running the binary directly.

## Updating the application icon

Edit `assets/phanerite-logo.svg`, then follow the
[icon generation instructions](assets/app-icons/README.md). PNG, ICO and ICNS
files are generated from the same SVG and retain its CC BY-SA 4.0 license.
If distributing a renamed fork, update `src/app_metadata.rs`, the Linux desktop
entry and Makefile, and the Cargo bundle metadata together.
