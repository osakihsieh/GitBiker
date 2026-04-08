# GitBiker

Fast, minimal, open-source Git GUI client built with Rust + Tauri + Svelte.

## Why

Existing Git GUIs all have trade-offs:
- **GitKraken**: Electron, heavy memory usage, poor WSL2 experience
- **Sourcetree**: No Linux support
- **Fork / Sublime Merge**: Not open-source
- **GitButler**: Fair Source license, focused on virtual branches

GitBiker aims to be a **fast, cross-platform, open-source** Git GUI with first-class WSL2 support.

## Features (planned)

### v1.0
- Three-panel workspace: file tree, diff viewer, commit history
- Stage/unstage with checkboxes, inline diff preview
- Branch create/switch/delete
- Clone with progress indicator
- Dark/Light theme (CSS custom properties)
- Keyboard shortcuts (Ctrl+Enter commit, Ctrl+Shift+P command palette)
- WSL2 Remote: operate WSL2 repos from native Windows window

### v2.0+
- SSH Remote
- Custom commit graph renderer (Canvas 2D / WebGL)
- Merge conflict UI
- AI commit messages (BYOK Gemini)
- Terminal panel
- Plugin system

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust |
| UI | Svelte + TypeScript |
| Framework | Tauri 2.x |
| Git reads | git2-rs |
| Git writes | subprocess git (2.30+) |
| Config | tauri-plugin-store |
| Logging | tracing crate |

## Architecture

```
┌──────────────────────────────────────────────┐
│  UI (Svelte + TypeScript)                     │
│  FileTree │ DiffViewer │ CommitLog │ Settings │
├──────────────────────────────────────────────┤
│  Tauri IPC                                    │
├──────────────────────────────────────────────┤
│  Core (Rust)                                  │
│  GitOperations trait ← LocalGit │ WslGit      │
└──────────────────────────────────────────────┘
```

## Platform Support

| Platform | Status |
|----------|--------|
| Windows 11 | v1.0 |
| Ubuntu 24.04+ | v1.0 |
| Fedora 40+ | v1.0 |
| macOS | Planned (v2.0+) |

## Development

```bash
# Prerequisites: Rust, Node.js, system git 2.30+

# Install dependencies
npm install

# Dev server
npm run tauri dev

# Build
npm run tauri build
```

## License

Friendly Non‑Commercial License (with Approval for Commercial Use) v1.1

非商業使用免費，商業使用需取得作者書面同意。詳見 [LICENSE](./LICENSE)。

Copyright (c) 2026 [osakihsieh](https://github.com/osakihsieh)
