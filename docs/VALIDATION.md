# Validation — ByteForge Rust 1.2.1

## Current 1.2.1 build — 30 October 2026

- Path: `dist/ByteForge-Rust.exe`; size: 553,984 bytes (541 KiB); Windows FileVersion/ProductVersion: `1.2.1.0`.
- SHA-256: `2FBBF71ECAEB1EF605669DA0685515AD23AAEF3E52F935FC4116FA2BBFD4172F`.
- Format check, **17 unit/model/storage tests**, Clippy with warnings denied and optimized release build passed.
- **116 native UI integration checks** and **10 process-stop/restart checks** passed, including file handoff accepted while the modal owner is disabled and subsequent existing-tab selection without changing unsaved buffers. Evidence: `output/qa-20261030-153007-312/`.
- `tests/explorer-open.ps1` passed real multi-process launches: multiple files in one request, later separate launches, spaces/Unicode names, relative caller paths, duplicate-path selection, empty/bare launch, startup timing, case/relative session-directory aliases and workspace isolation. Each secondary process exited promptly instead of blocking on the old warning. Restart verification passed **five primary-workspace checks** and **three isolation-workspace checks**, reading back actual live editor contents and original unsaved buffers/settings. Evidence: `output/explorer-20261030-153031-172/checks.txt`, `verify-primary/result.txt`, `verify-isolation/result.txt`.
- The same complete multi-process test also passed on the replaced canonical executable: `output/explorer-20261030-153313-843/checks.txt`, `verify-primary/result.txt`, `verify-isolation/result.txt`.
- Protocol tests cover every truncation of a valid message, trailing/oversized/malformed input, unknown flags, relative-path rejection and lossless Windows UTF-16 paths. Requests are bounded and are not executed as commands.
- Computer Use observed an isolated visible window with one file; launching the canonical executable with two more file arguments added both tabs to that same window and displayed the final file correctly, in dark and light modes. This verifies the same executable launch path used by Explorer without changing associations. Actual Explorer Open With/default-association UI was not changed or used as proof of registry configuration.
- The pre-existing Rust processes were left untouched. After the user closed them, the canonical executable was replaced with the verified build at the same path; no registry, user documents or original FASM installation was changed. The design-language palette/native chrome is retained with no new UI surface or runtime dependency. Source, executable and regression tests are distributed together through the repository.

## Previous 1.2.0 build — 30 October 2026

- Path: `dist/ByteForge-Rust.exe`; size: 536,064 bytes (523.5 KiB); Windows FileVersion/ProductVersion: `1.2.0.0`.
- SHA-256: `29A17D46A4F2FAF8ED7A66C25FFC72246E75EBEA887A116B403070788B99C2BA`.
- Format check, **15 unit/model/storage tests**, Clippy on all targets with warnings denied and optimized release build passed.
- **114 native UI integration checks** passed, including the plus immediately after the final tab, overflow reachability, native drag threshold/target/release messages, reordered editor visibility and live formatting, independent contexts, actual PowerShell/JSON foregrounds, theme/font switching, Plain text reset, manual highlight priority and undo/redo preservation.
- Scroll checks confirm newly visible lines are colored without relexing or moving the viewport. Existing save/search/hex/encoding/recovery tests remain included.
- **10 process-stop/restart checks** passed, including independent JSON, PowerShell and Plain text contexts restored in live tabs.
- Local evidence: `output/qa-20261030-144412-845/checks.txt`, `result.txt`, `restart/result.txt`.
- Lexer tests cover multiline PowerShell comments/here-strings, quoting, token roles, Unicode UTF-16 offsets, JSON keys/values, YAML block scalars, XML/INI categories, and incremental-versus-fresh equivalence across 120 edits for all six contexts. A legacy BFRSESS1 snapshot imports unchanged with Plain text contexts.
- Automated contrast tests verify all 12 token roles at **at least 4.5:1** on both editor backgrounds. Manual highlighter backgrounds use the default theme foreground, not potentially conflicting token colors.
- Release-mode cache measurement on this development host: **100,000 PowerShell lines / 3,688,890 bytes**, first analysis **73.76 ms**, subsequent one-line edit **8.24 ms**, only **two lines lexed**. These are lexer/cache timings, not end-to-end UI latency guarantees; initial work remains linear in file size.
- Computer Use verified actual mouse dragging in both directions, preservation of the selected editor and text after moving, the adjacent plus following tab creation, PowerShell/JSON display, live typed token colors, per-tab context persistence, Tools → Context popup and Plain text selection, and readable dark/light rendering. All work used isolated fixture files under `output/manual-120`; user documents and the original FASM process were not controlled.
- The design-language skill guided the two contrast-safe syntax palettes and native chrome; no dependency, sidecar grammar or interpreter was introduced. Syntax coloring is lexical, not full semantic analysis, linting, code folding or execution. Separate 125%/150% monitor configurations were not available for manual screenshot QA.
- The canonical executable was replaced only after the isolated test instance was closed. Source, executable and validation notes are distributed together through the repository; no separate GitHub Release or automatic Rust update channel is configured.

