//! Native controls keep per-tab undo buffers. Rust owns document/session lifetime.
mod native;
mod platform;
mod qa;
use byteforge_rust::{
    document::{self, Document, Highlight, HIGHLIGHTS, MAX_TABS, TAB_COLORS},
    session::{self, Session},
    storage,
};
use native::*;
use std::{
    cell::RefCell,
    fs,
    mem::{size_of, zeroed},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::mpsc,
};

const NEW: u16 = 201;
const OPEN: u16 = 202;
const SAVE: u16 = 203;
const SAVE_AS: u16 = 204;
const CLOSE: u16 = 205;
const SAVE_ALL: u16 = 206;
const INFO: u16 = 207;
const EXIT: u16 = 208;
const UNDO: u16 = 210;
const REDO: u16 = 211;
const CUT: u16 = 212;
const COPY: u16 = 213;
const PASTE: u16 = 214;
const SELECT_ALL: u16 = 215;
const FIND: u16 = 220;
const FIND_ALL: u16 = 221;
const FIND_NEXT: u16 = 222;
const FIND_MORE: u16 = 223;
const REPLACE: u16 = 224;
const JUMP: u16 = 225;
const CLEAR: u16 = 230;
const HL_RED: u16 = 231;
const HL_ORANGE: u16 = 232;
const HL_YELLOW: u16 = 233;
const HL_GREEN: u16 = 234;
const DARK_MODE: u16 = 240;
const WRAP: u16 = 241;
const ZOOM_IN: u16 = 242;
const ZOOM_OUT: u16 = 243;
const ZOOM_RESET: u16 = 244;
const FONT_DEFAULT: u16 = 250;
const FONT_SEGOE: u16 = 251;
const FONT_GEORGIA: u16 = 252;
const FONT_CONSOLAS: u16 = 253;
const FONT_CUSTOM: u16 = 254;
const COLOR_BASE: u16 = 260;
const HEX_LEFT: u16 = 270;
const HEX_RIGHT: u16 = 271;
const HEX_BELOW: u16 = 272;
const HEX_CLOSE: u16 = 273;
const HASH: u16 = 280;
const RECOVERY: u16 = 281;
const ERASE_RECOVERY: u16 = 282;
const UPDATE: u16 = 283;
const ABOUT: u16 = 284;
const NEXT_TAB: u16 = 290;
const PREV_TAB: u16 = 291;
const MOVE_LEFT: u16 = 292;
const MOVE_RIGHT: u16 = 293;
const DO_SEARCH: u16 = 410;
const DO_NEXT: u16 = 411;
const HIDE_SEARCH: u16 = 412;
const DO_REPLACE: u16 = 413;
const DO_REPLACE_ALL: u16 = 414;
const INIT: u32 = WM_APP + 1;
const RUN_QA: u32 = WM_APP + 2;
const ACTIVE_TAB: u32 = WM_APP + 3;
const REFRESH_HEX: u32 = WM_APP + 4;
fn tab_stripe_height(active: bool) -> i32 {
    if active {
        8
    } else {
        6
    }
}
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
thread_local! {static APP:RefCell<Option<App>>=const{RefCell::new(None)};}
struct Tab {
    doc: Document,
    edit: HWND,
    font: HFONT,
    changed: bool,
}
#[derive(Clone, Debug)]
struct Hit {
    tab: usize,
    start: i32,
    end: i32,
    line: i32,
    preview: String,
}
struct App {
    hwnd: HWND,
    tabs: HWND,
    status: HWND,
    hex: HWND,
    hex_label: HWND,
    editor_label: HWND,
    hex_font: HFONT,
    hex_rendered: RefCell<String>,
    hex_refresh_queued: bool,
    search: HWND,
    query: HWND,
    replace: HWND,
    scope: HWND,
    case: HWND,
    results: HWND,
    search_controls: Vec<HWND>,
    documents: Vec<Tab>,
    active: usize,
    menus: Menus,
    dpi: u32,
    search_visible: bool,
    hits: Vec<Hit>,
    hit_index: usize,
    dir: PathBuf,
    recovery: bool,
    allow_snapshot: bool,
    last_snapshot: Vec<u8>,
    last_error: String,
    note: String,
    update: Option<mpsc::Receiver<Result<String, String>>>,
    _lock: fs::File,
    next_control: u16,
}
pub fn run() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        let option = |name: &str| {
            args.iter()
                .position(|a| a == name)
                .and_then(|i| args.get(i + 1))
                .map(PathBuf::from)
        };
        let qa = option("--ui-self-test").or_else(|| option("--verify-recovery"));
        let dir = option("--session-dir").unwrap_or_else(|| {
            qa.as_ref()
                .map(|p| p.join("session"))
                .unwrap_or_else(session::directory)
        });
        if let Err(e) = fs::create_dir_all(&dir) {
            MessageBoxW(
                null_mut(),
                wide(&e.to_string()).as_ptr(),
                wide("ByteForge Rust").as_ptr(),
                MB_OK | MB_ICONERROR,
            );
            return;
        }
        let lock = match fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .open(dir.join("session.lock"))
        {
            Ok(lock) => lock,
            Err(_) => {
                MessageBoxW(null_mut(),wide("This session is already open in another ByteForge Rust window. Use a different --session-dir for an independent workspace.").as_ptr(),wide("ByteForge Rust").as_ptr(),MB_OK);
                return;
            }
        };
        LoadLibraryW(wide("Msftedit.dll").as_ptr());
        InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_TAB_CLASSES | ICC_STANDARD_CLASSES,
        });
        brushes();
        set_ui_font(96);
        let class = wide("ByteForgeRustWindow");
        let instance = GetModuleHandleW(null());
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hIcon: LoadIconW(instance, 1usize as _),
            hIconSm: LoadIconW(instance, 1usize as _),
            lpszClassName: class.as_ptr(),
            ..zeroed()
        };
        RegisterClassExW(&wc);
        let hwnd = CreateWindowExW(
            WS_EX_ACCEPTFILES,
            class.as_ptr(),
            wide("ByteForge Rust 1.1.2").as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1180,
            800,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return;
        }
        set_ui_font(GetDpiForWindow(hwnd).max(96));
        let mut app = App::new(hwnd, dir, lock);
        let mut restore = Session::default();
        match session::load(&app.dir) {
            Ok(Some((s, backup))) => {
                restore = s;
                if backup {
                    app.note = "Recovered previous snapshot; latest snapshot was corrupt.".into();
                }
            }
            Ok(None) => {}
            Err(e) => {
                app.allow_snapshot = false;
                error(hwnd,&format!("{e}\n\nAutomatic recovery writes are paused. Use Clear recovery data to start a new session."));
            }
        }
        app.recovery = restore.enabled;
        if app.dir.join("recovery.off").exists() {
            app.recovery = false;
        }
        DARK.set(restore.dark);
        if args.iter().any(|a| a == "--light") {
            DARK.set(false);
        }
        brushes();
        for doc in restore.documents {
            app.add_document(doc);
        }
        app.active = restore.active.min(app.documents.len() - 1);
        app.activate(app.active);
        app.apply_theme();
        app.layout();
        APP.with(|a| *a.borrow_mut() = Some(app));
        if !args.iter().any(|a| a == "--hidden") {
            ShowWindow(hwnd, SW_SHOW);
            APP.with(|a| {
                if let Some(app) = a.borrow().as_ref() {
                    SetFocus(app.edit());
                }
            });
        }
        UpdateWindow(hwnd);
        windows_sys::Win32::UI::Shell::DragAcceptFiles(hwnd, 1);
        let accel = accelerators();
        let table = CreateAcceleratorTableW(accel.as_ptr(), accel.len() as i32);
        if let Some(dir) = qa {
            qa::OUTPUT.with(|o| *o.borrow_mut() = Some(dir));
            PostMessageW(hwnd, RUN_QA, 0, 0);
        } else {
            let mut skip = false;
            for arg in args {
                if skip {
                    skip = false;
                    continue;
                }
                if arg == "--session-dir" {
                    skip = true;
                    continue;
                }
                if !arg.to_string_lossy().starts_with("--") {
                    let path = PathBuf::from(arg);
                    APP.with(|a| {
                        if let Some(app) = a.borrow_mut().as_mut() {
                            app.open(&path);
                        }
                    });
                }
            }
            PostMessageW(hwnd, INIT, 0, 0);
        }
        SetTimer(hwnd, 1, 1500, None);
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            let search_enter = msg.message == WM_KEYDOWN
                && msg.wParam == VK_RETURN as usize
                && APP.with(|a| {
                    a.borrow()
                        .as_ref()
                        .map(|a| {
                            msg.hwnd == a.query || msg.hwnd == a.results || msg.hwnd == a.replace
                        })
                        .unwrap_or(false)
                });
            if search_enter {
                APP.with(|cell| {
                    if let Some(a) = cell.borrow_mut().as_mut() {
                        if msg.hwnd == a.results {
                            let index = SendMessageW(a.results, LB_GETCURSEL, 0, 0);
                            if index >= 0 {
                                a.select_hit(index as usize);
                            }
                        } else {
                            a.search_all();
                        }
                    }
                });
                continue;
            }
            // Editing the find/replace field must retain native Ctrl+A/C/X/V/Z behavior.
            let local_edit = APP.with(|cell| {
                cell.borrow()
                    .as_ref()
                    .map(|a| msg.hwnd == a.query || msg.hwnd == a.replace)
                    .unwrap_or(false)
            });
            let edit_key = local_edit
                && GetKeyState(VK_CONTROL as i32) < 0
                && GetKeyState(VK_SHIFT as i32) >= 0
                && GetKeyState(VK_MENU as i32) >= 0
                && b"ACXVZY".iter().any(|k| msg.wParam == *k as usize);
            if edit_key {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                continue;
            }
            if TranslateAcceleratorW(hwnd, table, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        DestroyAcceleratorTable(table);
        APP.with(|a| {
            if let Some(mut app) = a.borrow_mut().take() {
                for tab in app.documents.drain(..) {
                    DeleteObject(tab.font);
                }
                DeleteObject(app.hex_font);
                DestroyMenu(app.menus.root);
            }
        });
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: usize, lp: isize) -> isize {
    if let Some(r) = paint_control(msg, wp, lp) {
        return r;
    }
    match msg {
        WM_MEASUREITEM => return measure_menu(lp),
        WM_DRAWITEM => {
            let d = &*(lp as *const DRAWITEMSTRUCT);
            if d.CtlType == ODT_MENU {
                return draw_menu(lp);
            }
            if d.CtlType == ODT_BUTTON {
                return draw_button(d);
            }
        }
        WM_MENUCHAR => return menu_char(wp, lp),
        WM_NCPAINT | WM_NCACTIVATE => {
            let r = DefWindowProcW(hwnd, msg, wp, lp);
            paint_menu_border(hwnd, null_mut());
            return r;
        }
        WM_ERASEBKGND => {
            let mut r = zeroed();
            GetClientRect(hwnd, &mut r);
            FillRect(wp as _, &r, brush(0));
            return 1;
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            return 0;
        }
        _ => {}
    }
    let handled = APP.with(|cell| {
        let Ok(mut guard) = cell.try_borrow_mut() else {
            return None;
        };
        let app = guard.as_mut()?;
        match msg {
            WM_SETFOCUS => {
                SetFocus(app.edit());
                Some(0)
            }
            WM_SIZE => {
                app.layout();
                Some(0)
            }
            WM_DPICHANGED => {
                app.dpi = (wp & 65535) as u32;
                set_ui_font(app.dpi);
                let r = &*(lp as *const RECT);
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                for i in 0..app.documents.len() {
                    app.apply_font(i);
                }
                let old = app.hex_font;
                app.hex_font = font("Consolas", 9, app.dpi);
                SendMessageW(app.hex, WM_SETFONT, app.hex_font as usize, 0);
                DeleteObject(old);
                app.apply_theme();
                app.layout();
                Some(0)
            }
            WM_DRAWITEM => {
                app.draw_tab(&*(lp as *const DRAWITEMSTRUCT));
                Some(1)
            }
            WM_NOTIFY => {
                let n = &*(lp as *const NMHDR);
                if n.hwndFrom == app.tabs && n.code == TCN_SELCHANGE {
                    let index = SendMessageW(app.tabs, TCM_GETCURSEL, 0, 0);
                    if index >= 0 {
                        app.activate(index as usize);
                    }
                }
                Some(0)
            }
            WM_COMMAND => {
                let id = (wp & 65535) as u16;
                let notification = (wp >> 16) as u16;
                if notification == EN_CHANGE as u16 {
                    if id == 420 || id == 422 {
                        app.invalidate_results();
                    }
                    app.editor_changed(lp as HWND);
                } else if id == 421 && notification == LBN_DBLCLK as u16 {
                    let n = SendMessageW(app.results, LB_GETCURSEL, 0, 0);
                    if n >= 0 {
                        app.select_hit(n as usize);
                    }
                } else if notification == 0 || notification == 1 {
                    app.command(id);
                }
                Some(0)
            }
            WM_TIMER => {
                app.tick();
                Some(0)
            }
            REFRESH_HEX => {
                app.flush_hex_refresh();
                Some(0)
            }
            WM_CLOSE => {
                if app.close_app() {
                    DestroyWindow(hwnd);
                }
                Some(0)
            }
            WM_QUERYENDSESSION => Some(app.snapshot().is_ok() as isize),
            WM_ENDSESSION => {
                if wp != 0 {
                    let _ = app.snapshot();
                }
                Some(0)
            }
            WM_INITMENUPOPUP => {
                app.menu_state();
                Some(0)
            }
            WM_DROPFILES => {
                let h = wp as _;
                let n = windows_sys::Win32::UI::Shell::DragQueryFileW(h, u32::MAX, null_mut(), 0);
                for i in 0..n {
                    let len = windows_sys::Win32::UI::Shell::DragQueryFileW(h, i, null_mut(), 0);
                    let mut b = vec![0; len as usize + 1];
                    windows_sys::Win32::UI::Shell::DragQueryFileW(
                        h,
                        i,
                        b.as_mut_ptr(),
                        b.len() as u32,
                    );
                    use std::os::windows::ffi::OsStringExt;
                    app.open(&PathBuf::from(std::ffi::OsString::from_wide(
                        &b[..len as usize],
                    )));
                }
                windows_sys::Win32::UI::Shell::DragFinish(h);
                Some(0)
            }
            WM_CONTEXTMENU => {
                app.context_menu(lp);
                Some(0)
            }
            INIT => {
                app.update_title();
                Some(0)
            }
            ACTIVE_TAB => {
                let i = SendMessageW(app.tabs, TCM_GETCURSEL, 0, 0);
                if i >= 0 {
                    app.activate(i as usize);
                }
                Some(0)
            }
            RUN_QA => {
                qa::run(app);
                Some(0)
            }
            _ => None,
        }
    });
    handled.unwrap_or_else(|| DefWindowProcW(hwnd, msg, wp, lp))
}

