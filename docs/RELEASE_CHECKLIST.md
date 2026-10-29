# Rust edition release checklist

- Run `scripts/build.ps1 -Test -Clippy` (format check, unit tests, Clippy with warnings denied, release build).
- Run `tests/ui-smoke.ps1` (isolated native controls plus forced termination/restart recovery).
- Verify PE FileVersion/ProductVersion and package version agree.
- Inspect dark and light modes, popup menu, search/results, colored tabs and a dialog; verify title-bar colors and readable controls.
- Verify all menu commands have unique keyboard mappings; confirm find/replace fields retain native clipboard/select-all shortcuts.
- Check only the active editor is visible after snapshots/theme switches.
- Verify executable launch with system-only PATH, no bundled runtime DLLs.
- Keep the original FASM repository untouched and retain its baseline in `legacy/`.
- Do not publish the Rust edition as a FASM update. Set up a distinct release/feed before enabling Rust update comparisons.
- Record tested build SHA-256 and evidence in `docs/VALIDATION.md`.
