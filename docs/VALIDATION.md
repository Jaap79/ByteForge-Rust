# Validation — ByteForge Rust 1.1.0

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
- The Rust edition is a local fork, not a published GitHub release. Its manual upstream query identifies the original FASM feed explicitly; no Rust update channel is configured.