## Previous 1.1.3 build — 30 October 2026

- Path: `dist/ByteForge-Rust-1.1.3.exe`; size: 482,816 bytes; Windows FileVersion/ProductVersion: `1.1.3.0`.
- SHA-256: `DC78104A4CCE497ABE3FA2C6C08CB473B11DABB364D1313D53E282805399542A`.
- Format check, seven model/storage tests, Clippy with warnings denied and optimized release build passed.
- **93 native UI integration checks** passed, including inactive-tab color metadata/persistence, eleven-color checked context menu, compact text-highlight submenu, plus-button command, individual close hit areas, inactive dirty-tab Cancel/Discard with active-tab preservation, non-overlapping plus/overflow geometry and menu text measurement.
- **Nine process-stop/restart checks** passed. Existing hex, theme, font, search, undo/redo, save and recovery coverage remains included.
- Local evidence: `output/qa-20261030-142338-601/checks.txt`, `result.txt`, `restart/result.txt`.
- An additional visible run passed **94 checks**, including reading the actual painted stripe pixel immediately after recoloring an inactive tab. Evidence: `output/qa-visible-113/checks.txt`, `result.txt`. This pixel check is skipped in the hidden harness because hidden windows have no readable screen pixels.
- Computer Use exercised actual plus-button and close-cross mouse clicks, tab right-click color selection without switching active document, editor **Tekstmarkering** submenu and dark/light rendering. All six top-level menu labels render completely. Existing design-language palette and thick color stripes are retained.
- The previous canonical executable was running during development and was not stopped or overwritten. The new version is delivered side-by-side; close the old instance before opening this version with the same recovery workspace. `scripts/build.ps1 -Output` supports alternate build paths.

## Previous 1.1.2 build — 30 October 2026

- Path: `dist/ByteForge-Rust.exe`; size: 471,040 bytes; Windows FileVersion/ProductVersion: `1.1.2.0`.
- SHA-256: `42E8EB0844AD5DA6FDAF2C9B4353624AED539065FFE37C29BA96A6D1DA0C4CE3`.
- Format check, seven model/storage tests, Clippy with warnings denied and optimized release build all passed.
- **74 native UI integration checks** passed, including live hex updates and undo, actual 9pt Consolas character formatting, foreground colors after three font choices in both themes, readable highlighted text, empty-result collapse and non-overlapping labeled panes in all three positions.
- **Nine process-stop/restart checks** passed with unchanged logical highlight colors and recovery metadata.
- Local evidence: `output/qa-20261030-090958-580/checks.txt`, `result.txt`, `restart/result.txt`.
- Computer Use verified real typing while the hex pane stayed open: typing ` XYZ` immediately added bytes `20 58 59 5A` to the preview. Switching the editor to Georgia retained light text in dark mode while hex stayed monospace. Light mode showed dark text in both panes. Left and below layouts were inspected with visible headers/divider and aligned ASCII columns; remaining layout paths are covered by native geometry checks.
- The Jaap design-language palette is retained. Dark-mode highlight backgrounds are deeper so the foreground remains consistently light; session metadata continues storing the original logical color.

## Previous 1.1.1 build

