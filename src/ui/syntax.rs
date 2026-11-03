//! Viewport-only foreground decoration using TOM ranges, never the selection.
use super::*;
use byteforge_rust::syntax::{Cache, Context};
use std::{
    collections::VecDeque,
    ffi::c_void,
    time::{Duration, Instant},
};
use windows_sys::core::GUID;

// Minimal typed ABI from Microsoft's TOM.h (IDispatch contributes seven slots).
type Query = unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32;
type Release = unsafe extern "system" fn(*mut c_void) -> u32;
type Undo = unsafe extern "system" fn(*mut c_void, i32, *mut i32) -> i32;
type LongGet = unsafe extern "system" fn(*mut c_void, *mut i32) -> i32;
type LongSet = unsafe extern "system" fn(*mut c_void, i32) -> i32;
type ObjectGet = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32;
type MakeRange = unsafe extern "system" fn(*mut c_void, i32, i32, *mut *mut c_void) -> i32;
type Action = unsafe extern "system" fn(*mut c_void) -> i32;
unsafe fn slot(object: *mut c_void, n: usize) -> *const c_void {
    *(*(object as *mut *mut *const c_void)).add(n)
}
struct Object(*mut c_void);
impl Drop for Object {
    fn drop(&mut self) {
        unsafe {
            let release: Release = std::mem::transmute(slot(self.0, 2));
            release(self.0);
        }
    }
}
pub struct UndoPause {
    doc: Object,
    frozen: bool,
}
unsafe fn text_document(hwnd: HWND) -> Option<Object> {
    let mut ole: *mut c_void = null_mut();
    if SendMessageW(hwnd, 0x43c, 0, &mut ole as *mut _ as isize) == 0 || ole.is_null() {
        return None;
    }
    let ole = Object(ole);
    let query: Query = std::mem::transmute(slot(ole.0, 0));
    let iid = GUID {
        data1: 0x8cc497c0,
        data2: 0xa1df,
        data3: 0x11ce,
        data4: [0x80, 0x98, 0, 0xaa, 0, 0x47, 0xbe, 0x5d],
    };
    let mut doc = null_mut();
    if query(ole.0, &iid, &mut doc) < 0 || doc.is_null() {
        None
    } else {
        Some(Object(doc))
    }
}
/// Keep insertion and plain-text normalization in one user undo action.
pub struct EditCollection(Object);
impl EditCollection {
    pub unsafe fn new(hwnd: HWND) -> Option<Self> {
        let doc = text_document(hwnd)?;
        let begin: Action = std::mem::transmute(slot(doc.0, 20));
        if begin(doc.0) < 0 {
            None
        } else {
            Some(Self(doc))
        }
    }
}
impl Drop for EditCollection {
    fn drop(&mut self) {
        unsafe {
            let end: Action = std::mem::transmute(slot(self.0 .0, 21));
            end(self.0 .0);
        }
    }
}
impl UndoPause {
    pub unsafe fn new(hwnd: HWND) -> Option<Self> {
        let doc = text_document(hwnd)?;
        let undo: Undo = std::mem::transmute(slot(doc.0, 22));
        if undo(doc.0, -9999995, null_mut()) < 0 {
            return None;
        }
        Some(Self { doc, frozen: false })
    }
    unsafe fn freeze(&mut self) {
        let f: LongGet = std::mem::transmute(slot(self.doc.0, 18));
        let mut count = 0;
        self.frozen = f(self.doc.0, &mut count) >= 0;
    }
    unsafe fn font(&self, r: Range) -> Option<Object> {
        let make: MakeRange = std::mem::transmute(slot(self.doc.0, 24));
        let mut range = null_mut();
        if make(self.doc.0, r.start, r.end, &mut range) < 0 || range.is_null() {
            return None;
        }
        let range = Object(range);
        let get: ObjectGet = std::mem::transmute(slot(range.0, 18));
        let mut font = null_mut();
        if get(range.0, &mut font) < 0 || font.is_null() {
            None
        } else {
            Some(Object(font))
        }
    }
}
impl Drop for UndoPause {
    fn drop(&mut self) {
        unsafe {
            if self.frozen {
                let f: LongGet = std::mem::transmute(slot(self.doc.0, 19));
                let mut count = 0;
                f(self.doc.0, &mut count);
            }
            let undo: Undo = std::mem::transmute(slot(self.doc.0, 22));
            undo(self.doc.0, -9999994, null_mut());
        }
    }
}
unsafe fn color(font: &Object, n: usize) -> i32 {
    let get: LongGet = std::mem::transmute(slot(font.0, n));
    let mut value = -9999999;
    if get(font.0, &mut value) < 0 {
        -9999999
    } else {
        value
    }
}
unsafe fn set_color(font: &Object, value: u32) {
    let set: LongSet = std::mem::transmute(slot(font.0, 25));
    set(font.0, value as i32);
}

