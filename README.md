# ByteForge Rust 1.2.3

Native Windows text editor with colored tabs, independent tab settings, cross-tab search and session recovery. This is a separate Rust edition forked from ByteForge 1.0.0.10 at `a401721`; the original FASM repository is unchanged. Original source and binary are retained in `legacy/` and the Git history is preserved.

Public repository: [Jaap79/ByteForge-Rust](https://github.com/Jaap79/ByteForge-Rust). The current executable is included under `dist/`. Help offers an explicit Rust-version check against `main`; there is no separate GitHub Release feed or automatic installation.

Run **`dist/ByteForge-Rust.exe`** (1.2.3). Close any old instance once when upgrading so the executable can be replaced. One self-contained executable; only standard Windows DLLs are needed. No browser, WebView, .NET, PowerShell installation, sidecar grammar files or installed Rust runtime is required.

## Tabs and editing

Each tab has its own document, undo/redo buffer, file path, encoding, line endings, color, font family/size, zoom, word wrap, selection, scroll position and highlights. Untitled scratch tabs have unique names. Open, command-line files and drag/drop open additional tabs; reopening a path selects its existing tab. Closing an unsaved tab asks Save/Discard/Cancel.

Files opened through Explorer/Open With are handed off to the existing ByteForge window as tabs, including multiple files and repeated separate launches. Opening an already-open path selects its existing tab; it does not reload or discard its unsaved contents. Starting ByteForge again without files brings the existing window forward. `--session-dir` workspaces remain independent and retain exclusive recovery ownership. A busy modal prompt queues incoming files until it is dismissed. A receiver running an older version or at an incompatible elevation level produces a clear failure instead of silently dropping files. Run the editor at the same normal permission level as Explorer; no association or registry changes are made by the application.

The **+** directly after the last tab opens a blank tab; each tab has its own **×** close target. Drag a tab to change the order; the orange insertion marker previews its destination. Moving retains the editor, contents, selection, undo/redo, font, color and context. Ctrl+Alt+Left/Right also reorders tabs. When tabs overflow, native scroll arrows remain available and the plus stays reachable. Right-click any tab for **Tabkleur** and **Tabblad sluiten**, without switching away from the current document. The palette has eleven choices, including Cyan, Pink, Teal and Gray. Editor right-click actions group highlight colors and removal under **Tekstmarkering**.

UTF-8, UTF-8 BOM, UTF-16LE BOM and UTF-16BE BOM are supported. Encoding and CRLF/LF/CR preference are preserved on save. Invalid Unicode and binary/NUL-containing input are rejected without replacing an open document. Plain-text files never receive highlight/font metadata. MD5/SHA-256 comparison, file info, jump-to-line, clipboard editing and hex preview are retained. The hex preview shows the active buffer's encoded bytes, limited to 64 KiB, and updates as you edit, undo or replace text. Its independent 9pt monospace font keeps the columns aligned when tab fonts change. Labeled panes adapt to left/right/below layout, and hex rows use 4, 8 or 16 bytes according to the available width.

Text follows the active theme after font changes: light in dark mode, dark in light mode. Highlight backgrounds have deeper shades in dark mode so light letters remain readable; the original logical highlight colors are retained in session storage.

The tab strip has 4 logical pixels of additional space above and below it. The status bar is 30 logical pixels high, with vertically centered text. These measurements scale with DPI and use the existing themed surfaces rather than system-white separators.

**Edit** and the editor's right-click menu offer **Paste with formatting** (Ctrl+V) and **Paste without formatting** (Ctrl+Shift+V). The first keeps available rich-text formatting. The second inserts only Unicode text, using the current tab's font/size and theme, without source or inherited character styles/highlights. Each paste is one undo/redo action. These remain plain-text documents: rich formatting is for the live editor, not RTF/HTML file saving; recovery retains the existing per-tab settings and manual highlight ranges, not arbitrary rich-text styles.

## Syntax context — per tab

Choose **Tools → Context (this tab)**: **Plain text**, **PowerShell**, **JSON**, **YAML**, **XML / HTML** or **INI / config**. New tabs start as Plain text; context is chosen explicitly, stays independent per tab and is restored with the session. Ctrl+Alt+Shift+0–5 selects the corresponding context.

PowerShell distinguishes keywords, function/command names, parameters, variables, member access, types, numbers, comments, quoted strings and here-strings; multiline state is retained. JSON distinguishes property names, strings, literals, numbers and delimiters. The other contexts cover configuration keys/values, sections, tags, attributes, comments and YAML block scalars. These are tolerant lexical display modes, not full semantic parsers, validators, code folding or language servers; XML/HTML does not include embedded JavaScript/CSS grammars. No scripts are run and nothing is sent to a service.

Separate syntax palettes provide at least 4.5:1 contrast on the dark/light editor backgrounds. Manual text-highlight backgrounds take priority, using the theme's default foreground so marks stay readable. Returning to Plain text removes syntax foregrounds, not manual marks or fonts. Syntax changes do not change file contents, dirty state, caret/selection, scrolling or undo/redo history.

Typing is debounced by 120 ms. A line-state cache reuses unchanged syntax until multiline state converges. Native foreground formatting clips each visible screen row horizontally, so long offscreen JSON/NDJSON tails are not formatted. Selection-free TOM ranges apply changes in batches of at most 256 operations or roughly 8 ms, yielding through the message loop between batches. Scroll refreshes reuse cached tokens. No extra dependency is added.

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
| Paste with / without formatting | Ctrl+V / Ctrl+Shift+V |
| Next / Previous tab | Ctrl+Tab / Ctrl+Shift+Tab |
| Move tab left / right | Ctrl+Alt+Left / Right |
| Tab color (default/red/orange/yellow/green/blue/purple) | Ctrl+Alt+0–6 |
| Tab color (cyan/pink/teal/gray) | Ctrl+Alt+7 / 8 / 9 / G |
| Context: Plain / PowerShell / JSON / YAML / XML / INI | Ctrl+Alt+Shift+0–5 |
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
| Check Rust updates / GitHub repository / About | Ctrl+U / Ctrl+Shift+U / F1 |

Ctrl+W closes a tab; word wrap uses Alt+Z. **Help → Check for updates** reads the Rust package version from the public repository's `main/Cargo.toml` over HTTPS on a background thread. It compares numeric version components, reports current/newer/local-ahead results, and distinguishes network/HTTP/invalid-response errors from success. It does not check the FASM edition, request credentials, install updates or send document contents. **Help → GitHub repository**, About and update results expose a clickable, keyboard-focusable repository link. No automatic network requests or telemetry.

## Build and verify

Rust Windows GNU toolchain; one pinned dependency, `windows-sys = 0.59.0`. `build.rs` embeds the geometric ByteForge-family icon, DPI manifest and Windows version resource without a resource compiler. Version: 1.2.3 / PE 1.2.3.0.

```powershell
.\scripts\build.ps1 -Test -Clippy
.\tests\ui-smoke.ps1
.\tests\explorer-open.ps1
.\tests\syntax-performance.ps1
.\tests\update-check.ps1 # Explicit live GitHub check; other tests are offline.
```

For a side-by-side build while the default executable is running:

```powershell
.\scripts\build.ps1 -Test -Clippy -Output dist\ByteForge-Rust-1.2.3.exe
.\tests\ui-smoke.ps1 -Executable .\dist\ByteForge-Rust-1.2.3.exe
```

The build helper reuses the existing sibling `CSVcoder-Rust\.tooling` installation if present, or accepts `-ToolingRoot`; otherwise it uses Cargo on PATH. It changes only the current process environment. Dependencies must already be cached for its offline build; on a fresh toolchain run `cargo fetch --locked` once. With a normal configured toolchain, `cargo test --locked` and `cargo build --release --locked` work directly.

Native integration tests run in an isolated session directory, apply real RichEdit formatting, exercise tabs/search/save/undo/redo, forcibly terminate the process after a snapshot, and start a second process to verify recovered live controls. Tests also launch with a system-only PATH to verify no development-toolchain DLL is needed. The original FASM application and its open documents are never controlled by these tests.

Source: `src/document.rs`, `src/session.rs`, `src/storage.rs`, `src/ui/`. Recovery format and invariants: [architecture notes](docs/ARCHITECTURE.md). Licence remains the inherited private-beta licence in `LICENSE.md`.
