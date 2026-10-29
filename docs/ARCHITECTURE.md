# ByteForge Rust architecture

Fork baseline: ByteForge FASM 1.0.0.10, commit a401721. Rust edition 1.1.0 targets native Windows x64. The original assembler and executable are retained under `legacy/`; no FASM files are required at Rust runtime.

## State ownership

`Document` owns normalized Unicode text, disk identity, dirty state, encoding/EOL and appearance. Each live `Tab` owns a separate RichEdit HWND and HFONT; native RichEdit retains per-tab undo/redo during the process. Tab reordering moves the whole `Tab`. Switching does not copy plain text through a shared control and therefore retains native formatting/undo.

The UI thread owns `App` through a thread-local RefCell. Reentrant native notifications use `try_borrow_mut`; paint/menu helpers and control subclasses can operate independently. Programmatic edits explicitly update capture state because reentrant EN_CHANGE is suppressed. User EN_CHANGE marks a tab for capture. UTF-16 CP positions are used at the FFI boundary; document paragraphs use one LF, matching RichEdit's one paragraph CP. Search line/column calculation uses document text, independent of wrap layout.

`native.rs` is the raw FFI/theme/dialog boundary. `ui/mod.rs` owns document commands, tab/search presentation and the Win32 message loop. `platform.rs` only performs the explicitly requested FASM upstream version query on a background thread. No network action occurs on startup.

## Persistence

The BFRSESS1 binary format is length-prefixed and checksummed with CRC-32 (corruption detection, not authentication). It preserves Windows path UTF-16, Unicode text and document settings. Decoder bounds: 128 tabs, 32 MiB text per document, 128 MiB snapshot, valid setting ranges and ordered non-overlapping highlight spans. Unknown/truncated/trailing/corrupt input is rejected. CRC does not make a snapshot trustworthy; parsing is bounded regardless.

Snapshot files are flushed then replaced atomically using ReplaceFileW/MoveFileExW. Before replacing the current valid session, its bytes are atomically saved as the previous snapshot. Corrupt current files never replace a good backup. If both snapshots are unusable, automatic writes pause. The user's files are never overwritten during recovery. Normal exit with recovery enabled must successfully snapshot before closing. Crash recovery can lose the most recent 1.5-second interval; hard power-loss durability depends on Windows/storage flush guarantees.

SHA-256 of the last opened/saved disk bytes is persisted independently of buffer text. A changed/deleted source file requires a decision before save. Save As refuses a path owned by another tab. Atomic document writes preserve the old file when replacement fails. Format settings remain session metadata; plain text remains plain text.

An exclusive file handle with share mode 0 prevents two processes from writing one session directory. Recovery OFF is remembered by an explicit marker; clearing recovery removes only the two known snapshot files, never recursive directories or user documents. Recovery files contain readable text under the current user's profile.

## Interface

The shared Jaap palette has semantic roles for dark/light surfaces, text, borders and accent. Native title bars are themed via DWM. Menus are owner-drawn while retaining labels/access keys. The native TabControl keeps its accessibility and hit testing; a subclass paints its full surface and per-tab color stripe. Checkbox painting preserves native toggle/keyboard semantics. DPI changes rebuild fonts and control sizes. Each theme change redraws the full child tree.

WM_SETREDRAW restores WS_VISIBLE; highlight snapshotting and theme helpers explicitly preserve prior visibility so hidden editors stay hidden. This has a native UI regression check.

## Tests

Unit tests cover Unicode/encoding/EOL round trips, strict input rejection, session corruption/truncation, backup fallback, known hash vectors, atomic save and external-file conflicts. The storage test locks the destination and proves a failed replacement neither truncates it nor leaves temporary files. Native tests cover actual RichEdit selection/highlight/zoom, tab settings, search in hidden wrapped tabs, save, undo/redo, replacing, snapshots and process-termination/restart recovery. GUI QA also inspects dark/light native chrome, menus, search, tabs and dialogs.
