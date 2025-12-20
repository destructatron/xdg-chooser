# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build              # Development build
cargo build --release    # Release build (with LTO, stripped)
cargo run                # Run the application
cargo check              # Fast type checking
cargo test               # Run unit tests
```

## Architecture Overview

xdg-chooser is a desktop-agnostic default application chooser for Linux, built with Rust and GTK 4. It allows users to set default applications for common tasks (browser, email, media players, etc.) by reading/writing the XDG `mimeapps.list` configuration.

### Data Flow

1. **Application Discovery** (`desktop/discovery.rs`): Scans `/usr/share/applications` and `~/.local/share/applications` for `.desktop` files, parsing them into `AppEntry` structs and indexing by MIME type and category.

2. **Configuration** (`config/mimeapps.rs`): Merges configs from all XDG locations in priority order (desktop-specific user config → user config → system configs → data dirs), writes changes to `~/.config/mimeapps.list`. Validates MIME types and app IDs before saving.

3. **Category Mapping** (`desktop/categories.rs`): Maps high-level categories (WebBrowser, EmailClient, etc.) to their associated MIME types (e.g., `x-scheme-handler/http`) and desktop categories (e.g., `TerminalEmulator`).

### UI Structure

- **MainWindow** (`window.rs`): Orchestrates the UI with a horizontal `Paned` layout
- **CategorySidebar** (`ui/sidebar.rs`): Navigation list of 14 application categories
- **CategoryPage** (`ui/category_page.rs`): Shows current default + available apps for selected category
- **AppRow** (`ui/app_row.rs`): Individual application entry with "Set as Default" and "Test" buttons

### Key Patterns

- Uses `Rc<RefCell<>>` for shared mutable state (GTK is single-threaded)
- Category pages rebuild themselves via callback when defaults change
- `.desktop` file parsing is done manually (not using external parser crate)
- Icon loading falls back to `freedesktop-icons` crate when GTK theme doesn't have the icon
- Exec command parsing uses `shell-words` crate for proper quoted argument handling
- Process launching uses `process_group(0)` for safe detachment from parent
- Locale matching supports `@modifier` suffix (e.g., `sr@latin`)

### Special Cases

- **Terminal/Calculator/Calendar**: These categories have no MIME types; apps are detected via `Categories=` field in `.desktop` files
- **Desktop-specific configs**: Checks `XDG_CURRENT_DESKTOP` for files like `gnome-mimeapps.list`
