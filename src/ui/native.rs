#![allow(dead_code)]
use std::{
    cell::{Cell, RefCell},
    mem::{size_of, zeroed},
    path::PathBuf,
    ptr::{null, null_mut},
};
pub use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Dwm::*, Gdi::*},
    System::LibraryLoader::*,
    UI::{
        Controls::Dialogs::*, Controls::*, HiDpi::*, Input::KeyboardAndMouse::*,
        WindowsAndMessaging::*,
    },
};
pub const SS_LEFT: u32 = 0;
pub const SS_CENTERIMAGE: u32 = 0x200;
pub const MK_SHIFT: usize = 4;

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn rgb(hex: u32) -> u32 {
    ((hex & 255) << 16) | (hex & 0xff00) | ((hex >> 16) & 255)
}
#[derive(Clone, Copy)]
pub struct Palette {
    pub window: u32,
    pub surface: u32,
    pub alt: u32,
    pub control: u32,
    pub text: u32,
    pub muted: u32,
    pub border: u32,
}
pub const ACCENT: u32 = 0x2e98ff;
thread_local! {
    pub static DARK:Cell<bool>=const{Cell::new(true)};
    static BRUSHES:RefCell<Option<[HBRUSH;4]>>=const{RefCell::new(None)};
    static UI_FONT:Cell<HFONT>=const{Cell::new(null_mut())};
}
pub fn palette() -> Palette {
    if DARK.get() {
        Palette {
            window: rgb(0x14171c),
            surface: rgb(0x1b1e24),
            alt: rgb(0x252a32),
            control: rgb(0x20242b),
            text: rgb(0xf1f4f6),
            muted: rgb(0x9ea7b3),
            border: rgb(0x353c47),
        }
    } else {
        Palette {
            window: rgb(0xf3f5f8),
            surface: rgb(0xffffff),
            alt: rgb(0xe9edf2),
            control: rgb(0xf8fafc),
            text: rgb(0x17202a),
            muted: rgb(0x5b6572),
            border: rgb(0xc9d0d8),
        }
    }
}
pub unsafe fn brushes() {
    let p = palette();
    BRUSHES.with(|b| {
        let mut b = b.borrow_mut();
        if let Some(old) = b.take() {
            for h in old {
                DeleteObject(h);
            }
        }
        *b = Some([
            CreateSolidBrush(p.window),
            CreateSolidBrush(p.surface),
            CreateSolidBrush(p.control),
            CreateSolidBrush(p.alt),
        ]);
    });
}
pub fn brush(i: usize) -> HBRUSH {
    BRUSHES.with(|b| b.borrow().as_ref().map(|v| v[i]).unwrap_or(null_mut()))
}
pub unsafe fn set_ui_font(dpi: u32) {
    let new = font("Segoe UI", 9, dpi);
    let old = UI_FONT.replace(new);
    if !old.is_null() {
        DeleteObject(old);
    }
}
pub fn ui_font() -> HFONT {
    UI_FONT.get()
}
pub unsafe fn font(face: &str, pt: i32, dpi: u32) -> HFONT {
    CreateFontW(
        -((pt as i64 * dpi as i64) / 72) as i32,
        0,
        0,
        0,
        FW_NORMAL as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET as u32,
        OUT_DEFAULT_PRECIS as u32,
        CLIP_DEFAULT_PRECIS as u32,
        CLEARTYPE_QUALITY as u32,
        0,
        wide(face).as_ptr(),
    )
}
pub unsafe fn title_theme(hwnd: HWND) {
    let p = palette();
    let dark = DARK.get() as i32;
    for (attr, value) in [
        (20, dark as u32),
        (34, p.border),
        (35, if dark != 0 { p.window } else { p.surface }),
        (36, p.text),
    ] {
        DwmSetWindowAttribute(hwnd, attr, &value as *const _ as _, 4);
    }
}
/// Native menu drawing adds a system-color separator outside owner-drawn items.
/// Cover just that separator; keep native menu layout, access keys and hit testing.
pub unsafe fn paint_menu_border(hwnd: HWND, target: HDC) {
    let mut bar: MENUBARINFO = zeroed();
    bar.cbSize = size_of::<MENUBARINFO>() as u32;
    if GetMenuBarInfo(hwnd, OBJID_MENU, 0, &mut bar) == 0 {
        return;
    }
    let mut window: RECT = zeroed();
    GetWindowRect(hwnd, &mut window);
    let y = bar.rcBar.bottom - window.top;
    let r = RECT {
        left: bar.rcBar.left - window.left,
        top: y,
        right: bar.rcBar.right - window.left,
        bottom: y + 1,
    };
    let dc = if target.is_null() {
        GetWindowDC(hwnd)
    } else {
        target
    };
    if dc.is_null() {
        return;
    }
    let b = CreateSolidBrush(palette().border);
    FillRect(dc, &r, b);
    DeleteObject(b);
    if target.is_null() {
        ReleaseDC(hwnd, dc);
    }
}
pub unsafe fn text(hwnd: HWND) -> String {
    let n = GetWindowTextLengthW(hwnd).max(0) as usize;
    let mut b = vec![0u16; n + 1];
    let len = GetWindowTextW(hwnd, b.as_mut_ptr(), b.len() as i32).max(0) as usize;
    String::from_utf16_lossy(&b[..len])
}
pub unsafe fn set_text(hwnd: HWND, s: &str) {
    SetWindowTextW(hwnd, wide(s).as_ptr());
}
pub unsafe fn child(parent: HWND, class: &str, label: &str, id: u16, style: u32) -> HWND {
    let h = CreateWindowExW(
        0,
        wide(class).as_ptr(),
        wide(label).as_ptr(),
        WS_CHILD | WS_VISIBLE | style,
        0,
        0,
        1,
        1,
        parent,
        id as usize as _,
        GetModuleHandleW(null()),
        null(),
    );
    SendMessageW(h, WM_SETFONT, ui_font() as usize, 1);
    h
}
pub unsafe fn error(owner: HWND, message: &str) {
    prompt(owner, "ByteForge Rust — error", message, None, &["OK"]);
}

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub submenu: bool,
    pub separator: bool,
    pub top: bool,
}
pub struct Menus {
    pub root: HMENU,
    // Win32 retains item pointers; each allocation must remain stable when the Vec grows.
    #[allow(clippy::vec_box)]
    pub items: Vec<Box<MenuItem>>,
    pub all: Vec<HMENU>,
}
impl Menus {
    pub unsafe fn new() -> Self {
        let root = CreateMenu();
        Self {
            root,
            items: vec![],
            all: vec![root],
        }
    }
    pub unsafe fn popup(&mut self) -> HMENU {
        let h = CreatePopupMenu();
        self.all.push(h);
        h
    }
    pub unsafe fn add(&mut self, menu: HMENU, id: usize, label: &str, sub: bool) {
        let item = Box::new(MenuItem {
            label: label.into(),
            submenu: sub,
            separator: label.is_empty(),
            top: menu == self.root,
        });
        let data = &*item as *const _ as usize;
        let mut info: MENUITEMINFOW = zeroed();
        info.cbSize = size_of::<MENUITEMINFOW>() as u32;
        info.fMask = MIIM_FTYPE | MIIM_DATA | MIIM_ID | MIIM_STRING;
        let mut accessible_label = wide(label);
        info.dwTypeData = accessible_label.as_mut_ptr();
        info.cch = (accessible_label.len() - 1) as u32;
        info.fType = MFT_OWNERDRAW;
        info.dwItemData = data;
        info.wID = id as u32;
        if sub {
            info.fMask |= MIIM_SUBMENU;
            info.hSubMenu = id as _;
        }
        InsertMenuItemW(menu, u32::MAX, 1, &info);
        self.items.push(item);
    }
    pub unsafe fn theme(&self) {
        for &h in &self.all {
            let mut i: MENUINFO = zeroed();
            i.cbSize = size_of::<MENUINFO>() as u32;
            i.fMask = MIM_BACKGROUND;
            i.hbrBack = brush(1);
            SetMenuInfo(h, &i);
        }
    }
    pub unsafe fn check(&self, id: u16, on: bool) {
        CheckMenuItem(
            self.root,
            id as u32,
            MF_BYCOMMAND | if on { MF_CHECKED } else { MF_UNCHECKED },
        );
    }
}
pub unsafe fn measure_menu(lp: isize) -> isize {
    let m = &mut *(lp as *mut MEASUREITEMSTRUCT);
    if m.CtlType != ODT_MENU {
        return 0;
    }
    let item = &*(m.itemData as *const MenuItem);
    let hdc = GetDC(null_mut());
    let old = SelectObject(hdc, ui_font());
    let label = wide(&item.label.replace('&', ""));
    let mut sz: SIZE = zeroed();
    GetTextExtentPoint32W(hdc, label.as_ptr(), (label.len() - 1) as i32, &mut sz);
    let dpi = GetDeviceCaps(hdc, LOGPIXELSX as i32) as u32;
    let scale = |n: i32| (n as i64 * dpi as i64 / 96) as u32;
    m.itemHeight = scale(if item.separator { 9 } else { 27 });
    m.itemWidth = sz.cx as u32 + scale(if item.top { 20 } else { 76 });
    SelectObject(hdc, old);
    ReleaseDC(null_mut(), hdc);
    1
}
pub unsafe fn draw_menu(lp: isize) -> isize {
    let d = &*(lp as *const DRAWITEMSTRUCT);
    if d.CtlType != ODT_MENU {
        return 0;
    }
    let item = &*(d.itemData as *const MenuItem);
    let p = palette();
    let selected = d.itemState & (ODS_SELECTED | ODS_HOTLIGHT) != 0;
    FillRect(d.hDC, &d.rcItem, brush(if selected { 3 } else { 1 }));
    let dpi = GetDeviceCaps(d.hDC, LOGPIXELSX as i32);
    let s = |n: i32| n * dpi / 96;
    let mut r = d.rcItem;
    if item.separator {
        r.left += s(28);
        r.right -= s(12);
        r.top = (r.top + r.bottom) / 2;
        r.bottom = r.top + 1;
        let b = CreateSolidBrush(p.border);
        FillRect(d.hDC, &r, b);
        DeleteObject(b);
        return 1;
    }
    let old = SelectObject(d.hDC, ui_font());
    SetBkMode(d.hDC, TRANSPARENT as i32);
    SetTextColor(
        d.hDC,
        if d.itemState & ODS_DISABLED != 0 {
            p.muted
        } else {
            p.text
        },
    );
    let parts: Vec<_> = item.label.split('\t').collect();
    r.left += s(if item.top { 8 } else { 28 });
    r.right -= s(if item.top { 8 } else { 22 });
    DrawTextW(
        d.hDC,
        wide(parts[0]).as_ptr(),
        -1,
        &mut r,
        DT_SINGLELINE | DT_VCENTER,
    );
    if let Some(shortcut) = parts.get(1) {
        SetTextColor(d.hDC, p.muted);
        DrawTextW(
            d.hDC,
            wide(shortcut).as_ptr(),
            -1,
            &mut r,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER,
        );
    }
    if d.itemState & ODS_CHECKED != 0 {
        let mut c = d.rcItem;
        c.left += s(8);
        c.right = c.left + s(16);
        SetTextColor(d.hDC, ACCENT);
        DrawTextW(
            d.hDC,
            wide("✓").as_ptr(),
            -1,
            &mut c,
            DT_SINGLELINE | DT_VCENTER,
        );
    }
    if item.submenu && !item.top {
        let mut c = d.rcItem;
        c.left = c.right - s(18);
        SetTextColor(d.hDC, p.muted);
        DrawTextW(
            d.hDC,
            wide("›").as_ptr(),
            -1,
            &mut c,
            DT_SINGLELINE | DT_VCENTER,
        );
    }
    SelectObject(d.hDC, old);
    1
}
pub unsafe fn menu_char(wp: usize, lp: isize) -> isize {
    let menu = lp as HMENU;
    let key = char::from_u32((wp & 65535) as u32)
        .unwrap_or('\0')
        .to_ascii_lowercase();
    for i in 0..GetMenuItemCount(menu) {
        let mut info: MENUITEMINFOW = zeroed();
        info.cbSize = size_of::<MENUITEMINFOW>() as u32;
        info.fMask = MIIM_DATA;
        GetMenuItemInfoW(menu, i as u32, 1, &mut info);
        if info.dwItemData == 0 {
            continue;
        }
        let item = &*(info.dwItemData as *const MenuItem);
        let mut chars = item.label.chars();
        while let Some(c) = chars.next() {
            if c == '&' && chars.next().map(|c| c.to_ascii_lowercase()) == Some(key) {
                return ((MNC_EXECUTE as isize) << 16) | i as isize;
            }
        }
    }
    0
}

