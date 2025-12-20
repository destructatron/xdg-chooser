# xdg-chooser

A desktop-agnostic default application chooser for Linux, built with Rust and GTK 4.

Set default applications for common tasks (web browser, email client, media players, etc.) by reading and writing the XDG `mimeapps.list` configuration.

## Features

- Works on any desktop environment
- 14 application categories (Browser, Email, File Manager, Terminal, etc.)
- Discovers applications from `.desktop` files
- Test applications before setting as default
- Respects XDG base directory specification

## Installation

### Prerequisites

**Arch Linux:**
```bash
sudo pacman -S rust gtk4 base-devel
```

**Debian / Ubuntu:**
```bash
sudo apt install rustc cargo libgtk-4-dev build-essential
```

**Fedora:**
```bash
sudo dnf install rust cargo gtk4-devel gcc
```

**openSUSE:**
```bash
sudo zypper install rust cargo gtk4-devel gcc
```

**Gentoo:**
```bash
sudo emerge dev-lang/rust gui-libs/gtk
```

Alternatively, install Rust via [rustup](https://rustup.rs/).

### Build and Install

```bash
git clone https://github.com/destructatron/xdg-chooser.git
cd xdg-chooser
cargo build --release
```

The binary will be at `target/release/xdg-chooser`. Copy it to a location in your PATH:

```bash
sudo cp target/release/xdg-chooser /usr/local/bin/
```

## Usage

```bash
xdg-chooser
```

Select a category from the sidebar, then choose an application to set as the default.

## License

MIT
