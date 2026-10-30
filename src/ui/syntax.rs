//! RichEdit foreground decoration. Never touches text, font, background, or undo.
use super::*;
use byteforge_rust::syntax::{Cache, Context};
use std::ffi::c_void;
use windows_sys::core::GUID;

// Minimal, typed TOM ABI boundary from Microsoft's TOM.h. ITextDocument inherits
// seven IUnknown/IDispatch slots; Undo is slot 22. No PowerShell/COM server runs.
type Query = unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32;
type Release = unsafe extern "system" fn(*mut c_void) -> u32;
type Undo = unsafe extern "system" fn(*mut c_void, i32, *mut i32) -> i32;
unsafe fn slot(object: *mut c_void, n: usize) -> *const c_void {
    *(*(object as *mut *mut *const c_void)).add(n)
}
pub struct UndoPause(*mut c_void);
impl UndoPause {
    pub unsafe fn new(hwnd: HWND) -> Option<Self> {
        let mut ole: *mut c_void = null_mut();
        if SendMessageW(hwnd, 0x43c, 0, &mut ole as *mut _ as isize) == 0 || ole.is_null() {
            return None;
        }
        let query: Query = std::mem::transmute(slot(ole, 0));
        let release: Release = std::mem::transmute(slot(ole, 2));
        let iid = GUID {
            data1: 0x8cc497c0,
            data2: 0xa1df,
            data3: 0x11ce,
            data4: [0x80, 0x98, 0, 0xaa, 0, 0x47, 0xbe, 0x5d],
        };
        let mut doc = null_mut();
        let hr = query(ole, &iid, &mut doc);
        release(ole);
        if hr < 0 || doc.is_null() {
            return None;
        }
        let undo: Undo = std::mem::transmute(slot(doc, 22));
        if undo(doc, -9999995, null_mut()) < 0 {
            let release: Release = std::mem::transmute(slot(doc, 2));
            release(doc);
            return None;
        }
        Some(Self(doc))
    }
}
impl Drop for UndoPause {
    fn drop(&mut self) {
        unsafe {
            let undo: Undo = std::mem::transmute(slot(self.0, 22));
            undo(self.0, -9999994, null_mut());
            let release: Release = std::mem::transmute(slot(self.0, 2));
            release(self.0);
        }
    }
}

#[derive(Default)]
pub struct Display {
    pub cache: Cache,
    key: Option<(u64, bool, i32, i32)>,
    pub decorated: bool,
    pub dirty: bool,
    context: Context,
    len: i32,
}
impl Display {
    pub fn invalidate(&mut self) {
        self.key = None;
    }
    pub unsafe fn paint(&mut self, hwnd: HWND, text: &str, context: Context, force: bool) {
        if context == Context::Plain && !self.decorated {
            return;
        }
        if self.dirty || self.context != context || force {
            self.cache.update(text, context);
            self.dirty = false;
            self.context = context;
            self.len = text.encode_utf16().count() as i32;
        }
        let len = self.len;
        let mut bounds: RECT = zeroed();
        GetClientRect(hwnd, &mut bounds);
        let point = |x, y| {
            let pt = POINT { x, y };
            SendMessageW(hwnd, 0x427, 0, &pt as *const _ as isize).clamp(0, len as isize) as i32
        };
        let start = if context == Context::Plain {
            0
        } else {
            point(0, 0)
        };
        let end = if context == Context::Plain {
            len
        } else {
            (point(bounds.right.max(1) - 1, bounds.bottom.max(1) - 1) + 1).min(len)
        };
        let key = (self.cache.revision, DARK.get(), start, end);
        if !force && self.key == Some(key) {
            return;
        }
        let Some(_undo) = UndoPause::new(hwnd) else {
            return;
        };
        let selected = range(hwnd);
        let modified = SendMessageW(hwnd, EM_GETMODIFY, 0, 0);
        let mut scroll: POINT = zeroed();
        SendMessageW(hwnd, 0x4dd, 0, &mut scroll as *mut _ as isize);
        let visible = GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_VISIBLE as isize != 0;
        let events = SendMessageW(hwnd, 0x43b, 0, 0);
        SendMessageW(hwnd, 0x445, 0, 0);
        SendMessageW(hwnd, WM_SETREDRAW, 0, 0);
        let mut cf: CharFormat = zeroed();
        cf.size = size_of::<CharFormat>() as u32;
        cf.mask = 0x40000000;
        cf.color = palette().text;
        let area = Range { start, end };
        set_range(hwnd, area);
        SendMessageW(hwnd, 0x444, 1, &cf as *const _ as isize);
        for span in self.cache.spans(start, end) {
            set_range(
                hwnd,
                Range {
                    start: span.start.max(start),
                    end: span.end.min(end),
                },
            );
            cf.color = rgb(span.kind.color(DARK.get()));
            SendMessageW(hwnd, 0x444, 1, &cf as *const _ as isize);
        }
        // Manual highlighter backgrounds take priority over syntax foregrounds.
        // Read live ranges so inserting/deleting before a mark cannot desync them.
        let mut stack = vec![area];
        while let Some(r) = stack.pop() {
            if r.start >= r.end {
                continue;
            }
            set_range(hwnd, r);
            let mut back: CharFormat = zeroed();
            back.size = size_of::<CharFormat>() as u32;
            SendMessageW(hwnd, 0x43a, 1, &mut back as *mut _ as isize);
            if back.mask & 0x04000000 == 0 && r.end - r.start > 1 {
                let mid = r.start + (r.end - r.start) / 2;
                stack.push(Range {
                    start: r.start,
                    end: mid,
                });
                stack.push(Range {
                    start: mid,
                    end: r.end,
                });
            } else if back.effects & 0x04000000 == 0 {
                cf.color = palette().text;
                SendMessageW(hwnd, 0x444, 1, &cf as *const _ as isize);
            }
        }
        set_range(hwnd, selected);
        cf.color = palette().text;
        SendMessageW(hwnd, 0x444, 0, &cf as *const _ as isize);
        if selected.start == selected.end {
            SendMessageW(hwnd, 0x444, 1, &cf as *const _ as isize);
        }
        SendMessageW(hwnd, 0x4de, 0, &scroll as *const _ as isize);
        SendMessageW(hwnd, EM_SETMODIFY, modified as usize, 0);
        SendMessageW(hwnd, 0x445, 0, events);
        SendMessageW(hwnd, WM_SETREDRAW, 1, 0);
        if !visible {
            ShowWindow(hwnd, SW_HIDE);
        }
        InvalidateRect(hwnd, null(), 0);
        self.key = Some(key);
        self.decorated = context != Context::Plain;
    }
}
pub unsafe extern "system" fn editor_proc(
    hwnd: HWND,
    msg: u32,
    wp: usize,
    lp: isize,
    _id: usize,
    _data: usize,
) -> isize {
    let result = DefSubclassProc(hwnd, msg, wp, lp);
    if matches!(
        msg,
        WM_VSCROLL
            | WM_HSCROLL
            | WM_MOUSEWHEEL
            | WM_MOUSEHWHEEL
            | WM_KEYUP
            | WM_LBUTTONUP
            | WM_SIZE
    ) {
        PostMessageW(GetParent(hwnd), SYNTAX_VIEW, hwnd as usize, 0);
    }
    result
}