- New editors immediately receive the current theme background and foreground after font initialization. Covers New, Open and recovery creation.
- Tab color stripes are 8 logical pixels on the active tab and 6 on inactive tabs; both render paths share the same sizing rule and DPI scaling.
- Size: 462,848 bytes; Windows FileVersion / ProductVersion: `1.1.1.0`.
- SHA-256: `2306E20B1AABDC3035523A8B46E5C77CDE9D09B802F2429F260E3E14981EDFA5`.
- Format, Clippy (warnings denied), release build, seven model/storage tests, **44 native UI checks** and **nine process-stop/restart checks** passed.
- Regression tests read back the actual RichEdit background and default foreground immediately after new tab creation in dark mode, light mode and after switching back.
- Computer Use screenshots confirm freshly created blank tabs use dark/light backgrounds immediately and the thicker color stripes are clearly visible in both modes. The existing design-language palette is preserved.
- Local evidence: `output/qa-20261029-204126-482/checks.txt`, `result.txt`, `restart/result.txt`.

## Original 1.1.0 validation

Validated on Windows x64, 29 October 2026. FASM baseline remains a401721 with a clean original worktree.

## Delivered build

- Path: `dist/ByteForge-Rust.exe`
- Size: 459,776 bytes (449 KiB)
- PE FileVersion / ProductVersion: `1.1.0.0`
- ProductName: `ByteForge Rust`
- SHA-256: `BAB0106791CB38A01318530E5042F28A9580CE6E75000F58AE6063F15EF43DEB`

## Automated evidence

`scripts/build.ps1 -Test -Clippy` passed format checking, seven model/storage unit tests, Clippy on all targets with warnings denied, and the optimized release build.

`tests/ui-smoke.ps1` passed **37 native UI checks**, including actual RichEdit highlighting/clearing, independent tab font/zoom/color/wrap, Unicode search through all unsaved buffers, hidden wrapped tab line numbers/previews, case/scope selection, encoding-preserving save, external file conflict detection, tab reordering, inactive editor visibility, actual Win32 accelerator dispatch, undo/redo across tab switches, replacing, stale result invalidation and disabling/clearing recovery.

The harness forcibly terminated its isolated process after a session snapshot, then started the executable again with the same session directory. **Nine restart checks** confirmed recreated editor text, tab order/active index, independent fonts/colors/zoom/wrap, actual RichEdit highlight/zoom, saved/unsaved state, original disk hash/conflict detection and cross-tab searching after recovery. Both processes launched with a system-only PATH.

Local evidence: `output/qa-20261029-202605-839/checks.txt`, `result.txt`, and `restart/result.txt`. Output/fixtures are intentionally not committed; rerun the harness to regenerate them.

## Visual verification

Computer Use screenshots inspected both themes: native title bar, complete tabstrip, color accents, editor, search inputs, checkbox labels, result list, native popup menu/shortcut columns and About dialog. Verified the light title bar is white and dark controls no longer retain white tabstrip chrome or unreadable checkbox text. Menu labels are exposed through native accessibility. Enter-to-search and Ctrl+F, Ctrl+D, F1 and Escape were exercised through the visible application. Cross-tab results display the correct logical lines and previews. Native popup transition screenshots were followed by settled-state captures.

The shared design-language skill guided the semantic palette, full-control repaint on theme changes, native title bar, accessible menu labels, project-specific embedded icon and screenshot review. The host scaling was inspected; separate 100%, 125% and 150% monitor configurations were not all available. ChooseFont and Windows common file dialogs retain their operating-system chrome.

## Known boundaries

- Undo/redo histories persist during tab switching, not across process restart; document text and appearance do persist.
- Snapshots update every approximately 1.5 seconds. Force termination/power loss can lose changes made since the last successful snapshot.
- Recovery text is readable local data, not encrypted. Disable/clear commands are included and documented.
- Limits: 128 tabs, 32 MiB per document, 128 MiB per snapshot and 10,000 listed search hits. Replace all refuses a capped result set.
- The Rust edition is maintained in the separate `Jaap79/ByteForge-Rust` repository. The preview executable is tracked under `dist/`; no GitHub Release or Rust update channel is configured. Its manual upstream query still identifies the original FASM feed explicitly.