/// Every visible screen row is clipped independently in both dimensions.
/// A contiguous first-to-last CP range would also include thousands of tokens
/// off the right edge on intervening long JSON/NDJSON rows.
pub unsafe fn visible_areas(hwnd: HWND, len: i32) -> Vec<Range> {
    let mut client: RECT = zeroed();
    GetClientRect(hwnd, &mut client);
    let mut rect: RECT = zeroed();
    SendMessageW(hwnd, EM_GETRECT, 0, &mut rect as *mut _ as isize);
    rect.left = rect.left.max(0);
    rect.top = rect.top.max(0);
    rect.right = rect.right.min(client.right);
    rect.bottom = rect.bottom.min(client.bottom);
    if rect.right <= rect.left || rect.bottom <= rect.top || len == 0 {
        return vec![];
    }
    // Pasted rich text may use smaller fonts than the tab's base font. Sample
    // physical rows so those lines are not silently skipped; cap huge displays.
    let step = ((rect.bottom - rect.top + 4095) / 4096).max(1);
    let point = |x, y| {
        let p = POINT { x, y };
        SendMessageW(hwnd, EM_CHARFROMPOS, 0, &p as *const _ as isize).clamp(0, len as isize) as i32
    };
    let mut out: Vec<Range> = vec![];
    let mut y = rect.top;
    loop {
        let start = point(rect.left, y);
        let end = (point(rect.right - 1, y) + 1).min(len);
        if start < end {
            if let Some(last) = out.last_mut().filter(|r| r.end >= start) {
                last.end = last.end.max(end);
            } else {
                out.push(Range { start, end });
            }
        }
        if y == rect.bottom - 1 {
            break;
        }
        y = (y + step).min(rect.bottom - 1);
    }
    out
}
#[derive(Default)]
pub struct PaintStats {
    pub lex_us: u128,
    pub format_us: u128,
    pub background_us: u128,
    pub total_us: u128,
    pub start: i32,
    pub end: i32,
    pub spans: usize,
    pub background_queries: usize,
    pub areas: usize,
    pub slices: usize,
    pub max_slice_us: u128,
    pub changes: usize,
}
#[derive(PartialEq, Eq)]
struct Key {
    revision: u64,
    dark: bool,
    areas: Vec<Range>,
}
#[derive(Clone, Copy)]
struct Run {
    range: Range,
    color: u32,
}
#[derive(Default)]
pub struct Display {
    pub cache: Cache,
    key: Option<Key>,
    pub decorated: bool,
    pub dirty: bool,
    context: Context,
    len: i32,
    pub stats: PaintStats,
    pending: VecDeque<Run>,
    cycle: Option<Instant>,
}
impl Display {
    pub fn invalidate(&mut self) {
        self.key = None;
        self.pending.clear();
    }
    pub fn pending(&self) -> bool {
        !self.pending.is_empty()
    }
    pub unsafe fn paint(&mut self, hwnd: HWND, text: &str, context: Context, force: bool) {
        if context == Context::Plain && !self.decorated {
            return;
        }
        let started = Instant::now();
        let changed = self.dirty || self.context != context || force;
        if changed {
            self.cache.update(text, context);
            self.dirty = false;
            self.context = context;
            self.len = text.encode_utf16().count() as i32;
        }
        let areas = if context == Context::Plain {
            vec![Range {
                start: 0,
                end: self.len,
            }]
        } else {
            visible_areas(hwnd, self.len)
        };
        let key = Key {
            revision: self.cache.revision,
            dark: DARK.get(),
            areas,
        };
        if force || self.key.as_ref() != Some(&key) {
            self.pending.clear();
            self.cycle = Some(started);
            let mut runs: Vec<Run> = vec![];
            let mut add = |range: Range, color: u32| {
                if range.start >= range.end {
                    return;
                }
                if let Some(last) = runs
                    .last_mut()
                    .filter(|r| r.range.end == range.start && r.color == color)
                {
                    last.range.end = range.end;
                } else {
                    runs.push(Run { range, color });
                }
            };
            for area in &key.areas {
                let mut pos = area.start;
                for span in self.cache.spans(area.start, area.end) {
                    let start = span.start.max(area.start);
                    let end = span.end.min(area.end);
                    add(
                        Range {
                            start: pos,
                            end: start,
                        },
                        palette().text,
                    );
                    add(Range { start, end }, rgb(span.kind.color(DARK.get())));
                    pos = end;
                }
                add(
                    Range {
                        start: pos,
                        end: area.end,
                    },
                    palette().text,
                );
            }
            self.stats = PaintStats {
                lex_us: started.elapsed().as_micros(),
                areas: key.areas.len(),
                start: key.areas.first().map(|r| r.start).unwrap_or(0),
                end: key.areas.last().map(|r| r.end).unwrap_or(0),
                spans: runs.len(),
                ..PaintStats::default()
            };
            self.pending = runs.into();
            self.key = Some(key);
            self.decorated = context != Context::Plain;
        }
        if self.pending.is_empty() {
            return;
        }
        let Some(mut doc) = UndoPause::new(hwnd) else {
            self.pending.clear();
            return;
        };
        doc.freeze();
        let slice = Instant::now();
        let selected = range(hwnd);
        let mut scroll: POINT = zeroed();
        SendMessageW(hwnd, 0x4dd, 0, &mut scroll as *mut _ as isize);
        let modified = SendMessageW(hwnd, EM_GETMODIFY, 0, 0);
        let events = SendMessageW(hwnd, 0x43b, 0, 0);
        SendMessageW(hwnd, 0x445, 0, 0);
        for _ in 0..256 {
            let Some(run) = self.pending.pop_front() else {
                break;
            };
            let Some(font) = doc.font(run.range) else {
                continue;
            };
            let back_start = Instant::now();
            let back = if context == Context::Plain {
                -9999997
            } else {
                color(&font, 18)
            };
            self.stats.background_queries += 1;
            self.stats.background_us += back_start.elapsed().as_micros();
            if back == -9999999 && run.range.end - run.range.start > 1 {
                let mid = run.range.start + (run.range.end - run.range.start) / 2;
                self.pending.push_front(Run {
                    range: Range {
                        start: mid,
                        end: run.range.end,
                    },
                    color: run.color,
                });
                self.pending.push_front(Run {
                    range: Range {
                        start: run.range.start,
                        end: mid,
                    },
                    color: run.color,
                });
            } else {
                let wanted = if back >= 0 { palette().text } else { run.color };
                if color(&font, 24) != wanted as i32 {
                    set_color(&font, wanted);
                    self.stats.changes += 1;
                }
            }
            if slice.elapsed() >= Duration::from_millis(8) {
                break;
            }
        }
        // Only adjust insertion formatting; the actual selection never moves.
        let mut cf: CharFormat = zeroed();
        cf.size = size_of::<CharFormat>() as u32;
        cf.mask = 0x40000000;
        cf.color = palette().text;
        SendMessageW(hwnd, 0x444, 0, &cf as *const _ as isize);
        if selected.start == selected.end {
            SendMessageW(hwnd, 0x444, 1, &cf as *const _ as isize);
        }
        SendMessageW(hwnd, EM_SETMODIFY, modified as usize, 0);
        SendMessageW(hwnd, 0x445, 0, events);
        drop(doc);
        SendMessageW(hwnd, 0x4de, 0, &scroll as *const _ as isize);
        InvalidateRect(hwnd, null(), 0);
        let us = slice.elapsed().as_micros();
        self.stats.format_us += us;
        self.stats.slices += 1;
        self.stats.max_slice_us = self.stats.max_slice_us.max(us);
        self.stats.total_us = self.cycle.map(|t| t.elapsed().as_micros()).unwrap_or(0);
    }
}
pub unsafe fn foreground_at(hwnd: HWND, r: Range) -> Option<u32> {
    let doc = UndoPause::new(hwnd)?;
    let font = doc.font(r)?;
    let c = color(&font, 24);
    (c >= 0).then_some(c as u32)
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
