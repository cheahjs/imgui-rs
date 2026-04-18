use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_void};
use std::panic::catch_unwind;
use std::process;
use std::ptr;

use crate::sys;
use crate::Ui;

/// Trait for clipboard backends
pub trait ClipboardBackend: 'static {
    /// Returns the current clipboard contents as an owned string, or None if the clipboard is
    /// empty or inaccessible.
    fn get(&mut self) -> Option<String>;
    /// Sets the clipboard contents.
    fn set(&mut self, value: &str);
}

pub(crate) struct ClipboardContext {
    backend: Box<dyn ClipboardBackend>,
    // retained so the C callback's returned pointer stays valid
    last_value: CString,
}

impl ClipboardContext {
    pub fn new<T: ClipboardBackend>(backend: T) -> ClipboardContext {
        ClipboardContext {
            backend: Box::new(backend) as Box<dyn ClipboardBackend>,
            last_value: CString::default(),
        }
    }

    pub fn dummy() -> ClipboardContext {
        Self {
            backend: Box::new(DummyClipboardContext),
            last_value: CString::default(),
        }
    }
}

pub struct DummyClipboardContext;
impl ClipboardBackend for DummyClipboardContext {
    fn get(&mut self) -> Option<String> {
        None
    }
    fn set(&mut self, _: &str) {}
}

impl fmt::Debug for ClipboardContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClipboardContext")
            .field("backend", &(&(*self.backend) as *const _))
            .field("last_value", &self.last_value)
            .finish()
    }
}

pub(crate) unsafe extern "C" fn get_clipboard_text(
    _ctx: *mut sys::ImGuiContext,
) -> *const c_char {
    // imgui 1.92 changed the signature to take an ImGuiContext*; user data is stored
    // on ImGuiPlatformIO::Platform_ClipboardUserData.
    let platform_io = sys::igGetPlatformIO();
    let user_data = (*platform_io).Platform_ClipboardUserData;
    if user_data.is_null() {
        return ptr::null();
    }
    let result = catch_unwind(|| {
        let ctx = &mut *(user_data as *mut ClipboardContext);
        match ctx.backend.get() {
            Some(text) => {
                ctx.last_value = CString::new(text).unwrap();
                ctx.last_value.as_ptr()
            }
            None => ptr::null(),
        }
    });
    result.unwrap_or_else(|_| {
        eprintln!("Clipboard getter panicked");
        process::abort();
    })
}

pub(crate) unsafe extern "C" fn set_clipboard_text(
    _ctx: *mut sys::ImGuiContext,
    text: *const c_char,
) {
    let platform_io = sys::igGetPlatformIO();
    let user_data = (*platform_io).Platform_ClipboardUserData;
    if user_data.is_null() || text.is_null() {
        return;
    }
    let result = catch_unwind(|| {
        let ctx = &mut *(user_data as *mut ClipboardContext);
        if let Ok(s) = CStr::from_ptr(text).to_str() {
            ctx.backend.set(s);
        }
    });
    result.unwrap_or_else(|_| {
        eprintln!("Clipboard setter panicked");
        process::abort();
    });
}

/// # Clipboard
impl<'ui> Ui<'ui> {
    /// Returns the current clipboard contents as text, or None if the clipboard is empty or
    /// cannot be accessed.
    pub fn clipboard_text(&self) -> Option<String> {
        unsafe {
            let p = sys::igGetClipboardText();
            if p.is_null() || *p == 0 {
                None
            } else {
                CStr::from_ptr(p).to_str().ok().map(|s| s.to_owned())
            }
        }
    }

    /// Sets the clipboard contents. Does nothing if the clipboard cannot be accessed.
    pub fn set_clipboard_text(&self, text: impl AsRef<str>) {
        let c = CString::new(text.as_ref()).unwrap_or_default();
        unsafe {
            sys::igSetClipboardText(c.as_ptr());
        }
    }
}

// user_data pointer support for the ClipboardContext set via context.rs
#[allow(dead_code)]
pub(crate) unsafe fn set_clipboard_context(ctx: *mut c_void) {
    let platform_io = sys::igGetPlatformIO();
    (*platform_io).Platform_ClipboardUserData = ctx;
    (*platform_io).Platform_GetClipboardTextFn = Some(get_clipboard_text);
    (*platform_io).Platform_SetClipboardTextFn = Some(set_clipboard_text);
}
