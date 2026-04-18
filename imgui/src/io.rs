use bitflags::bitflags;
use std::f32;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::os::raw::c_char;
use std::time::Duration;

use crate::input::mouse::MouseButton;
use crate::internal::RawCast;
use crate::sys;

bitflags! {
    /// Configuration flags
    #[repr(transparent)]
    pub struct ConfigFlags: u32 {
        /// Master keyboard navigation enable flag.
        const NAV_ENABLE_KEYBOARD = sys::ImGuiConfigFlags_NavEnableKeyboard;
        /// Master gamepad navigation enable flag.
        const NAV_ENABLE_GAMEPAD = sys::ImGuiConfigFlags_NavEnableGamepad;
        /// Instruction imgui-rs to clear mouse position/buttons in `frame()`.
        const NO_MOUSE = sys::ImGuiConfigFlags_NoMouse;
        /// Instruction backend to not alter mouse cursor shape and visibility.
        const NO_MOUSE_CURSOR_CHANGE = sys::ImGuiConfigFlags_NoMouseCursorChange;
        /// Application is SRGB-aware.
        const IS_SRGB = sys::ImGuiConfigFlags_IsSRGB;
        /// Application is using a touch screen instead of a mouse.
        const IS_TOUCH_SCREEN = sys::ImGuiConfigFlags_IsTouchScreen;
    }
}

bitflags! {
    /// Backend capabilities
    #[repr(transparent)]
    pub struct BackendFlags: u32 {
        /// Backend supports gamepad and currently has one connected
        const HAS_GAMEPAD = sys::ImGuiBackendFlags_HasGamepad;
        /// Backend supports honoring `get_mouse_cursor` value to change the OS cursor shape
        const HAS_MOUSE_CURSORS = sys::ImGuiBackendFlags_HasMouseCursors;
        /// Backend supports `io.want_set_mouse_pos` requests to reposition the OS mouse position.
        const HAS_SET_MOUSE_POS = sys::ImGuiBackendFlags_HasSetMousePos;
        /// Backend renderer supports DrawCmd::vtx_offset.
        const RENDERER_HAS_VTX_OFFSET = sys::ImGuiBackendFlags_RendererHasVtxOffset;
    }
}

/// Settings and inputs/outputs for imgui-rs.
///
/// Transparent newtype wrapper around [`sys::ImGuiIO`]. Field layout is guaranteed by the
/// `#[repr(transparent)]` representation to match the underlying C++ `ImGuiIO` exactly;
/// this is important since arcdps passes the context across a DLL boundary.
///
/// Access fields using their C++ PascalCase names (`io.DisplaySize`, `io.DeltaTime`, ...).
/// Common reads/writes are also exposed as snake_case helper methods.
#[repr(transparent)]
pub struct Io(pub sys::ImGuiIO);

unsafe impl RawCast<sys::ImGuiIO> for Io {}

impl Deref for Io {
    type Target = sys::ImGuiIO;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Io {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Io {
    // ---- snake_case accessors for common fields ----

    pub fn display_size(&self) -> [f32; 2] {
        [self.0.DisplaySize.x, self.0.DisplaySize.y]
    }
    pub fn set_display_size(&mut self, v: [f32; 2]) {
        self.0.DisplaySize = sys::ImVec2 { x: v[0], y: v[1] };
    }
    pub fn delta_time(&self) -> f32 {
        self.0.DeltaTime
    }
    pub fn set_delta_time(&mut self, v: f32) {
        self.0.DeltaTime = v;
    }
    pub fn mouse_pos(&self) -> [f32; 2] {
        [self.0.MousePos.x, self.0.MousePos.y]
    }
    pub fn set_mouse_pos(&mut self, v: [f32; 2]) {
        self.0.MousePos = sys::ImVec2 { x: v[0], y: v[1] };
    }
    pub fn mouse_delta(&self) -> [f32; 2] {
        [self.0.MouseDelta.x, self.0.MouseDelta.y]
    }
    pub fn mouse_wheel(&self) -> f32 {
        self.0.MouseWheel
    }
    pub fn mouse_wheel_h(&self) -> f32 {
        self.0.MouseWheelH
    }
    pub fn key_ctrl(&self) -> bool {
        self.0.KeyCtrl
    }
    pub fn key_shift(&self) -> bool {
        self.0.KeyShift
    }
    pub fn key_alt(&self) -> bool {
        self.0.KeyAlt
    }
    pub fn key_super(&self) -> bool {
        self.0.KeySuper
    }
    pub fn want_capture_mouse(&self) -> bool {
        self.0.WantCaptureMouse
    }
    pub fn want_capture_keyboard(&self) -> bool {
        self.0.WantCaptureKeyboard
    }
    pub fn want_text_input(&self) -> bool {
        self.0.WantTextInput
    }
    pub fn framerate(&self) -> f32 {
        self.0.Framerate
    }
    pub fn config_flags(&self) -> ConfigFlags {
        ConfigFlags::from_bits_truncate(self.0.ConfigFlags as u32)
    }
    pub fn set_config_flags(&mut self, flags: ConfigFlags) {
        self.0.ConfigFlags = flags.bits() as sys::ImGuiConfigFlags;
    }
    pub fn backend_flags(&self) -> BackendFlags {
        BackendFlags::from_bits_truncate(self.0.BackendFlags as u32)
    }
    pub fn set_backend_flags(&mut self, flags: BackendFlags) {
        self.0.BackendFlags = flags.bits() as sys::ImGuiBackendFlags;
    }
    pub fn backend_platform_name(&self) -> *const c_char {
        self.0.BackendPlatformName
    }
    pub fn backend_renderer_name(&self) -> *const c_char {
        self.0.BackendRendererName
    }

    // ---- behavior ----

    /// Queue new character input.
    #[doc(alias = "AddInputCharactersUTF8")]
    pub fn add_input_character(&mut self, character: char) {
        let mut buf = [0u8; 5];
        let s = character.encode_utf8(&mut buf);
        // Null-terminate.
        let len = s.len();
        if len < buf.len() {
            buf[len] = 0;
        }
        unsafe {
            sys::ImGuiIO_AddInputCharactersUTF8(&mut self.0, buf.as_ptr() as *const _);
        }
    }

    /// Update `delta_time` from a [`Duration`].
    pub fn update_delta_time(&mut self, delta: Duration) {
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.0.DeltaTime = if delta_s > 0.0 {
            delta_s
        } else {
            f32::MIN_POSITIVE
        };
    }
}

impl Index<MouseButton> for Io {
    type Output = bool;
    fn index(&self, index: MouseButton) -> &bool {
        &self.0.MouseDown[index as usize]
    }
}

impl IndexMut<MouseButton> for Io {
    fn index_mut(&mut self, index: MouseButton) -> &mut bool {
        &mut self.0.MouseDown[index as usize]
    }
}

#[test]
#[cfg(test)]
fn test_io_layout_matches_sys() {
    use std::mem;
    assert_eq!(mem::size_of::<Io>(), mem::size_of::<sys::ImGuiIO>());
    assert_eq!(mem::align_of::<Io>(), mem::align_of::<sys::ImGuiIO>());
}
