use super::*;
thread_local! {pub static OUTPUT:RefCell<Option<PathBuf>>=const{RefCell::new(None)};}
thread_local! {static RECEIVED:std::cell::Cell<u16>=const{std::cell::Cell::new(0)};}
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
    Ok(9)
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
