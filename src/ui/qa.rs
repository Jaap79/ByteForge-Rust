use super::*;
thread_local! {pub static OUTPUT:RefCell<Option<PathBuf>>=const{RefCell::new(None)};}
thread_local! {static RECEIVED:std::cell::Cell<u16>=const{std::cell::Cell::new(0)};}
unsafe fn foreground(hwnd: HWND, r: Range) -> u32 {
    let original = range(hwnd);
    set_range(hwnd, r);
    let mut cf: CharFormat = zeroed();
    cf.size = size_of::<CharFormat>() as u32;
    SendMessageW(hwnd, 0x43a, 1, &mut cf as *mut _ as isize);
    set_range(hwnd, original);
    cf.color
}
unsafe fn window_rect(hwnd: HWND) -> RECT {
    let mut r: RECT = zeroed();
    GetWindowRect(hwnd, &mut r);
    r
}
unsafe extern "system" fn answer_prompt(hwnd: HWND, choice: isize) -> i32 {
    let mut name = [0u16; 128];
    let n = GetClassNameW(hwnd, name.as_mut_ptr(), 128);
    if String::from_utf16_lossy(&name[..n.max(0) as usize]) == "ByteForgeRustPrompt" {
        PostMessageW(hwnd, WM_COMMAND, choice as usize, 0);
    }
    1
}
unsafe extern "system" fn cancel_close(_hwnd: HWND, _msg: u32, _id: usize, _time: u32) {
    EnumThreadWindows(
        windows_sys::Win32::System::Threading::GetCurrentThreadId(),
        Some(answer_prompt),
        3,
    );
}
unsafe extern "system" fn discard_close(_hwnd: HWND, _msg: u32, _id: usize, _time: u32) {
    EnumThreadWindows(
        windows_sys::Win32::System::Threading::GetCurrentThreadId(),
        Some(answer_prompt),
        2,
    );
}
unsafe extern "system" fn accelerator_probe(hwnd: HWND, msg: u32, wp: usize, lp: isize) -> isize {
    if msg == WM_COMMAND {
        RECEIVED.set((wp & 65535) as u16);
        return 0;
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}
unsafe fn verify_accelerators() -> bool {
    let instance = GetModuleHandleW(null());
    let class = wide("ByteForgeAcceleratorProbe");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(accelerator_probe),
        hInstance: instance,
        lpszClassName: class.as_ptr(),
        ..zeroed()
    };
    RegisterClassW(&wc);
    let hwnd = CreateWindowExW(
        0,
        class.as_ptr(),
        class.as_ptr(),
        0,
        0,
        0,
        0,
        0,
        HWND_MESSAGE,
        null_mut(),
        instance,
        null(),
    );
    let entries = accelerators();
    let table = CreateAcceleratorTableW(entries.as_ptr(), entries.len() as i32);
    if table.is_null() {
        DestroyWindow(hwnd);
        return false;
    }
    let mut old = [0u8; 256];
    GetKeyboardState(old.as_mut_ptr());
    let mut valid = true;
    for a in &entries {
        let mut state = [0u8; 256];
        for (flag, key, left) in [
            (FCONTROL, VK_CONTROL, VK_LCONTROL),
            (FSHIFT, VK_SHIFT, VK_LSHIFT),
            (FALT, VK_MENU, VK_LMENU),
        ] {
            if a.fVirt & flag != 0 {
                state[key as usize] = 128;
                state[left as usize] = 128;
            }
        }
        state[a.key as usize] = 128;
        SetKeyboardState(state.as_ptr());
        RECEIVED.set(0);
        let msg = MSG {
            hwnd,
            message: if a.fVirt & FALT != 0 {
                WM_SYSKEYDOWN
            } else {
                WM_KEYDOWN
            },
            wParam: a.key as usize,
            lParam: 1,
            ..zeroed()
        };
        let handled = TranslateAcceleratorW(hwnd, table, &msg);
        if handled == 0 || RECEIVED.get() != a.cmd {
            valid = false;
            break;
        }
    }
    SetKeyboardState(old.as_ptr());
    DestroyAcceleratorTable(table);
    DestroyWindow(hwnd);
    valid
}
pub unsafe fn run(app: &mut App) {
    let output = OUTPUT.with(|o| o.borrow().clone()).unwrap();
    let _ = fs::create_dir_all(&output);
    let verify = std::env::args_os().any(|a| a == "--verify-recovery");
    let result = if verify {
        verify_recovery(app)
    } else {
        exercise(app, &output)
    };
    let text = match result {
        Ok(n) => format!("PASS: {n} native UI integration checks\n"),
        Err(e) => format!("FAIL: {e}\n"),
    };
    let _ = fs::write(output.join("result.txt"), text);
    if !std::env::args_os().any(|a| a == "--crash-fixture") {
        DestroyWindow(app.hwnd);
    }
}
unsafe fn verify_recovery(app: &mut App) -> Result<usize, String> {
    if app.documents.len() != 3 || app.active != 1 {
        return Err("Tabs/active index did not restore after process termination".into());
    }
    if app.documents[0].doc.context != Context::Json
        || app.documents[1].doc.context != Context::PowerShell
        || app.documents[2].doc.context != Context::Plain
    {
        return Err("Independent syntax contexts did not restore".into());
    }
    if document::normalize(&raw_text(app.documents[0].edit))
        != "alpha café 😀\nneedle first\nneedle again"
    {
        return Err("Unsaved editor contents did not restore".into());
    }
    if app.documents[0].doc.font != "Georgia"
        || app.documents[0].doc.zoom != 110
        || app.documents[0].doc.color != 4
        || !app.documents[0].doc.wrap
    {
        return Err("First tab appearance did not restore".into());
    }
    if app.documents[1].doc.font != "Consolas"
        || app.documents[1].doc.zoom != 90
        || app.documents[1].doc.color != 5
    {
        return Err("Independent second-tab appearance did not restore".into());
    }
    app.capture_highlights(0);
    if app.documents[0].doc.highlights
        != vec![Highlight {
            start: 0,
            end: 5,
            color: 0x80FF80,
        }]
    {
        return Err("Live RichEdit highlight was not recreated".into());
    }
    let mut num = 0i32;
    let mut den = 0i32;
    SendMessageW(
        app.documents[0].edit,
        0x4e0,
        &mut num as *mut _ as usize,
        &mut den as *mut _ as isize,
    );
    if num * 100 != den * 110 {
        return Err(format!("Live zoom not restored: {num}/{den}"));
    }
    if !app.documents[0].doc.dirty || app.documents[2].doc.dirty {
        return Err("Dirty/saved tab state not restored".into());
    }
    if !storage::disk_changed(
        app.documents[2].doc.path.as_ref().unwrap(),
        app.documents[2].doc.disk_hash.as_deref(),
    ) {
        return Err("External file change was lost during recovery".into());
    }
    app.show_search(true);
    set_text(app.query, "needle");
    app.search_all();
    if app.hits.len() != 4 {
        return Err("Search across restored live controls failed".into());
    }
    Ok(10)
}
unsafe fn exercise(app: &mut App, output: &Path) -> Result<usize, String> {
    let mut checks = vec![];
    macro_rules! check {
        ($condition:expr,$label:expr) => {
            if !$condition {
                return Err(format!("{} (after {} checks)", $label, checks.len()));
            }
            checks.push($label.to_string());
        };
    }
    check!(app.documents.len() == 1, "Starts with one scratch tab");
    set_raw(app.edit(), "alpha café 😀\nneedle first\nneedle again");
    app.documents[0].changed = true;
    app.capture(0);
    check!(app.documents[0].doc.dirty, "Editing scratch marks dirty");
    app.command(HL_RED); // no selection, must remain unchanged
    set_range(app.edit(), Range { start: 0, end: 5 });
    app.command(HL_YELLOW);
    app.capture_highlights(0);
    check!(
        app.documents[0].doc.highlights
            == vec![Highlight {
                start: 0,
                end: 5,
                color: 0xFFFF00
            }],
        "Native highlight span is captured"
    );
    app.command(CLEAR);
    app.capture_highlights(0);
    check!(
        app.documents[0].doc.highlights.is_empty(),
        "Clear restores automatic background"
    );
    app.command(HL_GREEN);
    app.command(FONT_GEORGIA);
    app.command(ZOOM_IN);
    app.command(COLOR_BASE + 4);
    app.command(WRAP);
    check!(
        app.documents[0].doc.font == "Georgia"
            && app.documents[0].doc.zoom == 110
            && app.documents[0].doc.color == 4,
        "First tab owns font zoom color"
    );
    app.command(NEW);
    check!(
        SendMessageW(app.edit(), 0x443, 0, palette().control as isize) as u32 == palette().control,
        "New tab immediately uses the dark editor background"
    );
    // A mode change followed by another creation must inherit the mode without
    // relying on a subsequent apply_theme call to repair the new editor.
    app.command(DARK_MODE);
    app.command(NEW);
    check!(
        SendMessageW(app.edit(), 0x443, 0, palette().control as isize) as u32 == palette().control,
        "New tab immediately uses the light editor background"
    );
    let mut cf: CharFormat = zeroed();
    cf.size = size_of::<CharFormat>() as u32;
    SendMessageW(app.edit(), 0x43a, 0, &mut cf as *mut _ as isize);
    check!(
        cf.color == palette().text,
        format!(
            "New light tab has readable default text (color={:#x}, mask={:#x}, effects={:#x})",
            cf.color, cf.mask, cf.effects
        )
    );
    check!(app.close_tab(), "Empty themed test tab closes cleanly");
    app.command(DARK_MODE);
    app.command(NEW);
    check!(
        SendMessageW(app.edit(), 0x443, 0, palette().control as isize) as u32 == palette().control,
        "New tab inherits dark mode after switching back"
    );
    let mut cf: CharFormat = zeroed();
    cf.size = size_of::<CharFormat>() as u32;
    SendMessageW(app.edit(), 0x43a, 0, &mut cf as *mut _ as isize);
    check!(
        cf.color == palette().text,
        format!(
            "New dark tab has readable default text (color={:#x}, mask={:#x}, effects={:#x})",
            cf.color, cf.mask, cf.effects
        )
    );
    check!(app.close_tab(), "Extra dark test tab closes cleanly");
    set_raw(app.edit(), "second tab\nNEEDLE second\n😀 needle third");
    app.documents[1].changed = true;
    app.capture(1);
    check!(app.documents.len() == 2, "New creates another editor");
    check!(
        app.documents[1].doc.font == "Consolas" && app.documents[1].doc.zoom == 100,
        "New tab does not inherit first tab settings"
    );
    app.command(COLOR_BASE + 5);
    app.command(ZOOM_OUT);
    app.show_search(true);
    set_text(app.query, "needle");
    app.search_all();
    check!(
        app.hits.len() == 4,
        "Cross-tab search sees all unsaved buffers case-insensitively"
    );
    check!(
        app.hits[0].line == 2 && app.hits[0].preview == "needle first" && app.hits[1].line == 3,
        "Hidden wrapped tabs report logical line numbers and previews"
    );
    check!(
        app.hits.iter().any(|h| h.tab == 1 && h.line == 2),
        "Search result includes tab and line"
    );
    let index = app
        .hits
        .iter()
        .position(|h| h.tab == 1 && h.line == 3)
        .unwrap();
    app.select_hit(index);
    check!(
        app.active == 1 && selected_text(app.edit()) == "needle",
        "Selecting result activates correct tab and Unicode CP selection"
    );
    SendMessageW(app.case, BM_SETCHECK, 1, 0);
    app.search_all();
    check!(app.hits.len() == 3, "Match case search works across tabs");
    SendMessageW(app.case, BM_SETCHECK, 0, 0);
    SendMessageW(app.scope, BM_SETCHECK, 0, 0);
    app.search_all();
    check!(app.hits.len() == 2, "Current-tab scope is isolated");
    app.activate(0);
    check!(
        app.documents[0].doc.zoom == 110 && app.documents[0].doc.font == "Georgia",
        "Tab switch retains independent settings"
    );
    app.command(HEX_RIGHT);
    check!(
        raw_text(app.hex).contains("61 6C 70 68 61"),
        "Hex viewer encodes active buffer"
    );
    check!(
        foreground(app.hex, Range { start: 0, end: 6 }) == palette().text,
        "Hex viewer uses light foreground in dark mode"
    );
    let mut hex_cf: CharFormat = zeroed();
    hex_cf.size = size_of::<CharFormat>() as u32;
    SendMessageW(app.hex, 0x43a, 0, &mut hex_cf as *mut _ as isize);
    let hex_face = String::from_utf16_lossy(
        &hex_cf.face[..hex_cf.face.iter().position(|c| *c == 0).unwrap_or(32)],
    );
    check!(
        hex_face == "Consolas" && hex_cf.height == 180 && app.hex_font != app.documents[0].font,
        format!(
            "Hex viewer renders its own 9pt monospace font (face={hex_face}, height={})",
            hex_cf.height
        )
    );
    for dark in [false, true] {
        if DARK.get() != dark {
            app.command(DARK_MODE);
        }
        for choice in [FONT_SEGOE, FONT_CONSOLAS, FONT_GEORGIA] {
            app.command(choice);
            check!(
                foreground(app.edit(), Range { start: 8, end: 10 }) == palette().text,
                format!("Font switch keeps editor foreground in mode dark={dark}")
            );
            check!(
                foreground(app.edit(), Range { start: 0, end: 5 }) == palette().text,
                format!("Highlighted text follows foreground in mode dark={dark}")
            );
            check!(
                foreground(app.hex, Range { start: 0, end: 6 }) == palette().text,
                format!("Hex foreground survives editor font change in mode dark={dark}")
            );
        }
    }
    // Font choices reset font size but must not affect this tab's independent zoom.
    let before_hex = raw_text(app.hex);
    let original_text = app.documents[0].doc.text.clone();
    let original_range = range(app.edit());
    set_range(app.edit(), Range { start: -1, end: -1 });
    SendMessageW(app.edit(), EM_REPLACESEL, 1, wide("!").as_ptr() as isize);
    app.editor_changed(app.edit());
    check!(
        app.hex_refresh_queued,
        "Typing queues a live hex refresh immediately"
    );
    app.flush_hex_refresh();
    check!(
        raw_text(app.hex) != before_hex && raw_text(app.hex).contains("21 "),
        "Queued hex refresh displays newly typed bytes without reopening the view"
    );
    app.command(UNDO);
    check!(
        app.documents[0].doc.text == original_text && raw_text(app.hex) == before_hex,
        "Undo refreshes hex back to original bytes"
    );
    set_range(app.edit(), original_range);
    app.hits.clear();
    app.layout();
    check!(
        GetWindowLongPtrW(app.results, GWL_STYLE) & WS_VISIBLE as isize == 0,
        "Empty search results do not reserve a blank panel"
    );
    for mode in [HEX_LEFT, HEX_RIGHT, HEX_BELOW] {
        app.command(mode);
        let editor = window_rect(app.edit());
        let hex = window_rect(app.hex);
        let editor_header = window_rect(app.editor_label);
        let hex_header = window_rect(app.hex_label);
        check!(
            editor.top == editor_header.bottom && hex.top == hex_header.bottom,
            "Each split pane has a distinct header"
        );
        check!(
            match mode {
                HEX_LEFT => hex.right < editor.left && hex.top == editor.top,
                HEX_RIGHT => editor.right < hex.left && hex.top == editor.top,
                _ => editor.bottom < hex_header.top,
            },
            "Hex and editor panes have a visible non-overlapping divider"
        );
    }
    app.command(HEX_CLOSE);
    let file = output.join("utf16.txt");
    let bytes = [
        vec![0xFF, 0xFE],
        "disk\r\noriginal"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect(),
    ]
    .concat();
    storage::atomic_write(&file, &bytes)?;
    app.open(&file);
    check!(
        app.documents.len() == 3 && app.documents[2].doc.encoding == document::Encoding::Utf16Le,
        "Open detects UTF-16 BOM into a new tab"
    );
    let before = app.documents.len();
    app.open(&file);
    check!(
        app.documents.len() == before,
        "Opening same path selects existing tab"
    );
    set_raw(app.edit(), "changed\n😀");
    app.documents[2].changed = true;
    check!(app.save(2, false), "Native save succeeds");
    check!(
        fs::read(&file).map_err(|e| e.to_string())? == app.documents[2].doc.bytes(),
        "Save preserves UTF-16 encoding and CRLF"
    );
    storage::atomic_write(&file, b"external update")?;
    check!(
        storage::disk_changed(&file, app.documents[2].doc.disk_hash.as_deref()),
        "External disk modification is detected"
    );
    app.activate(1);
    app.snapshot()?;
    let (saved, backup) = session::load(&app.dir)?.unwrap();
    check!(
        !backup && saved.documents.len() == 3 && saved.active == 1,
        "Recovery stores all tabs and active tab"
    );
    check!(
        saved.documents[0].font == "Georgia"
            && saved.documents[1].zoom == 90
            && saved.documents[0].highlights[0].color == 0x80FF80,
        "Recovery includes per-tab formatting"
    );
    check!(
        saved.documents[0].text.contains('😀') && saved.documents[0].dirty,
        "Recovery includes Unicode unsaved scratch contents"
    );
    for dark in [false, true] {
        DARK.set(dark);
        brushes();
        app.apply_theme();
        check!(
            storage::hashes(app.documents[0].doc.text.as_bytes())?.1
                == storage::hashes(saved.documents[0].text.as_bytes())?.1,
            "Theme switch preserves document contents"
        );
    }
    app.command(MOVE_LEFT);
    check!(
        app.active == 0 && app.documents[0].doc.zoom == 90,
        "Reordering moves whole tab state"
    );
    app.command(MOVE_RIGHT);
    app.capture_highlights(0);
    check!(
        app.documents[0].doc.highlights[0].color == 0x80FF80,
        "Theme switches preserve highlight colors"
    );
    let entries = accelerators();
    check!(
        app.documents
            .iter()
            .enumerate()
            .all(
                |(i, t)| (GetWindowLongPtrW(t.edit, GWL_STYLE) & WS_VISIBLE as isize != 0)
                    == (i == app.active)
            ),
        "Snapshot and theme changes keep inactive editors hidden"
    );
    let mut keys = std::collections::HashSet::new();
    check!(
        entries.iter().all(|a| keys.insert((a.fVirt, a.key))),
        "Accelerator keys have no duplicates"
    );
    check!(
        verify_accelerators(),
        "Every shortcut dispatches its actual Win32 accelerator command"
    );
    // Exercise editing in a disposable fourth tab, leaving the crash fixture unchanged.
    app.command(NEW);
    set_raw(app.edit(), "test text");
    SendMessageW(app.edit(), EM_EMPTYUNDOBUFFER, 0, 0);
    set_range(app.edit(), Range { start: 9, end: 9 });
    SendMessageW(app.edit(), EM_REPLACESEL, 1, wide("!").as_ptr() as isize);
    app.documents[3].changed = true;
    app.command(UNDO);
    check!(
        raw_text(app.edit()) == "test text",
        "Undo reverses the active tab's edit"
    );
    app.command(REDO);
    check!(
        raw_text(app.edit()) == "test text!",
        "Redo reapplies the active tab's edit"
    );
    app.activate(0);
    app.activate(3);
    app.command(UNDO);
    check!(
        raw_text(app.edit()) == "test text",
        "Undo history survives tab switching"
    );
    app.show_search(false);
    set_text(app.query, "text");
    set_text(app.replace, "replacement");
    app.search_all();
    app.replace_hits(false);
    check!(
        raw_text(app.edit()) == "test replacement",
        "Replace changes the selected match"
    );
    app.capture(3);
    check!(
        app.hits.is_empty(),
        "Editing invalidates stale search offsets"
    );
    app.documents[3].doc.dirty = false;
    check!(app.close_tab(), "Unmodified tab can be closed");
    app.activate(1);
    // Validate the explicit preference file without prompting the test operator.
    let original = app.edit();
    let old_color = app.documents[0].doc.color;
    app.set_tab_color(0, 10);
    check!(
        app.edit() == original && app.documents[0].doc.color == 10,
        "Recoloring an inactive tab keeps the active document"
    );
    let mut colored_item: TCITEMW = zeroed();
    colored_item.mask = TCIF_PARAM;
    SendMessageW(
        app.tabs,
        TCM_GETITEMW,
        0,
        &mut colored_item as *mut _ as isize,
    );
    check!(
        colored_item.lParam == 10,
        "Changing an inactive tab color updates its native paint metadata immediately"
    );
    if IsWindowVisible(app.hwnd) != 0 {
        let mut r: RECT = zeroed();
        SendMessageW(app.tabs, TCM_GETITEMRECT, 0, &mut r as *mut _ as isize);
        let dc = GetDC(app.tabs);
        let pixel = GetPixel(dc, r.left + 8, r.top + 2);
        ReleaseDC(app.tabs, dc);
        check!(
            pixel == rgb(TAB_COLORS[10].1),
            format!("Visible tab stripe matches the new color: {pixel:#x}")
        );
    }
    app.snapshot()?;
    check!(
        session::load(&app.dir)?.unwrap().0.documents[0].color == 10,
        "Extra tab colors persist in session snapshots"
    );
    app.set_tab_color(0, old_color);
    let (m, pop) = tab_menu(10);
    let colors = GetSubMenu(pop, 0);
    check!(
        GetMenuItemCount(colors) == 11
            && GetMenuState(colors, (COLOR_BASE + 10) as u32, MF_BYCOMMAND) & MF_CHECKED != 0,
        "Tab context menu lists eleven colors and checks its own color"
    );
    DestroyMenu(pop);
    DestroyMenu(m.root);
    let (m, pop) = editor_menu(true);
    let mark = GetSubMenu(pop, 4);
    check!(
        GetMenuItemCount(pop) == 5 && !mark.is_null() && GetMenuItemCount(mark) == 5,
        "Editor context menu groups all markings under one compact submenu"
    );
    DestroyMenu(pop);
    DestroyMenu(m.root);
    let (m, pop) = editor_menu(false);
    check!(
        GetMenuState(GetSubMenu(pop, 4), CLEAR as u32, MF_BYCOMMAND) & MF_GRAYED != 0,
        "Text markings are disabled without a selection"
    );
    DestroyMenu(pop);
    DestroyMenu(m.root);
    app.command(GetDlgCtrlID(app.new_tab) as u16);
    check!(
        app.documents.len() == 4 && app.documents[3].doc.text.is_empty(),
        "Plus button command creates a blank independent tab"
    );
    app.documents[3].doc.text = "Close prompt fixture".into();
    set_raw(app.edit(), "Close prompt fixture");
    app.documents[3].doc.dirty = true;
    app.activate(1);
    let original = app.edit();
    let timer = SetTimer(null_mut(), 0, 20, Some(cancel_close));
    let closed = app.close_tab_at(3);
    KillTimer(null_mut(), timer);
    check!(
        !closed && app.documents.len() == 4 && app.edit() == original && app.documents[3].doc.dirty,
        "Cancelling an inactive-tab close retains its unsaved text and original active tab"
    );
    let timer = SetTimer(null_mut(), 0, 20, Some(discard_close));
    let closed = app.close_tab_at(3);
    KillTimer(null_mut(), timer);
    check!(
        closed && app.documents.len() == 3 && app.edit() == original,
        "Discard closes only the requested inactive tab"
    );
    let bar = window_rect(app.tabs);
    let plus = window_rect(app.new_tab);
    check!(
        plus.right <= window_rect(app.hwnd).right,
        "Plus button stays within the window"
    );
    for i in 0..app.documents.len() {
        let mut r: RECT = zeroed();
        SendMessageW(app.tabs, TCM_GETITEMRECT, i, &mut r as *mut _ as isize);
        let close = tab_close_rect(r, app.dpi as i32);
        let pt = POINT {
            x: (close.left + close.right) / 2,
            y: (close.top + close.bottom) / 2,
        };
        check!(
            tab_hit(app.tabs, pt) == Some((i, true)),
            "Close glyph hit area identifies the correct tab"
        );
    }
    for item in app.menus.items.iter().filter(|i| i.top) {
        let mut measure: MEASUREITEMSTRUCT = zeroed();
        measure.CtlType = ODT_MENU;
        measure.itemData = &**item as *const MenuItem as usize;
        measure_menu(&mut measure as *mut _ as isize);
        let dc = GetDC(app.hwnd);
        let old = SelectObject(dc, ui_font());
        let label = wide(&item.label.replace('&', ""));
        let mut size: SIZE = zeroed();
        GetTextExtentPoint32W(dc, label.as_ptr(), (label.len() - 1) as i32, &mut size);
        let dpi = GetDeviceCaps(dc, LOGPIXELSX as i32);
        SelectObject(dc, old);
        ReleaseDC(app.hwnd, dc);
        check!(
            measure.itemWidth as i32 >= size.cx + 16 * dpi / 96,
            "Menu-bar measurement reserves both text insets without clipping"
        );
    }
    let mut last: RECT = zeroed();
    SendMessageW(
        app.tabs,
        TCM_GETITEMRECT,
        app.documents.len() - 1,
        &mut last as *mut _ as isize,
    );
    check!(
        (plus.left - bar.left - last.right).abs() <= app.scale(8),
        "Plus button follows the last tab instead of the window edge"
    );
    app.command(NEW);
    let fixture="function Get-Thing {\n  param([string] $Name)\n  # comment\n  Get-Process -Name \"hello $Name\"\n}";
    set_raw(app.edit(), fixture);
    app.documents[app.active].changed = true;
    set_range(app.edit(), Range { start: 0, end: 0 });
    check!(
        syntax::UndoPause::new(app.edit()).is_some(),
        "RichEdit supports undo-preserving syntax decoration"
    );
    app.command(CONTEXT_BASE + Context::PowerShell as u16);
    check!(
        foreground(app.edit(), Range { start: 0, end: 8 })
            == rgb(byteforge_rust::syntax::Kind::Keyword.color(DARK.get())),
        "PowerShell keywords receive actual native token colors"
    );
    let ps_edit = app.edit();
    let original_range = range(ps_edit);
    app.paint_syntax(true);
    check!(
        range(ps_edit).start == original_range.start
            && range(ps_edit).end == original_range.end
            && raw_text(ps_edit).replace("\r\n", "\n") == fixture,
        "Syntax rendering preserves text and selection"
    );
    SendMessageW(ps_edit, EM_EMPTYUNDOBUFFER, 0, 0);
    set_range(
        ps_edit,
        Range {
            start: fixture.encode_utf16().count() as i32,
            end: fixture.encode_utf16().count() as i32,
        },
    );
    SendMessageW(
        ps_edit,
        EM_REPLACESEL,
        1,
        wide("\nGet-Service").as_ptr() as isize,
    );
    app.documents[app.active].changed = true;
    app.capture(app.active);
    app.paint_syntax(true);
    app.command(UNDO);
    app.capture(app.active);
    app.paint_syntax(true);
    check!(
        document::normalize(&raw_text(ps_edit)) == fixture,
        "One undo removes typed text without undoing syntax colors"
    );
    app.command(REDO);
    app.capture(app.active);
    app.paint_syntax(true);
    check!(
        document::normalize(&raw_text(ps_edit)).ends_with("Get-Service"),
        "Syntax decoration retains redo history"
    );
    for dark in [false, true] {
        DARK.set(dark);
        brushes();
        app.apply_theme();
        check!(
            foreground(ps_edit, Range { start: 0, end: 8 })
                == rgb(byteforge_rust::syntax::Kind::Keyword.color(dark)),
            "Syntax foreground switches with dark/light mode"
        );
    }
    set_range(ps_edit, Range { start: 0, end: 8 });
    app.highlight(Some(HIGHLIGHTS[2]));
    app.paint_syntax(true);
    check!(
        foreground(ps_edit, Range { start: 0, end: 8 }) == palette().text,
        "Manual highlighter keeps readable foreground over syntax colors"
    );
    app.highlight(None);
    app.paint_syntax(true);
    check!(
        foreground(ps_edit, Range { start: 0, end: 8 })
            == rgb(byteforge_rust::syntax::Kind::Keyword.color(true)),
        "Removing a manual mark restores the syntax foreground"
    );
    app.command(FONT_GEORGIA);
    check!(
        foreground(ps_edit, Range { start: 0, end: 8 })
            == rgb(byteforge_rust::syntax::Kind::Keyword.color(true)),
        "Changing a tab font preserves syntax colors"
    );
    app.move_tab(3, 0);
    check!(
        app.active == 0
            && app.edit() == ps_edit
            && app.documents[0].doc.context == Context::PowerShell
            && IsWindowVisible(app.documents[1].edit) == 0,
        "Moving a tab keeps its active editor context and hides the other editors"
    );
    check!(
        foreground(ps_edit, Range { start: 0, end: 8 })
            == rgb(byteforge_rust::syntax::Kind::Keyword.color(true)),
        "Moved editor remains rendered with its syntax formatting"
    );
    app.move_tab(0, 3);
    let mut r0: RECT = zeroed();
    let mut r2: RECT = zeroed();
    SendMessageW(app.tabs, TCM_GETITEMRECT, 0, &mut r0 as *mut _ as isize);
    SendMessageW(app.tabs, TCM_GETITEMRECT, 2, &mut r2 as *mut _ as isize);
    let packed = |x: i32, y: i32| ((y as u32) << 16 | (x as u32 & 65535)) as i32 as isize;
    SendMessageW(
        app.tabs,
        WM_LBUTTONDOWN,
        1,
        packed(r0.left + 25, r0.top + 15),
    );
    SendMessageW(app.tabs, WM_MOUSEMOVE, 1, packed(r2.left + 25, r2.top + 15));
    check!(
        TAB_DRAG
            .get()
            .is_some_and(|d| d.dragging && d.from == 0 && d.target == 2),
        "Native mouse drag crosses the threshold and identifies the drop target"
    );
    SendMessageW(app.tabs, WM_LBUTTONUP, 0, packed(r2.left + 25, r2.top + 15));
    let mut message: MSG = zeroed();
    check!(
        PeekMessageW(&mut message, app.hwnd, MOVE_TAB, MOVE_TAB, PM_REMOVE) != 0
            && message.wParam == 0
            && message.lParam == 2,
        "Native mouse release posts the correct tab reorder command"
    );
    while PeekMessageW(&mut message, app.hwnd, ACTIVE_TAB, ACTIVE_TAB, PM_REMOVE) != 0 {}
    app.command(CONTEXT_BASE + Context::Plain as u16);
    check!(
        foreground(ps_edit, Range { start: 0, end: 8 }) == palette().text
            && app.documents[0].doc.context == Context::Plain,
        "Plain text clears syntax colors on the current tab only"
    );
    set_raw(
        ps_edit,
        "{\"name\": \"ByteForge\", \"ready\": true, \"count\": 3}",
    );
    app.documents[app.active].changed = true;
    app.command(CONTEXT_BASE + Context::Json as u16);
    check!(
        foreground(ps_edit, Range { start: 1, end: 7 })
            == rgb(byteforge_rust::syntax::Kind::Key.color(true)),
        "JSON property names receive their distinct key color"
    );
    app.snapshot()?;
    check!(
        session::load(&app.dir)?.unwrap().0.documents[3].context == Context::Json,
        "Syntax context persists per tab in recovery snapshots"
    );
    let scroll_text = "Get-Item $name\n".repeat(1000);
    set_raw(ps_edit, &scroll_text);
    app.documents[app.active].changed = true;
    app.command(CONTEXT_BASE + Context::PowerShell as u16);
    let cache_revision = app.documents[app.active].syntax.borrow().cache.revision;
    SendMessageW(ps_edit, EM_LINESCROLL, 0, 300);
    let first_line = SendMessageW(ps_edit, EM_GETFIRSTVISIBLELINE, 0, 0);
    let cp = SendMessageW(ps_edit, EM_LINEINDEX, first_line as usize, 0) as i32;
    let mut scroll_before: POINT = zeroed();
    SendMessageW(ps_edit, 0x4dd, 0, &mut scroll_before as *mut _ as isize);
    app.paint_syntax(false);
    let mut scroll_after: POINT = zeroed();
    SendMessageW(ps_edit, 0x4dd, 0, &mut scroll_after as *mut _ as isize);
    check!(
        first_line > 0
            && foreground(
                ps_edit,
                Range {
                    start: cp,
                    end: cp + 3
                }
            ) == rgb(byteforge_rust::syntax::Kind::Command.color(true)),
        "Scrolling colors newly visible PowerShell lines"
    );
    check!(
        cache_revision == app.documents[app.active].syntax.borrow().cache.revision
            && scroll_before.y == scroll_after.y,
        "Viewport coloring reuses cached syntax and preserves scroll position"
    );
    for _ in 0..8 {
        app.command(NEW);
    }
    let plus_overflow = window_rect(app.new_tab);
    let bar_overflow = window_rect(app.tabs);
    check!(
        plus_overflow.left >= bar_overflow.right
            && plus_overflow.right <= window_rect(app.hwnd).right,
        "Plus remains reachable outside the overflow controls with many tabs"
    );
    while app.documents.len() > 4 {
        app.documents.last_mut().unwrap().doc.dirty = false;
        app.close_tab();
    }
    app.activate(3);
    // Keep the original three-tab restart fixture, with independent contexts.
    app.documents[3].doc.dirty = false;
    app.close_tab();
    app.activate(1);
    app.command(CONTEXT_BASE + Context::PowerShell as u16);
    app.documents[0].doc.context = Context::Json;
    app.disable_recovery()?;
    check!(
        app.dir.join("recovery.off").exists() && !app.dir.join("session.bfs").exists(),
        "Disabling recovery clears snapshots and persists preference"
    );
    fs::remove_file(app.dir.join("recovery.off")).map_err(|e| e.to_string())?;
    app.last_snapshot.clear();
    app.snapshot()?;
    let _ = fs::write(output.join("checks.txt"), checks.join("\n"));
    Ok(checks.len())
}