pub unsafe fn draw_button(d: &DRAWITEMSTRUCT) -> isize {
    let p = palette();
    FillRect(
        d.hDC,
        &d.rcItem,
        brush(if d.itemState & ODS_SELECTED != 0 {
            3
        } else {
            2
        }),
    );
    let pen = CreateSolidBrush(if d.itemState & ODS_FOCUS != 0 {
        ACCENT
    } else {
        p.border
    });
    FrameRect(d.hDC, &d.rcItem, pen);
    DeleteObject(pen);
    SetBkMode(d.hDC, TRANSPARENT as i32);
    SetTextColor(d.hDC, p.text);
    let old = SelectObject(d.hDC, ui_font());
    let mut r = d.rcItem;
    DrawTextW(
        d.hDC,
        wide(&text(d.hwndItem)).as_ptr(),
        -1,
        &mut r,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
    SelectObject(d.hDC, old);
    1
}

/// Keep the native TabControl's accessibility, keyboard navigation and hit testing,
/// but paint its entire surface so system-white borders do not leak into dark mode.
unsafe extern "system" fn tab_proc(
    hwnd: HWND,
    msg: u32,
    wp: usize,
    lp: isize,
    _id: usize,
    _data: usize,
) -> isize {
    if msg == WM_ERASEBKGND {
        return 1;
    }
    if msg == WM_PAINT {
        let mut ps: PAINTSTRUCT = zeroed();
        let dc = BeginPaint(hwnd, &mut ps);
        let mut rect: RECT = zeroed();
        GetClientRect(hwnd, &mut rect);
        FillRect(dc, &rect, brush(1));
        let old = SelectObject(dc, ui_font());
        SetBkMode(dc, TRANSPARENT as i32);
        let active = SendMessageW(hwnd, TCM_GETCURSEL, 0, 0);
        let dpi = GetDpiForWindow(hwnd).max(96) as i32;
        let s = |n| n * dpi / 96;
        for i in 0..SendMessageW(hwnd, TCM_GETITEMCOUNT, 0, 0).max(0) {
            let mut r: RECT = zeroed();
            SendMessageW(hwnd, TCM_GETITEMRECT, i as usize, &mut r as *mut _ as isize);
            let mut label = [0u16; 512];
            let mut item = TCITEMW {
                mask: TCIF_TEXT | TCIF_PARAM,
                pszText: label.as_mut_ptr(),
                cchTextMax: 512,
                ..zeroed()
            };
            SendMessageW(hwnd, TCM_GETITEMW, i as usize, &mut item as *mut _ as isize);
            let selected = i == active;
            FillRect(dc, &r, brush(if selected { 3 } else { 1 }));
            let border = CreateSolidBrush(palette().border);
            FrameRect(dc, &r, border);
            DeleteObject(border);
            let color = TAB_COLORS
                .get(item.lParam as usize)
                .unwrap_or(&TAB_COLORS[0])
                .1;
            let b = CreateSolidBrush(rgb(color));
            let stripe = RECT {
                left: r.left + 1,
                top: r.top + 1,
                right: r.right - 1,
                bottom: r.top + s(tab_stripe_height(selected)),
            };
            FillRect(dc, &stripe, b);
            DeleteObject(b);
            r.left += s(12);
            r.right -= s(8);
            SetTextColor(dc, palette().text);
            DrawTextW(
                dc,
                label.as_ptr(),
                -1,
                &mut r,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
            );
        }
        SelectObject(dc, old);
        EndPaint(hwnd, &ps);
        return 0;
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
unsafe extern "system" fn check_proc(
    hwnd: HWND,
    msg: u32,
    wp: usize,
    lp: isize,
    _id: usize,
    _data: usize,
) -> isize {
    if msg == WM_ERASEBKGND {
        return 1;
    }
    if msg == WM_PAINT {
        let mut ps: PAINTSTRUCT = zeroed();
        let dc = BeginPaint(hwnd, &mut ps);
        let mut r: RECT = zeroed();
        GetClientRect(hwnd, &mut r);
        FillRect(dc, &r, brush(0));
        let dpi = GetDpiForWindow(hwnd).max(96) as i32;
        let s = |n| n * dpi / 96;
        let top = (r.bottom - s(14)) / 2;
        let square = RECT {
            left: s(2),
            top,
            right: s(16),
            bottom: top + s(14),
        };
        let checked = SendMessageW(hwnd, BM_GETCHECK, 0, 0) != 0;
        let border = CreateSolidBrush(if GetFocus() == hwnd {
            ACCENT
        } else {
            palette().muted
        });
        FrameRect(dc, &square, border);
        DeleteObject(border);
        let old = SelectObject(dc, ui_font());
        SetBkMode(dc, TRANSPARENT as i32);
        if checked {
            let mut q = square;
            SetTextColor(dc, ACCENT);
            DrawTextW(
                dc,
                wide("✓").as_ptr(),
                -1,
                &mut q,
                DT_SINGLELINE | DT_CENTER | DT_VCENTER,
            );
        }
        SetTextColor(dc, palette().text);
        r.left = s(22);
        DrawTextW(
            dc,
            wide(&text(hwnd)).as_ptr(),
            -1,
            &mut r,
            DT_SINGLELINE | DT_VCENTER,
        );
        SelectObject(dc, old);
        EndPaint(hwnd, &ps);
        return 0;
    }
    let result = DefSubclassProc(hwnd, msg, wp, lp);
    if matches!(
        msg,
        BM_SETCHECK | WM_LBUTTONUP | WM_KEYUP | WM_SETFOCUS | WM_KILLFOCUS
    ) {
        InvalidateRect(hwnd, null(), 1);
    }
    result
}

impl App {
    unsafe fn new(hwnd: HWND, dir: PathBuf, lock: fs::File) -> Self {
        let menus = make_menus();
        SetMenu(hwnd, menus.root);
        let tabs = child(
            hwnd,
            "SysTabControl32",
            "Open documents",
            400,
            TCS_FIXEDWIDTH | TCS_BUTTONS | TCS_FLATBUTTONS | TCS_FOCUSONBUTTONDOWN | WS_TABSTOP,
        );
        SetWindowSubclass(tabs, Some(tab_proc), 1, 0);
        SendMessageW(tabs, TCM_SETITEMSIZE, 0, ((30 << 16) | 180) as isize);
        let status = child(hwnd, "STATIC", "Ready", 401, SS_LEFT);
        let hex = child(
            hwnd,
            "RICHEDIT50W",
            "",
            402,
            ES_MULTILINE as u32 | ES_READONLY as u32 | WS_VSCROLL | WS_HSCROLL,
        );
        let hex_font = font("Consolas", 9, GetDpiForWindow(hwnd).max(96));
        SendMessageW(hex, WM_SETFONT, hex_font as usize, 0);
        SendMessageW(hex, 0x448, 0, 1); // No wrapping: hex columns must stay aligned.
        let hex_label = child(hwnd, "STATIC", "  HEX — live buffer", 403, SS_LEFT);
        let editor_label = child(hwnd, "STATIC", "  EDITOR", 404, SS_LEFT);
        ShowWindow(hex_label, SW_HIDE);
        ShowWindow(editor_label, SW_HIDE);
        SendMessageW(hex, EM_SETREADONLY, 1, 0);
        let query = child(
            hwnd,
            "EDIT",
            "",
            420,
            WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32,
        );
        let replace = child(
            hwnd,
            "EDIT",
            "",
            422,
            WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32,
        );
        let scope = child(
            hwnd,
            "BUTTON",
            "All tabs",
            423,
            BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        );
        let case = child(
            hwnd,
            "BUTTON",
            "Match case",
            424,
            BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        );
        SetWindowSubclass(scope, Some(check_proc), 1, 0);
        SetWindowSubclass(case, Some(check_proc), 1, 0);
        let search = child(hwnd, "STATIC", "Find:", 425, SS_LEFT);
        let replace_label = child(hwnd, "STATIC", "Replace:", 426, SS_LEFT);
        let mut controls = vec![search, query, replace, replace_label, scope, case];
        for (id, label) in [
            (DO_SEARCH, "Find all"),
            (DO_NEXT, "Next"),
            (HIDE_SEARCH, "Close"),
            (DO_REPLACE, "Replace"),
            (DO_REPLACE_ALL, "Replace all"),
        ] {
            controls.push(child(
                hwnd,
                "BUTTON",
                label,
                id,
                BS_OWNERDRAW as u32 | WS_TABSTOP,
            ));
        }
        let results = child(
            hwnd,
            "LISTBOX",
            "Search results",
            421,
            LBS_NOTIFY as u32
                | LBS_NOINTEGRALHEIGHT as u32
                | WS_VSCROLL
                | WS_HSCROLL
                | WS_TABSTOP
                | WS_BORDER,
        );
        ShowWindow(results, SW_HIDE);
        for h in &controls {
            ShowWindow(*h, SW_HIDE);
        }
        ShowWindow(hex, SW_HIDE);
        Self {
            hwnd,
            tabs,
            status,
            hex,
            hex_label,
            editor_label,
            hex_font,
            hex_rendered: RefCell::new(String::new()),
            hex_refresh_queued: false,
            search,
            query,
            replace,
            scope,
            case,
            results,
            search_controls: controls,
            documents: vec![],
            active: 0,
            menus,
            dpi: GetDpiForWindow(hwnd).max(96),
            search_visible: false,
            hits: vec![],
            hit_index: 0,
            dir,
            recovery: true,
            allow_snapshot: true,
            last_snapshot: vec![],
            last_error: String::new(),
            note: String::new(),
            update: None,
            _lock: lock,
            next_control: 1000,
        }
    }
    fn scale(&self, n: i32) -> i32 {
        n * self.dpi as i32 / 96
    }
    fn edit(&self) -> HWND {
        self.documents[self.active].edit
    }
    unsafe fn add_document(&mut self, mut doc: Document) {
        if self.documents.len() >= MAX_TABS {
            error(self.hwnd, "At most 128 tabs may be open.");
            return;
        }
        if doc.path.is_none() && doc.title == "Untitled" {
            let mut n = 1;
            loop {
                let candidate = format!("Untitled {n}");
                if self.documents.iter().all(|t| t.doc.title != candidate) {
                    doc.title = candidate;
                    break;
                }
                n += 1;
            }
        }
        let edit = child(
            self.hwnd,
            "RICHEDIT50W",
            "",
            self.next_control,
            WS_TABSTOP
                | WS_VSCROLL
                | WS_HSCROLL
                | ES_MULTILINE as u32
                | ES_AUTOVSCROLL as u32
                | ES_AUTOHSCROLL as u32
                | ES_NOHIDESEL as u32,
        );
        self.next_control += 1;
        ShowWindow(edit, SW_HIDE);
        // RichEdit starts with a system-white background; apply the current mode
        // before the editor is populated or activated (also covers Open/restore).
        SendMessageW(edit, 0x443, 0, palette().control as isize);
        SendMessageW(edit, 0x435, 0, document::MAX_DOCUMENT as isize);
        SendMessageW(edit, 0x445, 0, 1); // EM_SETEVENTMASK: ENM_CHANGE
        set_raw(edit, &doc.text);
        for h in &doc.highlights {
            set_range(
                edit,
                Range {
                    start: h.start,
                    end: h.end,
                },
            );
            format(edit, Some(rgb(h.color)));
        }
        set_range(
            edit,
            Range {
                start: doc.selection.0,
                end: doc.selection.1,
            },
        );
        SendMessageW(edit, EM_EMPTYUNDOBUFFER, 0, 0);
        let label = wide(&doc.label());
        let item = TCITEMW {
            mask: TCIF_TEXT | TCIF_PARAM,
            pszText: label.as_ptr() as _,
            lParam: doc.color as isize,
            ..zeroed()
        };
        SendMessageW(
            self.tabs,
            TCM_INSERTITEMW,
            self.documents.len(),
            &item as *const _ as isize,
        );
        self.documents.push(Tab {
            doc,
            edit,
            font: null_mut(),
            changed: false,
        });
        self.invalidate_results();
        self.apply_font(self.documents.len() - 1);
        self.set_wrap(self.documents.len() - 1);
        let scroll = self.documents.last().unwrap().doc.scroll;
        let pt = POINT {
            x: scroll.0,
            y: scroll.1,
        };
        SendMessageW(edit, 0x4de, 0, &pt as *const _ as isize);
    }
    unsafe fn activate(&mut self, index: usize) {
        if index >= self.documents.len() {
            return;
        }
        self.capture(self.active);
        ShowWindow(self.edit(), SW_HIDE);
        self.active = index;
        SendMessageW(self.tabs, TCM_SETCURSEL, index, 0);
        ShowWindow(self.edit(), SW_SHOW);
        SetFocus(self.edit());
        self.refresh_hex();
        self.layout();
        self.update_title();
        InvalidateRect(self.tabs, null(), 1);
    }
    unsafe fn capture(&mut self, index: usize) {
        let mut edited = false;
        let Some(tab) = self.documents.get_mut(index) else {
            return;
        };
        if tab.changed {
            let current = document::normalize(&raw_text(tab.edit));
            if current != tab.doc.text {
                tab.doc.text = current;
                tab.doc.dirty = true;
                edited = true;
            }
            tab.changed = false;
        }
        let r = range(tab.edit);
        tab.doc.selection = (r.start, r.end);
        let mut pt: POINT = zeroed();
        SendMessageW(tab.edit, 0x4dd, 0, &mut pt as *mut _ as isize);
        tab.doc.scroll = (pt.x, pt.y);
        if edited {
            self.invalidate_results();
            if index == self.active {
                self.refresh_hex();
            }
        }
    }
    unsafe fn editor_changed(&mut self, hwnd: HWND) {
        if let Some(tab) = self.documents.iter_mut().find(|t| t.edit == hwnd) {
            tab.changed = true;
        }
        if hwnd == self.edit()
            && self.documents[self.active].doc.hex != 0
            && !self.hex_refresh_queued
        {
            self.hex_refresh_queued = true;
            PostMessageW(self.hwnd, REFRESH_HEX, 0, 0);
        }
    }
    unsafe fn flush_hex_refresh(&mut self) {
        self.hex_refresh_queued = false;
        self.capture(self.active);
        self.update_title();
    }
    unsafe fn capture_highlights(&mut self, index: usize) {
        let tab = &mut self.documents[index];
        let visible = GetWindowLongPtrW(tab.edit, GWL_STYLE) & WS_VISIBLE as isize != 0;
        let selected = range(tab.edit);
        let mut pt: POINT = zeroed();
        SendMessageW(tab.edit, 0x4dd, 0, &mut pt as *mut _ as isize);
        let end = tab.doc.text.encode_utf16().count() as i32;
        let mut stack = vec![Range { start: 0, end }];
        let mut spans: Vec<Highlight> = vec![];
        SendMessageW(tab.edit, WM_SETREDRAW, 0, 0);
        while let Some(r) = stack.pop() {
            if r.start >= r.end {
                continue;
            }
            set_range(tab.edit, r);
            let mut cf: CharFormat = zeroed();
            cf.size = size_of::<CharFormat>() as u32;
            SendMessageW(tab.edit, 0x43a, 1, &mut cf as *mut _ as isize);
            if cf.mask & 0x04000000 == 0 && r.end - r.start > 1 {
                let mid = r.start + (r.end - r.start) / 2;
                stack.push(Range {
                    start: mid,
                    end: r.end,
                });
                stack.push(Range {
                    start: r.start,
                    end: mid,
                });
            } else if cf.effects & 0x04000000 == 0 {
                let color = logical_highlight(cf.back);
                if let Some(last) = spans.last_mut() {
                    if last.end == r.start && last.color == color {
                        last.end = r.end;
                        continue;
                    }
                }
                spans.push(Highlight {
                    start: r.start,
                    end: r.end,
                    color,
                });
            }
        }
        set_range(tab.edit, selected);
        SendMessageW(tab.edit, 0x4de, 0, &pt as *const _ as isize);
        SendMessageW(tab.edit, WM_SETREDRAW, 1, 0);
        if !visible {
            ShowWindow(tab.edit, SW_HIDE);
        }
        tab.doc.highlights = spans;
    }
    unsafe fn snapshot(&mut self) -> Result<(), String> {
        if !self.allow_snapshot {
            return Err(
                "Recovery writes paused after a corrupt session; clear recovery data to resume."
                    .into(),
            );
        }
        if !self.recovery {
            return Ok(());
        }
        for i in 0..self.documents.len() {
            self.capture(i);
            self.capture_highlights(i);
        }
        let s = Session {
            documents: self.documents.iter().map(|t| t.doc.clone()).collect(),
            active: self.active,
            dark: DARK.get(),
            enabled: self.recovery,
        };
        let bytes = session::encode(&s)?;
        if bytes != self.last_snapshot {
            session::save(&self.dir, &s)?;
            self.last_snapshot = bytes;
        }
        Ok(())
    }
    unsafe fn tick(&mut self) {
        self.capture(self.active);
        self.update_title();
        if let Err(e) = self.snapshot() {
            self.note = format!("Recovery failed: {e}");
            if self.last_error != e {
                self.last_error = e.clone();
                error(
                    self.hwnd,
                    &format!(
                        "Automatic session recovery failed. Your documents are still open.\n\n{e}"
                    ),
                );
            }
        }
        if let Some(receiver) = self.update.as_ref() {
            if let Ok(result) = receiver.try_recv() {
                self.update = None;
                match result {
                    Ok(v) => {
                        prompt(self.hwnd,"Upstream version",&format!("Rust edition: 1.1.2\nFASM upstream: {v}\n\nThis is the original FASM feed, not an update channel for this Rust edition.\nhttps://github.com/Jaap79/ByteForge"),None,&["OK"]);
                    }
                    Err(e) => error(self.hwnd, &e),
                }
            }
        }
    }
    unsafe fn update_title(&mut self) {
        let tab = &self.documents[self.active];
        let path = tab
            .doc
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| tab.doc.title.clone());
        set_text(
            self.hwnd,
            &format!(
                "{}{} — ByteForge Rust 1.1.2",
                if tab.doc.dirty { "* " } else { "" },
                path
            ),
        );
        for (i, t) in self.documents.iter().enumerate() {
            let w = wide(&t.doc.label());
            let item = TCITEMW {
                mask: TCIF_TEXT | TCIF_PARAM,
                pszText: w.as_ptr() as _,
                lParam: t.doc.color as isize,
                ..zeroed()
            };
            SendMessageW(self.tabs, TCM_SETITEMW, i, &item as *const _ as isize);
        }
        let r = range(tab.edit);
        let (line, column) = document::line_column(&tab.doc.text, r.start);
        set_text(
            self.status,
            &format!(
                "  Tab {}/{}  |  Ln {}, Col {}  |  {:?} / {:?}  |  {} {}pt / {}%  |  Recovery {}{}",
                self.active + 1,
                self.documents.len(),
                line,
                column,
                tab.doc.encoding,
                tab.doc.eol,
                tab.doc.font,
                tab.doc.points,
                tab.doc.zoom,
                if self.recovery { "ON" } else { "OFF" },
                if self.note.is_empty() {
                    String::new()
                } else {
                    format!("  |  {}", self.note)
                }
            ),
        );
        self.menu_state();
    }
    unsafe fn layout(&self) {
        let mut r: RECT = zeroed();
        GetClientRect(self.hwnd, &mut r);
        let width = r.right.max(0);
        let height = r.bottom.max(0);
        let s = |n| self.scale(n);
        MoveWindow(self.tabs, 0, 0, width, s(36), 1);
        SendMessageW(
            self.tabs,
            TCM_SETITEMSIZE,
            0,
            ((s(30) << 16) | s(180)) as isize,
        );
        MoveWindow(self.status, 0, (height - s(26)).max(0), width, s(26), 1);
        let top = if self.search_visible { s(112) } else { s(36) };
        if self.search_visible {
            let positions = [
                (self.search, 8, 44, 42, 24),
                (self.query, 54, 40, 300, 27),
                (self.scope, 365, 40, 90, 27),
                (self.case, 462, 40, 110, 27),
                (GetDlgItem(self.hwnd, DO_SEARCH as i32), 580, 40, 88, 27),
                (GetDlgItem(self.hwnd, DO_NEXT as i32), 676, 40, 70, 27),
                (GetDlgItem(self.hwnd, HIDE_SEARCH as i32), 754, 40, 70, 27),
                (GetDlgItem(self.hwnd, 426), 8, 80, 60, 24),
                (self.replace, 74, 76, 280, 27),
                (GetDlgItem(self.hwnd, DO_REPLACE as i32), 365, 76, 90, 27),
                (
                    GetDlgItem(self.hwnd, DO_REPLACE_ALL as i32),
                    462,
                    76,
                    110,
                    27,
                ),
            ];
            for (h, x, y, w, ht) in positions {
                MoveWindow(h, s(x), s(y), s(w).min((width - s(x) - 4).max(0)), s(ht), 1);
            }
        }
        let available = (height - s(26) - top).max(0);
        let result_h = if self.search_visible && !self.hits.is_empty() {
            s(150).min(available / 3)
        } else {
            0
        };
        ShowWindow(self.results, if result_h > 0 { SW_SHOW } else { SW_HIDE });
        let bottom = (height - s(26) - result_h).max(top);
        if result_h > 0 {
            MoveWindow(self.results, 0, bottom, width, result_h, 1);
        }
        let h = (bottom - top).max(0);
        let mode = self.documents[self.active].doc.hex;
        for label in [self.hex_label, self.editor_label] {
            ShowWindow(label, if mode == 0 { SW_HIDE } else { SW_SHOW });
        }
        if mode == 0 {
            ShowWindow(self.hex, SW_HIDE);
            MoveWindow(self.edit(), 0, top, width, h, 1);
            return;
        }
        let gap = s(8).min(h).min(width);
        let header = s(24);
        let (ex, ey, ew, eh, hx, hy, hw, hh) = if mode == 3 {
            let hex_h = (h - gap) * 2 / 5;
            (
                0,
                top,
                width,
                h - gap - hex_h,
                0,
                top + h - hex_h,
                width,
                hex_h,
            )
        } else {
            let left = (width - gap) / 2;
            let right = width - gap - left;
            if mode == 1 {
                (left + gap, top, right, h, 0, top, left, h)
            } else {
                (0, top, left, h, left + gap, top, right, h)
            }
        };
        set_text(
            self.editor_label,
            &format!("  EDITOR — {}", self.documents[self.active].doc.title),
        );
        MoveWindow(self.editor_label, ex, ey, ew, header.min(eh), 1);
        MoveWindow(self.hex_label, hx, hy, hw, header.min(hh), 1);
        MoveWindow(
            self.edit(),
            ex,
            ey + header.min(eh),
            ew,
            (eh - header).max(0),
            1,
        );
        MoveWindow(
            self.hex,
            hx,
            hy + header.min(hh),
            hw,
            (hh - header).max(0),
            1,
        );
        ShowWindow(self.hex, SW_SHOW);
        self.refresh_hex();
    }
    unsafe fn apply_font(&mut self, i: usize) {
        let t = &mut self.documents[i];
        let new = font(&t.doc.font, t.doc.points, self.dpi);
        SendMessageW(t.edit, WM_SETFONT, new as usize, 0);
        if !t.font.is_null() {
            DeleteObject(t.font);
        }
        t.font = new;
        let mut cf: CharFormat = zeroed();
        cf.size = size_of::<CharFormat>() as u32;
        cf.mask = 0xA0000000;
        cf.height = t.doc.points * 20;
        let face = wide(&t.doc.font);
        for (a, b) in cf.face.iter_mut().zip(face) {
            *a = b;
        }
        SendMessageW(t.edit, 0x444, 4, &cf as *const _ as isize);
        SendMessageW(t.edit, 0x444, 0, &cf as *const _ as isize);
        SendMessageW(t.edit, 0x4e1, t.doc.zoom as usize, 100);
        recolor_raw(t.edit);
        InvalidateRect(t.edit, null(), 1);
    }
    unsafe fn set_wrap(&self, i: usize) {
        let t = &self.documents[i];
        SendMessageW(t.edit, 0x448, 0, if t.doc.wrap { 0 } else { 1 });
    }
    unsafe fn apply_theme(&self) {
        title_theme(self.hwnd);
        self.menus.theme();
        let p = palette();
        for hwnd in [
            self.tabs,
            self.status,
            self.results,
            self.hex_label,
            self.editor_label,
        ]
        .into_iter()
        .chain(self.search_controls.iter().copied())
        {
            SendMessageW(hwnd, WM_SETFONT, ui_font() as usize, 1);
        }
        for t in &self.documents {
            SendMessageW(t.edit, 0x443, 0, p.control as isize);
            recolor_raw(t.edit);
        }
        SendMessageW(self.hex, 0x443, 0, p.control as isize);
        recolor_raw(self.hex);
        DrawMenuBar(self.hwnd);
        InvalidateRect(self.hwnd, null(), 1);
        InvalidateRect(self.tabs, null(), 1);
        RedrawWindow(
            self.hwnd,
            null(),
            null_mut(),
            RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_FRAME,
        );
    }
    unsafe fn draw_tab(&self, d: &DRAWITEMSTRUCT) {
        if d.CtlType != ODT_TAB {
            return;
        }
        let Some(t) = self.documents.get(d.itemID as usize) else {
            return;
        };
        let active = d.itemID as usize == self.active;
        FillRect(d.hDC, &d.rcItem, brush(if active { 3 } else { 1 }));
        let color = CreateSolidBrush(rgb(TAB_COLORS[t.doc.color as usize].1));
        let mut stripe = d.rcItem;
        stripe.bottom = stripe.top + self.scale(tab_stripe_height(active));
        FillRect(d.hDC, &stripe, color);
        DeleteObject(color);
        let mut r = d.rcItem;
        r.left += self.scale(12);
        r.right -= self.scale(8);
        SetBkMode(d.hDC, TRANSPARENT as i32);
        SetTextColor(d.hDC, palette().text);
        let old = SelectObject(d.hDC, ui_font());
        DrawTextW(
            d.hDC,
            wide(&t.doc.label()).as_ptr(),
            -1,
            &mut r,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
        SelectObject(d.hDC, old);
    }

    unsafe fn open(&mut self, path: &Path) {
        let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if let Some(i) = self
            .documents
            .iter()
            .position(|t| t.doc.path.as_ref() == Some(&path))
        {
            self.activate(i);
            return;
        }
        let result: Result<Document, String> = (|| {
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            if meta.len() > document::MAX_DOCUMENT as u64 {
                return Err("File exceeds 32 MiB limit".into());
            }
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let (text, encoding, eol) = document::decode(&bytes)?;
            let hash = storage::hashes(&bytes)?.1;
            Ok(Document {
                title: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                path: Some(path.clone()),
                text,
                encoding,
                eol,
                disk_hash: Some(hash),
                ..Document::default()
            })
        })();
        match result {
            Ok(doc) => {
                self.add_document(doc);
                self.activate(self.documents.len() - 1);
                self.invalidate_results();
            }
            Err(e) => error(self.hwnd, &format!("{}\n\n{e}", path.display())),
        }
    }
    unsafe fn save(&mut self, index: usize, save_as: bool) -> bool {
        self.capture(index);
        let doc = &self.documents[index].doc;
        let target = if save_as || doc.path.is_none() {
            file_dialog(self.hwnd, true, false, doc.path.as_deref())
        } else {
            doc.path.clone()
        };
        let Some(target) = target else {
            return false;
        };
        if self.documents.iter().enumerate().any(|(i, t)| {
            i != index
                && t.doc
                    .path
                    .as_ref()
                    .map(|p| fs::canonicalize(p).unwrap_or_else(|_| p.clone()))
                    == Some(fs::canonicalize(&target).unwrap_or_else(|_| target.clone()))
        }) {
            error(
                self.hwnd,
                "This file is already open in another tab. Save to a different path.",
            );
            return false;
        }
        if doc.path.as_ref() == Some(&target)
            && storage::disk_changed(&target, doc.disk_hash.as_deref())
        {
            let choice=prompt(self.hwnd,"File changed on disk",&format!("{}\n\nThe disk file changed or disappeared after it was opened. Overwrite it with this tab, save a copy, or cancel?",target.display()),None,&["Overwrite","Save copy","Cancel"]);
            match choice.map(|c| c.0) {
                Some(1) => {}
                Some(2) => return self.save(index, true),
                _ => return false,
            }
        }
        let bytes = doc.bytes();
        match storage::atomic_write(&target, &bytes) {
            Ok(()) => {
                let t = &mut self.documents[index];
                t.doc.path = Some(fs::canonicalize(&target).unwrap_or(target.clone()));
                t.doc.title = target
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into();
                t.doc.disk_hash = storage::hashes(&bytes).ok().map(|h| h.1);
                t.doc.dirty = false;
                SendMessageW(t.edit, EM_SETMODIFY, 0, 0);
                self.note = "Saved".into();
                self.update_title();
                true
            }
            Err(e) => {
                error(
                    self.hwnd,
                    &format!("Save failed; existing file retained.\n\n{e}"),
                );
                false
            }
        }
    }
    unsafe fn close_tab(&mut self) -> bool {
        self.capture(self.active);
        if self.documents[self.active].doc.dirty {
            let label = self.documents[self.active].doc.title.clone();
            let answer=prompt(self.hwnd,"Close tab",&format!("Save changes to {label}?\n\nDiscard removes this tab from the recoverable session."),None,&["Save","Discard","Cancel"]);
            match answer.map(|a| a.0) {
                Some(1) => {
                    if !self.save(self.active, false) {
                        return false;
                    }
                }
                Some(2) => {}
                _ => return false,
            }
        }
        let old = self.documents.remove(self.active);
        DestroyWindow(old.edit);
        DeleteObject(old.font);
        SendMessageW(self.tabs, TCM_DELETEITEM, self.active, 0);
        if self.documents.is_empty() {
            self.add_document(Document::default());
        }
        self.active = self.active.min(self.documents.len() - 1);
        self.activate(self.active);
        self.invalidate_results();
        if let Err(e) = self.snapshot() {
            error(self.hwnd, &e);
        }
        true
    }
    unsafe fn close_app(&mut self) -> bool {
        if self.recovery {
            if let Err(e) = self.snapshot() {
                error(self.hwnd,&format!("Session was not saved. Leave ByteForge open and resolve the storage error, or save documents individually.\n\n{e}"));
                return false;
            }
            return true;
        }
        for i in 0..self.documents.len() {
            self.capture(i);
            if self.documents[i].doc.dirty {
                self.activate(i);
                let choice = prompt(
                    self.hwnd,
                    "Unsaved tab",
                    &format!(
                        "Save changes to {} before exit?",
                        self.documents[i].doc.title
                    ),
                    None,
                    &["Save", "Discard", "Cancel"],
                );
                match choice.map(|a| a.0) {
                    Some(1) => {
                        if !self.save(i, false) {
                            return false;
                        }
                    }
                    Some(2) => {}
                    _ => return false,
                }
            }
        }
        true
    }
    unsafe fn show_search(&mut self, all: bool) {
        self.search_visible = true;
        for h in &self.search_controls {
            ShowWindow(*h, SW_SHOW);
        }
        ShowWindow(self.results, SW_SHOW);
        SendMessageW(self.scope, BM_SETCHECK, all as usize, 0);
        self.layout();
        SetFocus(self.query);
        SendMessageW(self.query, EM_SETSEL, 0, -1);
    }
    unsafe fn invalidate_results(&mut self) {
        let had_results = !self.hits.is_empty();
        self.hits.clear();
        self.hit_index = 0;
        SendMessageW(self.results, LB_RESETCONTENT, 0, 0);
        if had_results && !self.documents.is_empty() {
            self.layout();
        }
    }
    unsafe fn search_all(&mut self) {
        for i in 0..self.documents.len() {
            self.capture(i);
        }
        self.invalidate_results();
        let needle = text(self.query);
        if needle.is_empty() {
            return;
        }
        let all = SendMessageW(self.scope, BM_GETCHECK, 0, 0) != 0;
        let case = SendMessageW(self.case, BM_GETCHECK, 0, 0) != 0;
        for i in 0..self.documents.len() {
            if !all && i != self.active {
                continue;
            }
            self.capture(i);
            let t = &self.documents[i];
            let mut pos = 0;
            while let Some(r) = rich_find(t.edit, &needle, pos, case) {
                if r.end <= pos {
                    break;
                }
                let line = document::line_column(&t.doc.text, r.start).0 as i32 - 1;
                let preview = t
                    .doc
                    .text
                    .lines()
                    .nth(line as usize)
                    .unwrap_or("")
                    .trim()
                    .chars()
                    .take(160)
                    .collect::<String>();
                let hit = Hit {
                    tab: i,
                    start: r.start,
                    end: r.end,
                    line: line + 1,
                    preview,
                };
                let label = format!("{}  :{}  {}", t.doc.title, hit.line, hit.preview);
                SendMessageW(
                    self.results,
                    LB_ADDSTRING,
                    0,
                    wide(&label).as_ptr() as isize,
                );
                self.hits.push(hit);
                pos = r.end;
                if self.hits.len() >= 10000 {
                    break;
                }
            }
            if self.hits.len() >= 10000 {
                break;
            }
        }
        self.note = format!(
            "{} match{}{}",
            self.hits.len(),
            if self.hits.len() == 1 { "" } else { "es" },
            if self.hits.len() >= 10000 {
                " (first 10,000 shown)"
            } else {
                ""
            }
        );
        if !self.hits.is_empty() {
            self.select_hit(0);
        }
        self.update_title();
    }
    unsafe fn select_hit(&mut self, index: usize) {
        if self.documents.iter().any(|t| t.changed) {
            for i in 0..self.documents.len() {
                self.capture(i);
            }
            if self.hits.is_empty() {
                self.note = "Text changed; run the search again.".into();
                return;
            }
        }
        let Some(hit) = self.hits.get(index).cloned() else {
            return;
        };
        if hit.tab >= self.documents.len() {
            self.invalidate_results();
            return;
        }
        self.activate(hit.tab);
        set_range(
            self.edit(),
            Range {
                start: hit.start,
                end: hit.end,
            },
        );
        SendMessageW(self.edit(), EM_SCROLLCARET, 0, 0);
        SetFocus(self.edit());
        self.hit_index = index;
        SendMessageW(self.results, LB_SETCURSEL, index, 0);
        self.update_title();
    }
    unsafe fn find_next(&mut self) {
        let stale = self.documents.iter().any(|t| t.changed);
        if stale || self.hits.is_empty() {
            self.search_all();
            return;
        }
        let n = (self.hit_index + 1) % self.hits.len();
        self.select_hit(n);
    }
    unsafe fn replace_hits(&mut self, all: bool) {
        let selected_tab = self.active;
        let selected_range = range(self.edit());
        self.search_all();
        if self.hits.is_empty() {
            return;
        }
        if all && self.hits.len() >= 10000 {
            error(
                self.hwnd,
                "The result limit was reached. Refine the search before replacing all matches.",
            );
            return;
        }
        if !all {
            if let Some(i) = self.hits.iter().position(|h| {
                h.tab == selected_tab
                    && h.start == selected_range.start
                    && h.end == selected_range.end
            }) {
                self.hit_index = i;
            }
        }
        let replacement = text(self.replace);
        let matches = if all {
            self.hits.clone()
        } else {
            vec![self.hits[self.hit_index].clone()]
        };
        if all && matches.len() > 1 {
            let choice=prompt(self.hwnd,"Replace all",&format!("Replace {} matches in the selected search scope? Each tab retains its own undo history.",matches.len()),None,&["Replace all","Cancel"]);
            if !matches!(choice, Some((1, _))) {
                return;
            }
        }
        for hit in matches.iter().rev() {
            let t = &mut self.documents[hit.tab];
            set_range(
                t.edit,
                Range {
                    start: hit.start,
                    end: hit.end,
                },
            );
            SendMessageW(
                t.edit,
                EM_REPLACESEL,
                1,
                wide(&replacement).as_ptr() as isize,
            );
            t.changed = true;
        }
        self.invalidate_results();
        self.capture(self.active);
        self.update_title();
    }
    unsafe fn highlight(&mut self, color: Option<u32>) {
        let r = range(self.edit());
        if r.start == r.end {
            return;
        }
        format(self.edit(), color.map(rgb));
        self.capture_highlights(self.active);
        self.note = if color.is_some() {
            "Selection highlighted"
        } else {
            "Highlight removed"
        }
        .into();
        InvalidateRect(self.edit(), null(), 1);
    }
    unsafe fn refresh_hex(&self) {
        let d = &self.documents[self.active].doc;
        if d.hex == 0 {
            return;
        }
        let mut rect: RECT = zeroed();
        GetClientRect(self.hex, &mut rect);
        let dc = GetDC(self.hex);
        let old = SelectObject(dc, self.hex_font);
        let mut metrics: TEXTMETRICW = zeroed();
        GetTextMetricsW(dc, &mut metrics);
        SelectObject(dc, old);
        ReleaseDC(self.hex, dc);
        let columns = if rect.right / metrics.tmAveCharWidth.max(1) >= 78 {
            16
        } else if rect.right / metrics.tmAveCharWidth.max(1) >= 46 {
            8
        } else {
            4
        };
        let bytes = d.bytes();
        let mut out = format!(
            "Offset    | {:width$}| ASCII\r\n",
            "Hex bytes",
            width = columns * 3
        );
        for (i, chunk) in bytes[..bytes.len().min(65536)].chunks(columns).enumerate() {
            out.push_str(&format!("{:08X}  | ", i * columns));
            for n in 0..columns {
                if let Some(b) = chunk.get(n) {
                    out.push_str(&format!("{b:02X} "));
                } else {
                    out.push_str("   ");
                }
            }
            out.push_str("| ");
            for b in chunk {
                out.push(if (32..127).contains(b) {
                    *b as char
                } else {
                    '.'
                });
            }
            out.push_str("\r\n");
        }
        if bytes.len() > 65536 {
            out.push_str("Preview limited to 64 KiB.");
        }
        if *self.hex_rendered.borrow() == out {
            return;
        }
        let selected = range(self.hex);
        let mut scroll: POINT = zeroed();
        SendMessageW(self.hex, 0x4dd, 0, &mut scroll as *mut _ as isize);
        set_raw(self.hex, &out);
        SendMessageW(self.hex, 0x443, 0, palette().control as isize);
        recolor_raw(self.hex);
        let end = out.encode_utf16().count() as i32;
        set_range(
            self.hex,
            Range {
                start: selected.start.min(end),
                end: selected.end.min(end),
            },
        );
        SendMessageW(self.hex, 0x4de, 0, &scroll as *const _ as isize);
        *self.hex_rendered.borrow_mut() = out;
    }
    unsafe fn checksum(&mut self) {
        self.capture(self.active);
        let d = &self.documents[self.active].doc;
        let bytes = if !d.dirty {
            d.path
                .as_ref()
                .and_then(|p| fs::read(p).ok())
                .unwrap_or_else(|| d.bytes())
        } else {
            d.bytes()
        };
        match storage::hashes(&bytes) {
            Ok((md5, sha)) => {
                let hashes = format!(
                    "Source: {}\r\nMD5: {md5}\r\nSHA-256: {sha}",
                    if d.dirty || d.path.is_none() {
                        "editor buffer"
                    } else {
                        "file on disk"
                    }
                );
                let answer = prompt(
                    self.hwnd,
                    "MD5 / SHA-256",
                    &format!("{hashes}\n\nOptional expected MD5 or SHA-256:"),
                    Some(""),
                    &["Compare", "Copy hashes", "Close"],
                );
                if let Some((choice, value)) = answer {
                    if choice == 2 {
                        copy_clipboard(self.hwnd, &hashes);
                    } else if choice == 1 {
                        let value = value.trim();
                        let message = if value.eq_ignore_ascii_case(&md5)
                            || value.eq_ignore_ascii_case(&sha)
                        {
                            "MATCH"
                        } else {
                            "MISMATCH (expected a 32-character MD5 or 64-character SHA-256)"
                        };
                        prompt(self.hwnd, "Checksum comparison", message, None, &["OK"]);
                    }
                }
            }
            Err(e) => error(self.hwnd, &e),
        }
    }
    unsafe fn command(&mut self, id: u16) {
        match id {
            NEW => {
                self.add_document(Document::default());
                self.activate(self.documents.len() - 1);
            }
            OPEN => {
                if let Some(p) = file_dialog(self.hwnd, false, false, None) {
                    self.open(&p);
                }
            }
            SAVE => {
                self.save(self.active, false);
            }
            SAVE_AS => {
                self.save(self.active, true);
            }
            SAVE_ALL => {
                for i in 0..self.documents.len() {
                    self.capture(i);
                    if self.documents[i].doc.dirty && !self.save(i, false) {
                        break;
                    }
                }
            }
            CLOSE => {
                self.close_tab();
            }
            EXIT => {
                PostMessageW(self.hwnd, WM_CLOSE, 0, 0);
            }
            UNDO => {
                SendMessageW(self.edit(), WM_UNDO, 0, 0);
                self.documents[self.active].changed = true;
            }
            REDO => {
                SendMessageW(self.edit(), 0x454, 0, 0);
                self.documents[self.active].changed = true;
            }
            CUT | COPY | PASTE | SELECT_ALL => {
                let msg = match id {
                    CUT => WM_CUT,
                    COPY => WM_COPY,
                    PASTE => WM_PASTE,
                    _ => EM_SETSEL,
                };
                SendMessageW(self.edit(), msg, 0, if id == SELECT_ALL { -1 } else { 0 });
                if id == CUT || id == PASTE {
                    self.documents[self.active].changed = true;
                }
            }
            FIND | REPLACE => self.show_search(false),
            FIND_ALL => self.show_search(true),
            FIND_MORE => {
                let selected = selected_text(self.edit());
                self.show_search(true);
                if !selected.is_empty() {
                    set_text(self.query, &selected);
                    self.search_all();
                }
            }
            FIND_NEXT | DO_NEXT => self.find_next(),
            DO_SEARCH => self.search_all(),
            DO_REPLACE => self.replace_hits(false),
            DO_REPLACE_ALL => self.replace_hits(true),
            HIDE_SEARCH => {
                self.search_visible = false;
                for h in &self.search_controls {
                    ShowWindow(*h, SW_HIDE);
                }
                ShowWindow(self.results, SW_HIDE);
                self.layout();
                SetFocus(self.edit());
            }
            CLEAR => self.highlight(None),
            HL_RED..=HL_GREEN => self.highlight(Some(HIGHLIGHTS[(id - HL_RED) as usize])),
            DARK_MODE => {
                DARK.set(!DARK.get());
                brushes();
                self.apply_theme();
            }
            WRAP => {
                let i = self.active;
                self.documents[i].doc.wrap = !self.documents[i].doc.wrap;
                self.set_wrap(i);
            }
            ZOOM_IN | ZOOM_OUT | ZOOM_RESET => {
                let t = &mut self.documents[self.active];
                t.doc.zoom = if id == ZOOM_RESET {
                    100
                } else {
                    (t.doc.zoom + if id == ZOOM_IN { 10 } else { -10 }).clamp(25, 500)
                };
                SendMessageW(t.edit, 0x4e1, t.doc.zoom as usize, 100);
            }
            FONT_DEFAULT..=FONT_CONSOLAS => {
                self.documents[self.active].doc.font =
                    ["Consolas", "Segoe UI", "Georgia", "Consolas"][(id - FONT_DEFAULT) as usize]
                        .into();
                self.documents[self.active].doc.points = 11;
                self.apply_font(self.active);
            }
            FONT_CUSTOM => {
                let t = &self.documents[self.active];
                let mut log: LOGFONTW = zeroed();
                let name = wide(&t.doc.font);
                for (a, b) in log.lfFaceName.iter_mut().zip(name) {
                    *a = b;
                }
                log.lfHeight = -t.doc.points * self.dpi as i32 / 72;
                let mut choice: CHOOSEFONTW = zeroed();
                choice.lStructSize = size_of::<CHOOSEFONTW>() as u32;
                choice.hwndOwner = self.hwnd;
                choice.lpLogFont = &mut log;
                choice.Flags = CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT | CF_LIMITSIZE;
                choice.nSizeMin = 6;
                choice.nSizeMax = 72;
                if ChooseFontW(&mut choice) != 0 {
                    let len = log.lfFaceName.iter().position(|u| *u == 0).unwrap_or(32);
                    let t = &mut self.documents[self.active];
                    t.doc.font = String::from_utf16_lossy(&log.lfFaceName[..len]);
                    t.doc.points = (choice.iPointSize / 10).clamp(6, 72);
                    self.apply_font(self.active);
                }
            }
            COLOR_BASE..=266 => {
                self.documents[self.active].doc.color = (id - COLOR_BASE) as u8;
                InvalidateRect(self.tabs, null(), 1);
            }
            HEX_LEFT | HEX_RIGHT | HEX_BELOW | HEX_CLOSE => {
                self.capture(self.active);
                self.documents[self.active].doc.hex = match id {
                    HEX_LEFT => 1,
                    HEX_RIGHT => 2,
                    HEX_BELOW => 3,
                    _ => 0,
                };
                self.refresh_hex();
                self.layout();
            }
            HASH => self.checksum(),
            INFO => {
                self.capture(self.active);
                let d = &self.documents[self.active].doc;
                let bytes = d.bytes();
                let (md5, sha) = storage::hashes(&bytes).unwrap_or_default();
                let content=format!("{}\r\nEncoding: {:?} / {:?}\r\nLines: {}  Characters: {}  Bytes: {}\r\nUnsaved: {}\r\nMD5: {md5}\r\nSHA-256: {sha}",d.path.as_ref().map(|p|p.display().to_string()).unwrap_or_else(||"Unsaved scratch tab".into()),d.encoding,d.eol,d.text.split('\n').count(),d.text.chars().count(),bytes.len(),d.dirty);
                if let Some((1, _)) = prompt(
                    self.hwnd,
                    "File info (editor buffer)",
                    &content,
                    None,
                    &["Copy", "Close"],
                ) {
                    copy_clipboard(self.hwnd, &content);
                }
            }
            JUMP => {
                if let Some((1, s)) = prompt(
                    self.hwnd,
                    "Jump to line / character",
                    "Enter line[:column], starting at 1:",
                    Some("1:1"),
                    &["Jump", "Cancel"],
                ) {
                    let mut parts = s.split(':');
                    let line = parts
                        .next()
                        .and_then(|s| s.parse::<i32>().ok())
                        .filter(|n| *n > 0);
                    let col = parts
                        .next()
                        .unwrap_or("1")
                        .parse::<i32>()
                        .unwrap_or(1)
                        .max(1);
                    if let Some(line) = line {
                        self.capture(self.active);
                        if let Some(p) = document::position(
                            &self.documents[self.active].doc.text,
                            line as usize,
                            col as usize,
                        ) {
                            set_range(self.edit(), Range { start: p, end: p });
                            SendMessageW(self.edit(), EM_SCROLLCARET, 0, 0);
                            SetFocus(self.edit());
                        }
                    }
                }
            }
            NEXT_TAB => self.activate((self.active + 1) % self.documents.len()),
            PREV_TAB => {
                self.activate((self.active + self.documents.len() - 1) % self.documents.len())
            }
            MOVE_LEFT | MOVE_RIGHT => {
                let from = self.active;
                let to = if id == MOVE_LEFT {
                    from.saturating_sub(1)
                } else {
                    (from + 1).min(self.documents.len() - 1)
                };
                self.documents.swap(from, to);
                self.active = to;
                self.activate(to);
                self.invalidate_results();
            }
            RECOVERY => {
                if self.recovery {
                    let choice=prompt(self.hwnd,"Disable session recovery","Disable automatic recovery and remove saved recovery snapshots? Open tabs stay open. Unsaved tabs will require Save or Discard on exit.",None,&["Disable","Cancel"]);
                    if matches!(choice, Some((1, _))) {
                        match self.disable_recovery() {
                            Ok(()) => {
                                self.recovery = false;
                                self.last_snapshot.clear();
                            }
                            Err(e) => error(self.hwnd, &e),
                        }
                    }
                } else {
                    if let Err(e) = fs::remove_file(self.dir.join("recovery.off")) {
                        if e.kind() != std::io::ErrorKind::NotFound {
                            error(self.hwnd, &e.to_string());
                            return;
                        }
                    }
                    self.recovery = true;
                    self.allow_snapshot = true;
                    if let Err(e) = self.snapshot() {
                        error(self.hwnd, &e);
                    }
                }
            }
            ERASE_RECOVERY => {
                let choice=prompt(self.hwnd,"Clear recovery data","Remove stored recovery snapshots and turn recovery OFF? Open tabs and saved documents are retained.",None,&["Clear","Cancel"]);
                if matches!(choice, Some((1, _))) {
                    match self.disable_recovery() {
                        Ok(()) => {
                            self.recovery = false;
                            self.allow_snapshot = true;
                            self.last_snapshot.clear();
                            self.note = "Recovery data cleared".into();
                        }
                        Err(e) => error(self.hwnd, &e),
                    }
                }
            }
            UPDATE => {
                if self.update.is_none() {
                    let (tx, rx) = mpsc::channel();
                    self.update = Some(rx);
                    std::thread::spawn(move || {
                        let _ = tx.send(platform::upstream_version());
                    });
                    self.note = "Checking FASM upstream…".into();
                }
            }
            ABOUT => {
                prompt(self.hwnd,"About ByteForge Rust","ByteForge Rust 1.1.2\nNative Win32 / RichEdit — Rust edition\nForked from ByteForge 1.0.0.10 (a401721).\n\nTabs, independent formatting, cross-tab search and crash recovery.\nRecovery stores readable text in your local profile.\n\nBugs / security: jaapengel79@proton.me",None,&["OK"]);
            }
            423 | 424 => self.invalidate_results(),
            _ => {}
        }
        self.capture(self.active);
        self.update_title();
    }
    unsafe fn menu_state(&self) {
        self.menus.check(DARK_MODE, DARK.get());
        self.menus.check(WRAP, self.documents[self.active].doc.wrap);
        self.menus.check(RECOVERY, self.recovery);
        for i in 0..TAB_COLORS.len() {
            self.menus.check(
                COLOR_BASE + i as u16,
                i == self.documents[self.active].doc.color as usize,
            );
        }
        let r = range(self.edit());
        for id in [
            CUT, COPY, FIND_MORE, CLEAR, HL_RED, HL_ORANGE, HL_YELLOW, HL_GREEN,
        ] {
            EnableMenuItem(
                self.menus.root,
                id as u32,
                MF_BYCOMMAND
                    | if r.start != r.end {
                        MF_ENABLED
                    } else {
                        MF_GRAYED
                    },
            );
        }
    }
    fn disable_recovery(&self) -> Result<(), String> {
        storage::atomic_write(
            &self.dir.join("recovery.off"),
            b"Session recovery disabled by user\n",
        )?;
        session::clear(&self.dir)
    }
    unsafe fn context_menu(&mut self, lp: isize) {
        let mut m = Menus::new();
        let pop = m.popup();
        for (id, label) in [
            (CUT, "Cut\tCtrl+X"),
            (COPY, "Copy\tCtrl+C"),
            (PASTE, "Paste\tCtrl+V"),
            (FIND_MORE, "Find selection in all tabs"),
            (CLEAR, "Remove highlight\tCtrl+0"),
            (HL_RED, "Red\tCtrl+1"),
            (HL_ORANGE, "Orange\tCtrl+2"),
            (HL_YELLOW, "Yellow\tCtrl+3"),
            (HL_GREEN, "Green\tCtrl+4"),
        ] {
            m.add(pop, id as usize, label, false);
        }
        m.theme();
        let mut p = POINT {
            x: (lp as i16) as i32,
            y: ((lp >> 16) as i16) as i32,
        };
        if lp == -1 {
            GetCursorPos(&mut p);
        }
        let id = TrackPopupMenu(
            pop,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            p.x,
            p.y,
            0,
            self.hwnd,
            null(),
        );
        DestroyMenu(pop);
        DestroyMenu(m.root);
        if id > 0 {
            self.command(id as u16);
        }
    }
}

unsafe fn make_menus() -> Menus {
    let mut m = Menus::new();
    let groups: [(&str, Vec<(u16, &str)>); 6] = [
        (
            "&File",
            vec![
                (NEW, "&New tab\tCtrl+N"),
                (OPEN, "&Open…\tCtrl+O"),
                (SAVE, "&Save\tCtrl+S"),
                (SAVE_AS, "Save &as…\tCtrl+Shift+S"),
                (SAVE_ALL, "Save a&ll\tCtrl+Alt+S"),
                (CLOSE, "&Close tab\tCtrl+W"),
                (0, ""),
                (INFO, "File &info…\tCtrl+I"),
                (EXIT, "E&xit\tAlt+F4"),
            ],
        ),
        (
            "&Edit",
            vec![
                (UNDO, "&Undo\tCtrl+Z"),
                (REDO, "&Redo\tCtrl+Y"),
                (0, ""),
                (CUT, "Cu&t\tCtrl+X"),
                (COPY, "&Copy\tCtrl+C"),
                (PASTE, "&Paste\tCtrl+V"),
                (SELECT_ALL, "Select &all\tCtrl+A"),
                (0, ""),
                (FIND, "&Find in tab…\tCtrl+F"),
                (FIND_ALL, "Find in all &tabs…\tCtrl+Shift+F"),
                (FIND_NEXT, "Find &next\tF3"),
                (FIND_MORE, "Find &selection\tCtrl+F3"),
                (REPLACE, "&Replace…\tCtrl+H"),
                (JUMP, "&Jump…\tCtrl+G"),
            ],
        ),
        (
            "&Tab",
            vec![
                (NEXT_TAB, "&Next tab\tCtrl+Tab"),
                (PREV_TAB, "&Previous tab\tCtrl+Shift+Tab"),
                (MOVE_LEFT, "Move &left\tCtrl+Alt+Left"),
                (MOVE_RIGHT, "Move &right\tCtrl+Alt+Right"),
            ],
        ),
        (
            "&View",
            vec![
                (DARK_MODE, "&Dark mode\tCtrl+D"),
                (WRAP, "&Word wrap (this tab)\tAlt+Z"),
                (ZOOM_IN, "Zoom &in\tCtrl++"),
                (ZOOM_OUT, "Zoom &out\tCtrl+-"),
                (ZOOM_RESET, "Reset &zoom\tCtrl+Shift+0"),
            ],
        ),
        (
            "&Tools",
            vec![
                (HASH, "&Checksums…\tCtrl+K"),
                (RECOVERY, "&Session recovery\tCtrl+Alt+P"),
                (ERASE_RECOVERY, "Clear recovery &data…\tCtrl+Alt+Delete"),
            ],
        ),
        (
            "&Help",
            vec![
                (UPDATE, "Check FASM &upstream…\tCtrl+U"),
                (ABOUT, "&About ByteForge Rust\tF1"),
            ],
        ),
    ];
    let mut tab_menu = null_mut();
    let mut edit_menu = null_mut();
    let mut view_menu = null_mut();
    let mut tools_menu = null_mut();
    for (label, items) in groups {
        let menu = m.popup();
        for (id, text) in items {
            m.add(menu, id as usize, text, false);
        }
        m.add(m.root, menu as usize, label, true);
        match label {
            "&Tab" => tab_menu = menu,
            "&Edit" => edit_menu = menu,
            "&View" => view_menu = menu,
            "&Tools" => tools_menu = menu,
            _ => {}
        }
    }
    let colors = m.popup();
    for (i, (name, _)) in TAB_COLORS.iter().enumerate() {
        m.add(
            colors,
            (COLOR_BASE + i as u16) as usize,
            &format!("{name}\tCtrl+Alt+{}", i),
            false,
        );
    }
    m.add(tab_menu, colors as usize, "Tab &color", true);
    let hl = m.popup();
    for (id, text) in [
        (CLEAR, "&Remove highlight\tCtrl+0"),
        (HL_RED, "&Red\tCtrl+1"),
        (HL_ORANGE, "&Orange\tCtrl+2"),
        (HL_YELLOW, "&Yellow\tCtrl+3"),
        (HL_GREEN, "&Green\tCtrl+4"),
    ] {
        m.add(hl, id as usize, text, false);
    }
    m.add(edit_menu, hl as usize, "&Highlight", true);
    let fonts = m.popup();
    for (id, text) in [
        (FONT_DEFAULT, "&Default\tCtrl+Shift+1"),
        (FONT_SEGOE, "&Segoe UI\tCtrl+Shift+2"),
        (FONT_GEORGIA, "&Georgia\tCtrl+Shift+3"),
        (FONT_CONSOLAS, "&Consolas\tCtrl+Shift+4"),
        (FONT_CUSTOM, "&Choose font / size…\tCtrl+Shift+P"),
    ] {
        m.add(fonts, id as usize, text, false);
    }
    m.add(view_menu, fonts as usize, "&Font (this tab)", true);
    let hex = m.popup();
    for (id, text) in [
        (HEX_LEFT, "&Left\tCtrl+Shift+L"),
        (HEX_RIGHT, "&Right\tCtrl+Shift+R"),
        (HEX_BELOW, "&Below\tCtrl+Shift+B"),
        (HEX_CLOSE, "&Close\tCtrl+Shift+X"),
    ] {
        m.add(hex, id as usize, text, false);
    }
    m.add(tools_menu, hex as usize, "&Hex viewer", true);
    m
}
fn accelerators() -> Vec<ACCEL> {
    let ctrl = FVIRTKEY | FCONTROL;
    let shift = ctrl | FSHIFT;
    let alt = FVIRTKEY | FALT;
    let ca = ctrl | FALT;
    let entries = vec![
        (ctrl, b'N' as u16, NEW),
        (ctrl, b'O' as u16, OPEN),
        (ctrl, b'S' as u16, SAVE),
        (shift, b'S' as u16, SAVE_AS),
        (ca, b'S' as u16, SAVE_ALL),
        (ctrl, b'W' as u16, CLOSE),
        (ctrl, b'I' as u16, INFO),
        (alt, VK_F4, EXIT),
        (ctrl, b'Z' as u16, UNDO),
        (ctrl, b'Y' as u16, REDO),
        (ctrl, b'X' as u16, CUT),
        (ctrl, b'C' as u16, COPY),
        (ctrl, b'V' as u16, PASTE),
        (ctrl, b'A' as u16, SELECT_ALL),
        (ctrl, b'F' as u16, FIND),
        (shift, b'F' as u16, FIND_ALL),
        (FVIRTKEY, VK_F3, FIND_NEXT),
        (ctrl, VK_F3, FIND_MORE),
        (ctrl, b'H' as u16, REPLACE),
        (ctrl, b'G' as u16, JUMP),
        (ctrl, b'0' as u16, CLEAR),
        (ctrl, b'1' as u16, HL_RED),
        (ctrl, b'2' as u16, HL_ORANGE),
        (ctrl, b'3' as u16, HL_YELLOW),
        (ctrl, b'4' as u16, HL_GREEN),
        (ctrl, b'D' as u16, DARK_MODE),
        (alt, b'Z' as u16, WRAP),
        (ctrl, VK_OEM_PLUS, ZOOM_IN),
        (shift, VK_OEM_PLUS, ZOOM_IN),
        (ctrl, VK_ADD, ZOOM_IN),
        (ctrl, VK_OEM_MINUS, ZOOM_OUT),
        (ctrl, VK_SUBTRACT, ZOOM_OUT),
        (shift, b'0' as u16, ZOOM_RESET),
        (shift, b'1' as u16, FONT_DEFAULT),
        (shift, b'2' as u16, FONT_SEGOE),
        (shift, b'3' as u16, FONT_GEORGIA),
        (shift, b'4' as u16, FONT_CONSOLAS),
        (shift, b'P' as u16, FONT_CUSTOM),
        (shift, b'L' as u16, HEX_LEFT),
        (shift, b'R' as u16, HEX_RIGHT),
        (shift, b'B' as u16, HEX_BELOW),
        (shift, b'X' as u16, HEX_CLOSE),
        (ctrl, b'K' as u16, HASH),
        (ca, b'P' as u16, RECOVERY),
        (ca, VK_DELETE, ERASE_RECOVERY),
        (ctrl, b'U' as u16, UPDATE),
        (FVIRTKEY, VK_F1, ABOUT),
        (ctrl, VK_TAB, NEXT_TAB),
        (shift, VK_TAB, PREV_TAB),
        (ca, VK_LEFT, MOVE_LEFT),
        (ca, VK_RIGHT, MOVE_RIGHT),
        (FVIRTKEY, VK_ESCAPE, HIDE_SEARCH),
    ];
    let mut accels: Vec<_> = entries
        .into_iter()
        .map(|(f_virt, key, cmd)| ACCEL {
            fVirt: f_virt,
            key,
            cmd,
        })
        .collect();
    for i in 0..7 {
        accels.push(ACCEL {
            fVirt: ca,
            key: b'0' as u16 + i,
            cmd: COLOR_BASE + i,
        });
    }
    accels
}
