# Changelog

## Rust edition 1.1.2

- Refresh the visible hex preview immediately after editor changes, undo/redo and replacing text; coalesce native change notifications and preserve hex scroll/selection when refreshing.
- Give the hex viewer its own 9pt Consolas font, independent of per-tab fonts, zoom and font-handle lifetime.
- Restore theme foreground colors after font changes; use light text throughout dark mode and dark text in light mode. Highlights use deeper dark-mode backgrounds while preserving logical session colors.
- Separate editor and hex panes with labeled headers and a visible gap; adapt hex rows to 4/8/16 bytes for available width and collapse empty search results.
- Verify 74 native UI checks and nine process-stop/restart recovery checks, plus seven model/storage tests.

## Rust edition 1.1.1

- Apply the current dark/light editor background immediately when creating any tab, including New and Open.
- Increase tab color stripes to 8 logical pixels for the active tab and 6 for inactive tabs, scaled with DPI.
- Add native regression checks for new tabs in dark/light mode and after switching back, including readable default text.

## Rust edition 1.1.0

- Local fork of FASM baseline a401721 with native Win32/RichEdit UI in Rust.
- Colored tabs, independent undo, font/size, zoom, wrap, selection, scrolling and highlights.
- Autosaved recoverable sessions including unsaved scratch tabs; crash/restart recovery, previous-copy fallback, disable/clear controls and concurrent-writer protection.
- All-tab search with logical line previews, selectable results, case option and replacing; original editing, checksums, file info, hex preview and shortcuts retained.
- Atomic file writes, external-disk-change detection, embedded icon/version resource and complete dark/light palette.
- Original assembler and executable retained under legacy/; original repository unchanged.

## 1.0 RC2 beta 9

- Added right-click editor context menu with Cut, Copy, Paste and Find More.
- Added Find More to prefill the Find dialog from the current text selection.
- Added text highlighting from the Edit menu and selection context menu.
- Supported highlighter colors: red, orange, yellow and green.
- Kept highlighting visual/session-only so plain-text file contents remain unchanged.

## 1.0 RC2 beta 8

- Removed the experimental document preview/parser work from the mainline build.
- Restored the build to a single FASM executable.
- Kept the empty-document save prompt fix for brand-new untouched files.

## 1.0 RC1

- Added manual Help -> Check for Updates using the public repository `version.json`.
- Promoted ByteForge to 1.0 RC1.
- Standardized release executable name to `Byteforge.exe`.
- Added public bug/security reporting contact: `jaapengel79@proton.me`.
- Removed remaining legacy project-name references from source and documentation.

## 1.0 beta 7

- Renamed application branding to ByteForge.
- Added executable version resource metadata:
  - `InternalName`: ByteForge
  - `ProductName`: ByteForge
  - `FileDescription`: Small and fast text editor without fluff
- Added selectable checksum dialog with optional expected MD5 and SHA-256 verification.
- Added colored MATCH / MISMATCH feedback in checksum comparison.
- Added File Info dialog with path, format, counts, save state and checksums.
- Added text zoom in/out/reset for the main editor.
- Added Jump to line/character.
- Improved synchronized hex viewer behavior for search and jump.
- Tweaked hex viewer layout, dynamic width and right margin.
- Limited repeated "No more matches found" popups.

## 1.0 beta 6

- Added Explorer file association / Open With startup support.
- Added Unicode command-line parsing with `GetCommandLineW` and `CommandLineToArgvW`.
- Unified Drag & Drop, File Open, Open With and Explorer startup through the same open implementation.
