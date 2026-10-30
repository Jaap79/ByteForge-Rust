# ByteForge Rust 1.1.2

Native Windows text editor with colored tabs, independent tab settings, cross-tab search and session recovery. This is a separate Rust edition forked from ByteForge 1.0.0.10 at `a401721`; the original FASM repository is unchanged. Original source and binary are retained in `legacy/` and the Git history is preserved.

Repository: [Jaap79/ByteForge-Rust](https://github.com/Jaap79/ByteForge-Rust). The current executable is included under `dist/`; this preview does not yet have a separate GitHub Release or automatic Rust update feed.

Run **`dist/ByteForge-Rust.exe`**. One executable is distributed; only standard Windows DLLs are needed. No browser, WebView, .NET, sidecar icon or installed Rust runtime is required.

## Tabs and editing

Each tab has its own document, undo/redo buffer, file path, encoding, line endings, color, font family/size, zoom, word wrap, selection, scroll position and highlights. Untitled scratch tabs have unique names. Open, command-line files and drag/drop open additional tabs; reopening a path selects its existing tab. Closing an unsaved tab asks Save/Discard/Cancel.

UTF-8, UTF-8 BOM, UTF-16LE BOM and UTF-16BE BOM are supported. Encoding and CRLF/LF/CR preference are preserved on save. Invalid Unicode and binary/NUL-containing input are rejected without replacing an open document. Plain-text files never receive highlight/font metadata. MD5/SHA-256 comparison, file info, jump-to-line, clipboard editing and hex preview are retained. The hex preview shows the active buffer's encoded bytes, limited to 64 KiB, and updates as you edit, undo or replace text. Its independent 9pt monospace font keeps the columns aligned when tab fonts change. Labeled panes adapt to left/right/below layout, and hex rows use 4, 8 or 16 bytes according to the available width.

Text follows the active theme after font changes: light in dark mode, dark in light mode. Highlight backgrounds have deeper shades in dark mode so light letters remain readable; the original logical highlight colors are retained in session storage.

## Search

`Ctrl+F` opens the search/replace panel for the current tab. `Ctrl+Shift+F` searches **all open tabs**, including unsaved buffers. Enable Match case if needed. Press Enter or Find all to list matches with tab name, logical line number and preview. Double-click a result or focus it and press Enter to activate that tab and select the match; F3 cycles results. Wrapped/hidden tabs use logical file lines, not screen rows. Editing invalidates old result offsets. Replace all asks for confirmation and works in the selected scope; each tab retains its own undo history. Search results are capped at 10,000; refine searches reaching that cap before Replace all.

## Session recovery

Recovery is enabled by default. Snapshots include unsaved scratch text, modified file buffers, tab order/active tab, tab appearance, highlight ranges, selection and scroll. They are updated about every 1.5 seconds and on normal exit/Windows shutdown. With recovery enabled, exiting retains unsaved buffers instead of asking to save every tab. Closing an individual unsaved tab still asks.

Recovery files live in **`%LOCALAPPDATA%\ByteForge-Rust`**:

- `session.bfs`: current bounded, checksummed snapshot.
- `session.previous.bfs`: previous valid snapshot, used if the current one is damaged.
- `session.lock`: exclusively opened while the session is running; prevents concurrent writers.
- `recovery.off`: remembers the user's disabled-recovery preference.

Recovery text is **not encrypted**. Tools → Session recovery disables it and removes stored snapshots. Tools → Clear recovery data also removes snapshots and disables recovery, retaining open tabs and real saved files. If recovery is disabled, exit asks Save/Discard/Cancel for unsaved documents. Undo histories survive tab switching in the running process but are not restored after restart.

Recovery restores buffer contents without overwriting source files. A SHA-256 of the original disk bytes is retained. Saving a file changed/deleted on disk prompts Overwrite/Save copy/Cancel. A damaged current snapshot falls back to the previous copy; if both are damaged, existing files are retained and recovery writes pause until recovery data is explicitly cleared. Writes use flush + atomic Windows replacement. A crash may lose the last autosave interval; no application can guarantee recovery of bytes not yet written during power loss.

To use an isolated workspace: `ByteForge-Rust.exe --session-dir "C:\path\workspace"`. One instance may own each workspace. The first release limits documents to 32 MiB, sessions to 128 MiB and open tabs to 128.

## Keyboard shortcuts

All menu actions show their shortcuts. Important mappings:

| Action | Shortcut |
|---|---|
| New / Open / Save | Ctrl+N / Ctrl+O / Ctrl+S |
| Save as / Save all | Ctrl+Shift+S / Ctrl+Alt+S |
| Close tab / Exit | Ctrl+W / Alt+F4 |
| Next / Previous tab | Ctrl+Tab / Ctrl+Shift+Tab |
| Move tab left / right | Ctrl+Alt+Left / Right |
| Tab color (default/red/orange/yellow/green/blue/purple) | Ctrl+Alt+0–6 |
| Highlight red/orange/yellow/green | Ctrl+1–4 |
| Remove selection highlight | Ctrl+0 |
| Find / All tabs / Next / Selection | Ctrl+F / Ctrl+Shift+F / F3 / Ctrl+F3 |
| Replace / Jump / File info | Ctrl+H / Ctrl+G / Ctrl+I |
| Dark/light mode / Word wrap | Ctrl+D / Alt+Z |
| Zoom in / out / reset | Ctrl++ / Ctrl+- / Ctrl+Shift+0 |
| Default / Segoe UI / Georgia / Consolas | Ctrl+Shift+1–4 |
| Choose font and size | Ctrl+Shift+P |
| Hex left / right / below / close | Ctrl+Shift+L / R / B / X |
| Checksums | Ctrl+K |
| Session recovery / Clear recovery data | Ctrl+Alt+P / Ctrl+Alt+Delete |
| Manual FASM upstream check / About | Ctrl+U / F1 |

Ctrl+W now closes a tab; word wrap moved to Alt+Z. The manual upstream check explicitly reports the FASM feed; this Rust edition does not treat the FASM version as a Rust update. No automatic network requests or telemetry.

## Build and verify

Rust Windows GNU toolchain; one pinned dependency, `windows-sys = 0.59.0`. `build.rs` embeds the geometric ByteForge-family icon, DPI manifest and Windows version resource without a resource compiler. Version: 1.1.2 / PE 1.1.2.0.

```powershell
.\scripts\build.ps1 -Test -Clippy
.\tests\ui-smoke.ps1
```

The build helper reuses the existing sibling `CSVcoder-Rust\.tooling` installation if present, or accepts `-ToolingRoot`; otherwise it uses Cargo on PATH. It changes only the current process environment. Dependencies must already be cached for its offline build; on a fresh toolchain run `cargo fetch --locked` once. With a normal configured toolchain, `cargo test --locked` and `cargo build --release --locked` work directly.

Native integration tests run in an isolated session directory, apply real RichEdit formatting, exercise tabs/search/save/undo/redo, forcibly terminate the process after a snapshot, and start a second process to verify recovered live controls. Tests also launch with a system-only PATH to verify no development-toolchain DLL is needed. The original FASM application and its open documents are never controlled by these tests.

Source: `src/document.rs`, `src/session.rs`, `src/storage.rs`, `src/ui/`. Recovery format and invariants: [architecture notes](docs/ARCHITECTURE.md). Licence remains the inherited private-beta licence in `LICENSE.md`.