pub unsafe fn paint_control(msg: u32, wp: usize, _lp: isize) -> Option<isize> {
    match msg {
        WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX | WM_CTLCOLORBTN => {
            let p = palette();
            let dc = wp as HDC;
            SetTextColor(dc, p.text);
            let edit = matches!(msg, WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX);
            SetBkColor(dc, if edit { p.control } else { p.surface });
            Some(brush(if edit { 2 } else { 1 }) as isize)
        }
        _ => None,
    }
}

struct PromptState {
    result: Cell<usize>,
}
unsafe extern "system" fn prompt_proc(hwnd: HWND, msg: u32, wp: usize, lp: isize) -> isize {
    if let Some(v) = paint_control(msg, wp, lp) {
        return v;
    }
    match msg {
        WM_DRAWITEM => super::draw_button(&*(lp as *const DRAWITEMSTRUCT)),
        WM_ERASEBKGND => {
            let mut r = zeroed();
            GetClientRect(hwnd, &mut r);
            FillRect(wp as HDC, &r, brush(1));
            1
        }
        WM_CLOSE | WM_COMMAND => {
            let choice = if msg == WM_CLOSE { 0 } else { wp & 65535 };
            if msg == WM_CLOSE || (1..=3).contains(&choice) {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const PromptState;
                if !ptr.is_null() {
                    (*ptr)
                        .result
                        .set(if msg == WM_CLOSE { usize::MAX } else { choice });
                }
                return 0;
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

/// A small modal native dialog. Result index starts at 1, 0 means cancelled.
pub unsafe fn prompt(
    owner: HWND,
    title: &str,
    label: &str,
    initial: Option<&str>,
    buttons: &[&str],
) -> Option<(usize, String)> {
    let instance = GetModuleHandleW(null());
    let class = wide("ByteForgeRustPrompt");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(prompt_proc),
        hInstance: instance,
        lpszClassName: class.as_ptr(),
        hCursor: LoadCursorW(null_mut(), IDC_ARROW),
        ..zeroed()
    };
    RegisterClassW(&wc);
    let dpi = GetDpiForWindow(owner).max(96);
    let s = |n: i32| n * dpi as i32 / 96;
    let mut r: RECT = zeroed();
    GetWindowRect(owner, &mut r);
    let width = s(600);
    let lines: usize = label
        .lines()
        .map(|line| line.chars().count().max(1).div_ceil(80))
        .sum();
    let label_height = (lines as i32 * 18 + 8).clamp(48, 260);
    let height = s(label_height + if initial.is_some() { 150 } else { 110 });
    let hwnd = CreateWindowExW(
        WS_EX_DLGMODALFRAME,
        class.as_ptr(),
        wide(title).as_ptr(),
        WS_CAPTION | WS_SYSMENU | WS_POPUP,
        r.left + (r.right - r.left - width) / 2,
        r.top + (r.bottom - r.top - height) / 2,
        width,
        height,
        owner,
        null_mut(),
        instance,
        null(),
    );
    if hwnd.is_null() {
        return None;
    }
    let state = PromptState {
        result: Cell::new(0),
    };
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, &state as *const _ as isize);
    title_theme(hwnd);
    let lbl = child(hwnd, "STATIC", label, 110, SS_LEFT);
    MoveWindow(lbl, s(16), s(16), width - s(48), s(label_height), 1);
    let edit = if let Some(value) = initial {
        let h = child(
            hwnd,
            "EDIT",
            value,
            111,
            WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32,
        );
        MoveWindow(h, s(16), s(label_height + 28), width - s(48), s(28), 1);
        SendMessageW(h, EM_SETSEL, 0, -1);
        h
    } else {
        null_mut()
    };
    for (i, label) in buttons.iter().enumerate() {
        let h = child(
            hwnd,
            "BUTTON",
            label,
            (i + 1) as u16,
            WS_TABSTOP | BS_OWNERDRAW as u32,
        );
        MoveWindow(
            h,
            width - s(32 + (buttons.len() - i) as i32 * 112),
            height - s(80),
            s(100),
            s(30),
            1,
        );
    }
    EnableWindow(owner, 0);
    ShowWindow(hwnd, SW_SHOW);
    SetFocus(if edit.is_null() {
        GetDlgItem(hwnd, 1)
    } else {
        edit
    });
    let mut msg: MSG = zeroed();
    let mut quit = None;
    while state.result.get() == 0 {
        let result = GetMessageW(&mut msg, null_mut(), 0, 0);
        if result <= 0 {
            if result == 0 {
                quit = Some(msg.wParam);
            }
            break;
        }
        if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE as usize {
            state.result.set(usize::MAX);
            continue;
        }
        if msg.message == WM_KEYDOWN && msg.wParam == VK_RETURN as usize && GetFocus() == edit {
            state.result.set(1);
            continue;
        }
        if !IsDialogMessageW(hwnd, &msg).eq(&0) {
            continue;
        }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    let value = if edit.is_null() {
        String::new()
    } else {
        text(edit)
    };
    EnableWindow(owner, 1);
    SetActiveWindow(owner);
    DestroyWindow(hwnd);
    if let Some(code) = quit {
        PostQuitMessage(code as i32);
    }
    let result = state.result.get();
    if result == 0 || result == usize::MAX {
        None
    } else {
        Some((result, value))
    }
}

pub unsafe fn file_dialog(
    owner: HWND,
    save: bool,
    pdf: bool,
    initial: Option<&std::path::Path>,
) -> Option<PathBuf> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    let mut buffer = vec![0u16; 32768];
    if let Some(path) = initial {
        let name: Vec<_> = path.as_os_str().encode_wide().take(32766).collect();
        buffer[..name.len()].copy_from_slice(&name);
    }
    let filter = wide(if pdf {
        "PDF files (*.pdf)\0*.pdf\0\0"
    } else {
        "Text files (*.txt)\0*.txt\0All files (*.*)\0*.*\0\0"
    });
    let ext = wide(if pdf { "pdf" } else { "txt" });
    let mut ofn: OPENFILENAMEW = zeroed();
    ofn.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = owner;
    ofn.lpstrFile = buffer.as_mut_ptr();
    ofn.nMaxFile = buffer.len() as u32;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrDefExt = ext.as_ptr();
    ofn.Flags = OFN_EXPLORER
        | OFN_NOCHANGEDIR
        | OFN_PATHMUSTEXIST
        | if save {
            OFN_OVERWRITEPROMPT
        } else {
            OFN_FILEMUSTEXIST
        };
    let ok = if save {
        GetSaveFileNameW(&mut ofn)
    } else {
        GetOpenFileNameW(&mut ofn)
    };
    if ok != 0 {
        let n = buffer.iter().position(|&c| c == 0).unwrap_or(0);
        Some(std::ffi::OsString::from_wide(&buffer[..n]).into())
    } else {
        let code = CommDlgExtendedError();
        if code != 0 {
            error(owner, &format!("File dialog error: {code}"));
        }
        None
    }
}

// RichEdit definitions are deliberately kept at this FFI boundary.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Range {
    pub start: i32,
    pub end: i32,
}
#[repr(C)]
pub struct TextRange {
    pub range: Range,
    pub text: *mut u16,
}
#[repr(C)]
pub struct GetText {
    pub bytes: u32,
    pub flags: u32,
    pub codepage: u32,
    pub default_char: *const u8,
    pub used_default: *mut i32,
}
#[repr(C)]
pub struct GetTextLength {
    pub flags: u32,
    pub codepage: u32,
}
#[repr(C)]
pub struct SetText {
    pub flags: u32,
    pub codepage: u32,
}
#[repr(C)]
pub struct FindText {
    pub range: Range,
    pub text: *const u16,
    pub found: Range,
}
#[repr(C)]
pub struct CharFormat {
    pub size: u32,
    pub mask: u32,
    pub effects: u32,
    pub height: i32,
    pub offset: i32,
    pub color: u32,
    pub charset: u8,
    pub pitch: u8,
    pub face: [u16; 32],
    pub weight: u16,
    pub spacing: i16,
    pub back: u32,
    pub locale: u32,
    pub reserved: u32,
    pub style: i16,
    pub kerning: u16,
    pub underline: u8,
    pub animation: u8,
    pub rev_author: u8,
    pub reserved1: u8,
}
pub unsafe fn raw_text(hwnd: HWND) -> String {
    let len = GetTextLength {
        // Match GT_USECRLF below: every internal paragraph mark expands to two units.
        flags: 1 | 2 | 8,
        codepage: 1200,
    };
    let n = SendMessageW(hwnd, 0x45f, &len as *const _ as usize, 0).max(0) as usize;
    let mut b = vec![0u16; n + 2];
    let opts = GetText {
        bytes: (b.len() * 2) as u32,
        flags: 1,
        codepage: 1200,
        default_char: null(),
        used_default: null_mut(),
    };
    let n = SendMessageW(
        hwnd,
        0x45e,
        &opts as *const _ as usize,
        b.as_mut_ptr() as isize,
    )
    .max(0) as usize;
    String::from_utf16_lossy(&b[..n.min(b.len())])
}
pub unsafe fn set_raw(hwnd: HWND, s: &str) {
    let opts = SetText {
        flags: 0,
        codepage: 1200,
    };
    SendMessageW(
        hwnd,
        0x461,
        &opts as *const _ as usize,
        wide(s).as_ptr() as isize,
    );
    let mut cf: CharFormat = zeroed();
    cf.size = size_of::<CharFormat>() as u32;
    cf.mask = 0x40000000 | 0x04000000;
    cf.effects = 0x04000000;
    cf.color = palette().text;
    SendMessageW(hwnd, 0x444, 4, &cf as *const _ as isize);
    SendMessageW(hwnd, 0x444, 0, &cf as *const _ as isize);
}
pub unsafe fn range(hwnd: HWND) -> Range {
    let mut r = Range::default();
    SendMessageW(hwnd, 0x434, 0, &mut r as *mut _ as isize);
    r
}
pub unsafe fn set_range(hwnd: HWND, r: Range) {
    SendMessageW(hwnd, 0x437, 0, &r as *const _ as isize);
}
pub unsafe fn format(hwnd: HWND, color: Option<u32>) {
    let mut cf: CharFormat = zeroed();
    cf.size = size_of::<CharFormat>() as u32;
    cf.mask = 0x04000000 | 0x40000000;
    cf.color = palette().text;
    if let Some(c) = color {
        cf.back = themed_highlight(c);
    } else {
        cf.effects = 0x04000000;
    }
    SendMessageW(hwnd, 0x444, 1, &cf as *const _ as isize);
}
// Session metadata keeps the original logical colors. Dark mode uses deeper
// backgrounds so highlighted text can retain the same light foreground.
const HIGHLIGHT_PAIRS: [(u32, u32); 4] = [
    (0xFFA0A0, 0x68343B),
    (0xFFC800, 0x654719),
    (0xFFFF00, 0x57521C),
    (0x80FF80, 0x28583D),
];
pub fn logical_highlight(color: u32) -> u32 {
    let color = rgb(color);
    HIGHLIGHT_PAIRS
        .iter()
        .find(|(light, dark)| *light == color || *dark == color)
        .map(|p| p.0)
        .unwrap_or(color)
}
fn themed_highlight(color: u32) -> u32 {
    let logical = logical_highlight(color);
    let displayed = if DARK.get() {
        HIGHLIGHT_PAIRS
            .iter()
            .find(|p| p.0 == logical)
            .map(|p| p.1)
            .unwrap_or(logical)
    } else {
        logical
    };
    rgb(displayed)
}
/// Recolor only uniform background runs, retaining readable text on existing highlights.
pub unsafe fn recolor_raw(hwnd: HWND) {
    let _undo = super::syntax::UndoPause::new(hwnd);
    let visible = GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_VISIBLE as isize != 0;
    let selected = range(hwnd);
    let mut scroll: POINT = zeroed();
    SendMessageW(hwnd, 0x4dd, 0, &mut scroll as *mut _ as isize);
    SendMessageW(hwnd, WM_SETREDRAW, 0, 0);
    set_range(hwnd, Range { start: 0, end: -1 });
    let end = range(hwnd).end;
    let mut stack = vec![Range { start: 0, end }];
    while let Some(r) = stack.pop() {
        set_range(hwnd, r);
        let mut cf: CharFormat = zeroed();
        cf.size = size_of::<CharFormat>() as u32;
        SendMessageW(hwnd, 0x43a, 1, &mut cf as *mut _ as isize);
        if cf.mask & 0x04000000 == 0 && r.end - r.start > 1 {
            let mid = r.start + (r.end - r.start) / 2;
            stack.push(Range {
                start: r.start,
                end: mid,
            });
            stack.push(Range {
                start: mid,
                end: r.end,
            });
        } else {
            let automatic = cf.effects & 0x04000000 != 0;
            cf.mask = 0x40000000;
            cf.effects = 0;
            cf.color = palette().text;
            if !automatic {
                cf.mask |= 0x04000000;
                cf.back = themed_highlight(cf.back);
            }
            SendMessageW(hwnd, 0x444, 1, &cf as *const _ as isize);
        }
    }
    set_range(hwnd, selected);
    SendMessageW(hwnd, 0x4de, 0, &scroll as *const _ as isize);
    let mut cf: CharFormat = zeroed();
    cf.size = size_of::<CharFormat>() as u32;
    cf.mask = 0x40000000;
    cf.color = palette().text;
    SendMessageW(hwnd, 0x444, 0, &cf as *const _ as isize);
    SendMessageW(hwnd, WM_SETREDRAW, 1, 0);
    if !visible {
        ShowWindow(hwnd, SW_HIDE);
    }
    InvalidateRect(hwnd, null(), 1);
}
pub unsafe fn selected_text(hwnd: HWND) -> String {
    let r = range(hwnd);
    let mut b = vec![0u16; (r.end - r.start).max(0) as usize + 1];
    let t = TextRange {
        range: r,
        text: b.as_mut_ptr(),
    };
    SendMessageW(hwnd, 0x44b, 0, &t as *const _ as isize);
    let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf16_lossy(&b[..n])
}
pub unsafe fn rich_find(hwnd: HWND, needle: &str, start: i32, match_case: bool) -> Option<Range> {
    let w = wide(needle);
    let mut f = FindText {
        range: Range { start, end: -1 },
        text: w.as_ptr(),
        found: Range::default(),
    };
    let pos = SendMessageW(
        hwnd,
        0x47c,
        1 | if match_case { 4 } else { 0 },
        &mut f as *mut _ as isize,
    );
    if pos < 0 {
        None
    } else {
        Some(f.found)
    }
}

pub unsafe fn copy_clipboard(owner: HWND, value: &str) -> bool {
    use windows_sys::Win32::System::{DataExchange::*, Memory::*};
    let bytes = wide(value);
    let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len() * 2);
    if h.is_null() {
        return false;
    }
    let p = GlobalLock(h) as *mut u16;
    if p.is_null() {
        GlobalFree(h);
        return false;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    GlobalUnlock(h);
    if OpenClipboard(owner) == 0 {
        GlobalFree(h);
        return false;
    }
    let ok = EmptyClipboard() != 0 && !SetClipboardData(13, h).is_null();
    CloseClipboard();
    if !ok {
        GlobalFree(h);
    }
    ok
}
