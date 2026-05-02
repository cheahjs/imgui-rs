use bitflags::bitflags;
use std::f32;
use std::ops::{Index, IndexMut};
use std::os::raw::{c_char, c_void};
use std::time::Duration;

use crate::fonts::atlas::FontAtlas;
use crate::fonts::font::Font;
use crate::input::keyboard::Key;
use crate::input::mouse::MouseButton;
use crate::internal::{ImVector, RawCast};
use crate::{sys, MouseSource};

bitflags! {
    /// Configuration flags
    #[repr(transparent)]
    pub struct ConfigFlags: u32 {
        /// Master keyboard navigation enable flag.
        ///
        /// `frame()` will automatically fill `io.nav_inputs` based on `io.keys_down`.
        const NAV_ENABLE_KEYBOARD = sys::ImGuiConfigFlags_NavEnableKeyboard;
        /// Master gamepad navigation enable flag.
        ///
        /// This is mostly to instruct the backend to fill `io.nav_inputs`. The backend
        /// also needs to set `BackendFlags::HasGamepad`.
        const NAV_ENABLE_GAMEPAD = sys::ImGuiConfigFlags_NavEnableGamepad;
        /// Instruction navigation to move the mouse cursor.
        ///
        /// May be useful on TV/console systems where moving a virtual mouse is awkward.
        /// Will update `io.mouse_pos` and set `io.want_set_mouse_pos = true`. If enabled,
        /// you *must* honor `io.want_set_mouse_pos`, or imgui-rs will react as if the mouse is
        /// jumping around back and forth.
        const NAV_ENABLE_SET_MOUSE_POS = sys::ImGuiConfigFlags_NavEnableSetMousePos;
        /// Instruction navigation to not set the `io.want_capture_keyboard` flag when
        /// `io.nav_active` is set.
        const NAV_NO_CAPTURE_KEYBOARD = sys::ImGuiConfigFlags_NavNoCaptureKeyboard;
        /// Instruction imgui-rs to clear mouse position/buttons in `frame()`.
        ///
        /// This allows ignoring the mouse information set by the backend.
        const NO_MOUSE = sys::ImGuiConfigFlags_NoMouse;
        /// Instruction backend to not alter mouse cursor shape and visibility.
        ///
        /// Use if the backend cursor changes are interfering with yours and you don't want to use
        /// `set_mouse_cursor` to change the mouse cursor. You may want to honor requests from
        /// imgui-rs by reading `get_mouse_cursor` yourself instead.
        const NO_MOUSE_CURSOR_CHANGE = sys::ImGuiConfigFlags_NoMouseCursorChange;
        /// Application is SRGB-aware.
        ///
        /// Not used by core imgui-rs.
        const IS_SRGB = sys::ImGuiConfigFlags_IsSRGB;
        /// Application is using a touch screen instead of a mouse.
        ///
        /// Not used by core imgui-rs.
        const IS_TOUCH_SCREEN = sys::ImGuiConfigFlags_IsTouchScreen;

        #[cfg(feature = "docking")]
        const DOCKING_ENABLE = sys::ImGuiConfigFlags_DockingEnable;

        #[cfg(feature = "docking")]
        const VIEWPORTS_ENABLE = sys::ImGuiConfigFlags_ViewportsEnable;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct ViewportFlags: u32 {
        const IS_PLATFORM_WINDOW = sys::ImGuiViewportFlags_IsPlatformWindow;
        const IS_PLATFORM_MONITOR = sys::ImGuiViewportFlags_IsPlatformMonitor;
        const OWNED_BY_APP = sys::ImGuiViewportFlags_OwnedByApp;
        #[cfg(feature = "docking")]
        const NO_DECORATION = sys::ImGuiViewportFlags_NoDecoration;
        #[cfg(feature = "docking")]
        const NO_TASK_BAR_ICON = sys::ImGuiViewportFlags_NoTaskBarIcon;
        #[cfg(feature = "docking")]
        const NO_FOCUS_ON_APPEARING = sys::ImGuiViewportFlags_NoFocusOnAppearing;
        #[cfg(feature = "docking")]
        const NO_FOCUS_ON_CLICK = sys::ImGuiViewportFlags_NoFocusOnClick;
        #[cfg(feature = "docking")]
        const NO_INPUTS = sys::ImGuiViewportFlags_NoInputs;
        #[cfg(feature = "docking")]
        const NO_RENDERER_CLEAR = sys::ImGuiViewportFlags_NoRendererClear;
        #[cfg(feature = "docking")]
        const TOP_MOST = sys::ImGuiViewportFlags_TopMost;
        #[cfg(feature = "docking")]
        const IS_MINIMIZED = sys::ImGuiViewportFlags_IsMinimized;
        #[cfg(feature = "docking")]
        const NO_AUTO_MERGE = sys::ImGuiViewportFlags_NoAutoMerge;
        #[cfg(feature = "docking")]
        const CAN_HOST_OTHER_WINDOWS = sys::ImGuiViewportFlags_CanHostOtherWindows;
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
        ///
        /// Only used if `ConfigFlags::NavEnableSetMousePos` is set.
        const HAS_SET_MOUSE_POS = sys::ImGuiBackendFlags_HasSetMousePos;
        /// Backend renderer supports DrawCmd::vtx_offset.
        ///
        /// This enables output of large meshes (64K+ vertices) while still using 16-bits indices.
        const RENDERER_HAS_VTX_OFFSET = sys::ImGuiBackendFlags_RendererHasVtxOffset;
        #[cfg(not(feature = "docking"))]
        /// Backend renderer supports Dear ImGui 1.92 texture create/update/destroy requests.
        const RENDERER_HAS_TEXTURES = sys::ImGuiBackendFlags_RendererHasTextures;

        #[cfg(feature = "docking")]
        /// Set if the platform backend supports viewports.
        const PLATFORM_HAS_VIEWPORTS = sys::ImGuiBackendFlags_PlatformHasViewports;
        #[cfg(feature = "docking")]
        /// Set if the renderer backend supports viewports.
        const RENDERER_HAS_VIEWPORTS = sys::ImGuiBackendFlags_RendererHasViewports;
    }
}

/// Settings and inputs/outputs for imgui-rs
#[repr(C)]
pub struct Io {
    /// Flags set by user/application
    pub config_flags: ConfigFlags,
    /// Flags set by backend
    pub backend_flags: BackendFlags,
    /// Main display size in pixels
    pub display_size: [f32; 2],

    #[cfg(not(feature = "docking"))]
    pub display_framebuffer_scale: [f32; 2],

    /// Time elapsed since last frame, in seconds
    pub delta_time: f32,
    /// Minimum time between saving positions/sizes to .ini file, in seconds
    pub ini_saving_rate: f32,

    pub(crate) ini_filename: *const c_char,
    pub(crate) log_filename: *const c_char,
    user_data: *mut c_void,

    pub(crate) fonts: *mut FontAtlas,

    #[cfg(feature = "docking")]
    pub font_global_scale: f32,
    #[cfg(feature = "docking")]
    pub font_allow_user_scaling: bool,
    pub(crate) font_default: *mut Font,
    #[cfg(not(feature = "docking"))]
    pub font_allow_user_scaling: bool,
    #[cfg(feature = "docking")]
    pub display_framebuffer_scale: [f32; 2],

    #[cfg(feature = "docking")]
    pub config_docking_no_split: bool,
    #[cfg(feature = "docking")]
    pub config_docking_with_shift: bool,
    #[cfg(feature = "docking")]
    pub config_docking_always_tab_bar: bool,
    #[cfg(feature = "docking")]
    pub config_docking_transparent_payload: bool,
    #[cfg(feature = "docking")]
    pub config_viewports_no_auto_merge: bool,
    #[cfg(feature = "docking")]
    pub config_viewports_no_task_bar_icon: bool,
    #[cfg(feature = "docking")]
    pub config_viewports_no_decoration: bool,
    #[cfg(feature = "docking")]
    pub config_viewports_no_default_parent: bool,

    #[cfg(not(feature = "docking"))]
    pub config_nav_swap_gamepad_buttons: bool,
    #[cfg(not(feature = "docking"))]
    pub config_nav_move_set_mouse_pos: bool,
    #[cfg(not(feature = "docking"))]
    pub config_nav_capture_keyboard: bool,
    #[cfg(not(feature = "docking"))]
    pub config_nav_escape_clear_focus_item: bool,
    #[cfg(not(feature = "docking"))]
    pub config_nav_escape_clear_focus_window: bool,
    #[cfg(not(feature = "docking"))]
    pub config_nav_cursor_visible_auto: bool,
    #[cfg(not(feature = "docking"))]
    pub config_nav_cursor_visible_always: bool,

    /// Request imgui-rs to draw a mouse cursor for you
    pub mouse_draw_cursor: bool,
    /// macOS-style input behavior.
    pub config_mac_os_behaviors: bool,

    #[cfg(feature = "docking")]
    pub config_nav_swap_gamepad_buttons: bool,

    /// Enable input queue trickling.
    pub config_input_trickle_event_queue: bool,
    /// Set to false to disable blinking cursor
    pub config_input_text_cursor_blink: bool,
    /// Pressing Enter will keep item active and select contents (single-line only).
    pub config_input_text_enter_keep_active: bool,
    /// Enable turning DragXXX widgets into text input with a simple mouse click-release.
    pub config_drag_click_to_input_text: bool,
    /// Enable resizing of windows from their edges and from the lower-left corner.
    pub config_windows_resize_from_edges: bool,
    /// Only allow moving windows when clicked+dragged from the title bar.
    pub config_windows_move_from_title_bar_only: bool,

    #[cfg(not(feature = "docking"))]
    pub config_windows_copy_contents_with_ctrl_c: bool,

    /// Enable scrolling page by page when clicking outside the scrollbar grab.
    pub config_scrollbar_scroll_by_page: bool,

    /// Compact memory usage when unused. Set to -1.0 to disable.
    pub config_memory_compact_timer: f32,

    /// Time for a double-click, in seconds
    pub mouse_double_click_time: f32,
    /// Distance threshold to stay in to validate a double-click, in pixels
    pub mouse_double_click_max_dist: f32,
    /// Distance threshold before considering we are dragging
    pub mouse_drag_threshold: f32,
    /// When holding a key/button, time before it starts repeating, in seconds
    pub key_repeat_delay: f32,
    /// When holding a key/button, rate at which it repeats, in seconds
    pub key_repeat_rate: f32,

    pub config_error_recovery: bool,
    pub config_error_recovery_enable_assert: bool,
    pub config_error_recovery_enable_debug_log: bool,
    pub config_error_recovery_enable_tooltip: bool,

    pub config_debug_is_debugger_present: bool,
    pub config_debug_highlight_id_conflicts: bool,

    #[cfg(not(feature = "docking"))]
    pub config_debug_highlight_id_conflicts_show_item_picker: bool,

    pub config_debug_begin_return_value_once: bool,
    pub config_debug_begin_return_value_loop: bool,

    pub config_debug_ignore_focus_loss: bool,
    pub config_debug_ini_settings: bool,

    pub(crate) backend_platform_name: *const c_char,
    pub(crate) backend_renderer_name: *const c_char,
    pub(crate) backend_platform_user_data: *mut c_void,
    pub(crate) backend_renderer_user_data: *mut c_void,
    backend_language_user_data: *mut c_void,

    pub want_capture_mouse: bool,
    pub want_capture_keyboard: bool,
    pub want_text_input: bool,
    pub want_set_mouse_pos: bool,
    pub want_save_ini_settings: bool,
    pub nav_active: bool,
    pub nav_visible: bool,
    pub framerate: f32,
    pub metrics_render_vertices: i32,
    pub metrics_render_indices: i32,
    pub metrics_render_windows: i32,
    pub metrics_active_windows: i32,

    pub mouse_delta: [f32; 2],
    pub(crate) ctx: *mut sys::ImGuiContext,
    pub mouse_pos: [f32; 2],
    pub mouse_down: [bool; 5],
    pub mouse_wheel: f32,
    pub mouse_wheel_h: f32,

    pub mouse_source: MouseSource,

    #[cfg(feature = "docking")]
    mouse_hovered_viewport: sys::ImGuiID,

    pub key_ctrl: bool,
    pub key_shift: bool,
    pub key_alt: bool,
    pub key_super: bool,
    key_mods: sys::ImGuiKeyChord,

    keys_data: [sys::ImGuiKeyData; sys::ImGuiKey_NamedKey_COUNT as usize],

    pub want_capture_mouse_unless_popup_close: bool,

    mouse_pos_prev: [f32; 2],
    mouse_clicked_pos: [[f32; 2]; 5],
    mouse_clicked_time: [f64; 5],
    mouse_clicked: [bool; 5],
    mouse_double_clicked: [bool; 5],
    mouse_clicked_count: [u16; 5],
    mouse_clicked_last_count: [u16; 5],
    mouse_released: [bool; 5],

    #[cfg(not(feature = "docking"))]
    mouse_released_time: [f64; 5],

    mouse_down_owned: [bool; 5],
    mouse_down_owned_unless_popup_close: [bool; 5],

    mouse_wheel_request_axis_swap: bool,
    mouse_ctrl_left_as_right_click: bool,

    mouse_down_duration: [f32; 5],
    mouse_down_duration_prev: [f32; 5],

    #[cfg(feature = "docking")]
    mouse_drag_max_distance_abs: [sys::ImVec2; 5],

    mouse_drag_max_distance_sqr: [f32; 5],
    pen_pressure: f32,

    pub app_focus_lost: bool,
    app_accepting_events: bool,

    #[cfg(feature = "docking")]
    backend_using_legacy_key_arrays: sys::ImS8,
    #[cfg(feature = "docking")]
    backend_using_legacy_nav_input_array: bool,

    input_queue_surrogate: sys::ImWchar16,
    input_queue_characters: ImVector<sys::ImWchar>,

    #[cfg(not(feature = "docking"))]
    pub font_global_scale: f32,
    #[cfg(not(feature = "docking"))]
    pub(crate) get_clipboard_text_fn:
        Option<unsafe extern "C" fn(user_data: *mut c_void) -> *const c_char>,
    #[cfg(not(feature = "docking"))]
    pub(crate) set_clipboard_text_fn:
        Option<unsafe extern "C" fn(user_data: *mut c_void, text: *const c_char)>,
    #[cfg(not(feature = "docking"))]
    pub(crate) clipboard_user_data: *mut c_void,
}

unsafe impl RawCast<sys::ImGuiIO> for Io {}

impl Io {
    /// Queue new character input
    #[doc(alias = "AddInputCharactersUTF8")]
    pub fn add_input_character(&mut self, character: char) {
        let mut buf = [0; 5];
        character.encode_utf8(&mut buf);
        unsafe {
            sys::ImGuiIO_AddInputCharactersUTF8(self.raw_mut(), buf.as_ptr() as *const _);
        }
    }
    /// Clear character input buffer
    #[doc(alias = "ClearCharacters")]
    pub fn clear_input_characters(&mut self) {
        unsafe {
            sys::ImGuiIO_ClearInputKeys(self.raw_mut());
        }
    }
    /// Peek character input buffer, return a copy of entire buffer
    pub fn peek_input_characters(&self) -> String {
        self.input_queue_characters().collect()
    }

    /// Returns a view of the data in the input queue (without copying it).
    ///
    /// The returned iterator is a simple mapping over a slice more or less what
    /// you need for random access to the data (Rust has no
    /// `RandomAccessIterator`, or we'd use that).
    pub fn input_queue_characters(
        &self,
    ) -> impl DoubleEndedIterator<Item = char> + ExactSizeIterator + Clone + '_ {
        self.input_queue_characters
            .as_slice()
            .iter()
            // TODO: are the values in the buffer guaranteed to be valid unicode
            // scalar values? if so we can just expose this as a `&[char]`...
            .map(|c| core::char::from_u32(*c as u32).unwrap_or(core::char::REPLACEMENT_CHARACTER))
    }

    pub fn update_delta_time(&mut self, delta: Duration) {
        self.delta_time = delta.as_secs_f32().max(f32::MIN_POSITIVE);
    }

    pub fn add_mouse_pos_event(&mut self, pos: [f32; 2]) {
        unsafe {
            sys::ImGuiIO_AddMousePosEvent(self.raw_mut(), pos[0], pos[1]);
        }
    }

    pub fn add_mouse_button_event(&mut self, button: MouseButton, down: bool) {
        unsafe {
            sys::ImGuiIO_AddMouseButtonEvent(self.raw_mut(), button as i32, down);
        }
    }

    pub fn add_mouse_wheel_event(&mut self, wheel: [f32; 2]) {
        unsafe {
            sys::ImGuiIO_AddMouseWheelEvent(self.raw_mut(), wheel[0], wheel[1]);
        }
    }

    pub fn add_key_event(&mut self, key: Key, down: bool) {
        unsafe {
            sys::ImGuiIO_AddKeyEvent(self.raw_mut(), key as u32, down);
        }
    }

    /// Queue a gain/loss of focus for the application (generally based on OS/platform focus of your window)
    /// Note: [`io.config_debug_ignore_focus_loss`] will ignore this event from firing
    #[doc(alias = "AddFocusEvent")]
    pub fn add_focus_event(&mut self, focused: bool) {
        unsafe {
            sys::ImGuiIO_AddFocusEvent(self.raw_mut(), focused);
        }
    }

    pub fn add_key_analog_event(&mut self, key: Key, down: bool, value: f32) {
        unsafe {
            sys::ImGuiIO_AddKeyAnalogEvent(self.raw_mut(), key as u32, down, value);
        }
    }
}

impl Index<MouseButton> for Io {
    type Output = bool;
    fn index(&self, index: MouseButton) -> &bool {
        &self.mouse_down[index as usize]
    }
}

impl IndexMut<MouseButton> for Io {
    fn index_mut(&mut self, index: MouseButton) -> &mut bool {
        &mut self.mouse_down[index as usize]
    }
}

#[test]
#[cfg(test)]
fn test_io_memory_layout() {
    use std::mem;
    assert_eq!(mem::size_of::<Io>(), mem::size_of::<sys::ImGuiIO>());
    assert_eq!(mem::align_of::<Io>(), mem::align_of::<sys::ImGuiIO>());
    use sys::ImGuiIO;
    macro_rules! assert_field_offset {
        ($l:ident, $r:ident) => {
            assert_eq!(
                memoffset::offset_of!(Io, $l),
                memoffset::offset_of!(ImGuiIO, $r)
            );
        };
    }

    // We move this test into a Thread with a larger stack
    // since the stack size of the default thread is not large enough in
    // debug mode.
    std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(|| {
            assert_field_offset!(config_flags, ConfigFlags);
            assert_field_offset!(backend_flags, BackendFlags);
            assert_field_offset!(display_size, DisplaySize);
            assert_field_offset!(delta_time, DeltaTime);
            assert_field_offset!(ini_saving_rate, IniSavingRate);
            assert_field_offset!(ini_filename, IniFilename);
            assert_field_offset!(log_filename, LogFilename);
            assert_field_offset!(mouse_double_click_time, MouseDoubleClickTime);
            assert_field_offset!(mouse_double_click_max_dist, MouseDoubleClickMaxDist);
            assert_field_offset!(mouse_drag_threshold, MouseDragThreshold);
            assert_field_offset!(key_repeat_delay, KeyRepeatDelay);
            assert_field_offset!(key_repeat_rate, KeyRepeatRate);
            assert_field_offset!(user_data, UserData);
            assert_field_offset!(fonts, Fonts);
            assert_field_offset!(font_global_scale, FontGlobalScale);
            assert_field_offset!(font_allow_user_scaling, FontAllowUserScaling);
            assert_field_offset!(font_default, FontDefault);
            assert_field_offset!(display_framebuffer_scale, DisplayFramebufferScale);
            assert_field_offset!(mouse_draw_cursor, MouseDrawCursor);
            assert_field_offset!(config_mac_os_behaviors, ConfigMacOSXBehaviors);

            assert_field_offset!(config_error_recovery, ConfigErrorRecovery);

            assert_field_offset!(
                config_error_recovery_enable_assert,
                ConfigErrorRecoveryEnableAssert
            );

            assert_field_offset!(
                config_error_recovery_enable_debug_log,
                ConfigErrorRecoveryEnableDebugLog
            );

            assert_field_offset!(
                config_error_recovery_enable_tooltip,
                ConfigErrorRecoveryEnableTooltip
            );

            assert_field_offset!(
                config_input_trickle_event_queue,
                ConfigInputTrickleEventQueue
            );
            assert_field_offset!(config_input_text_cursor_blink, ConfigInputTextCursorBlink);
            assert_field_offset!(
                config_input_text_enter_keep_active,
                ConfigInputTextEnterKeepActive
            );
            assert_field_offset!(
                config_windows_resize_from_edges,
                ConfigWindowsResizeFromEdges
            );
            assert_field_offset!(
                config_windows_move_from_title_bar_only,
                ConfigWindowsMoveFromTitleBarOnly
            );
            assert_field_offset!(config_scrollbar_scroll_by_page, ConfigScrollbarScrollByPage);
            assert_field_offset!(backend_platform_name, BackendPlatformName);
            assert_field_offset!(backend_renderer_name, BackendRendererName);
            assert_field_offset!(backend_platform_user_data, BackendPlatformUserData);
            assert_field_offset!(backend_renderer_user_data, BackendRendererUserData);
            assert_field_offset!(backend_language_user_data, BackendLanguageUserData);
            assert_field_offset!(want_capture_mouse, WantCaptureMouse);
            assert_field_offset!(want_capture_keyboard, WantCaptureKeyboard);
            assert_field_offset!(want_text_input, WantTextInput);
            assert_field_offset!(want_set_mouse_pos, WantSetMousePos);
            assert_field_offset!(want_save_ini_settings, WantSaveIniSettings);
            assert_field_offset!(nav_active, NavActive);
            assert_field_offset!(nav_visible, NavVisible);
            assert_field_offset!(framerate, Framerate);
            assert_field_offset!(metrics_render_vertices, MetricsRenderVertices);
            assert_field_offset!(metrics_render_indices, MetricsRenderIndices);
            assert_field_offset!(metrics_render_windows, MetricsRenderWindows);
            assert_field_offset!(metrics_active_windows, MetricsActiveWindows);
            assert_field_offset!(mouse_delta, MouseDelta);
            assert_field_offset!(mouse_pos, MousePos);
            assert_field_offset!(mouse_down, MouseDown);
            assert_field_offset!(mouse_wheel, MouseWheel);
            assert_field_offset!(mouse_wheel_h, MouseWheelH);
            assert_field_offset!(mouse_source, MouseSource);
            assert_field_offset!(key_ctrl, KeyCtrl);
            assert_field_offset!(key_shift, KeyShift);
            assert_field_offset!(key_alt, KeyAlt);
            assert_field_offset!(key_super, KeySuper);
            assert_field_offset!(key_mods, KeyMods);
            assert_field_offset!(keys_data, KeysData);
            assert_field_offset!(
                want_capture_mouse_unless_popup_close,
                WantCaptureMouseUnlessPopupClose
            );
            assert_field_offset!(mouse_pos_prev, MousePosPrev);
            assert_field_offset!(mouse_clicked_pos, MouseClickedPos);
            assert_field_offset!(mouse_clicked_time, MouseClickedTime);
            assert_field_offset!(mouse_clicked, MouseClicked);
            assert_field_offset!(mouse_double_clicked, MouseDoubleClicked);
            assert_field_offset!(mouse_clicked_count, MouseClickedCount);
            assert_field_offset!(mouse_clicked_last_count, MouseClickedLastCount);
            assert_field_offset!(mouse_released, MouseReleased);
            #[cfg(not(feature = "docking"))]
            assert_field_offset!(mouse_released_time, MouseReleasedTime);
            assert_field_offset!(mouse_down_owned, MouseDownOwned);
            assert_field_offset!(
                mouse_down_owned_unless_popup_close,
                MouseDownOwnedUnlessPopupClose
            );
            assert_field_offset!(mouse_wheel_request_axis_swap, MouseWheelRequestAxisSwap);
            assert_field_offset!(mouse_ctrl_left_as_right_click, MouseCtrlLeftAsRightClick);
            assert_field_offset!(mouse_down_duration, MouseDownDuration);
            assert_field_offset!(mouse_down_duration_prev, MouseDownDurationPrev);
            assert_field_offset!(mouse_drag_max_distance_sqr, MouseDragMaxDistanceSqr);
            assert_field_offset!(pen_pressure, PenPressure);
            assert_field_offset!(app_focus_lost, AppFocusLost);
            assert_field_offset!(app_accepting_events, AppAcceptingEvents);
            assert_field_offset!(input_queue_surrogate, InputQueueSurrogate);
            assert_field_offset!(input_queue_characters, InputQueueCharacters);

            #[cfg(not(feature = "docking"))]
            {
                assert_field_offset!(display_framebuffer_scale, DisplayFramebufferScale);
                assert_field_offset!(config_nav_swap_gamepad_buttons, ConfigNavSwapGamepadButtons);
                assert_field_offset!(config_nav_move_set_mouse_pos, ConfigNavMoveSetMousePos);
                assert_field_offset!(config_nav_capture_keyboard, ConfigNavCaptureKeyboard);
                assert_field_offset!(
                    config_nav_escape_clear_focus_item,
                    ConfigNavEscapeClearFocusItem
                );
                assert_field_offset!(
                    config_nav_escape_clear_focus_window,
                    ConfigNavEscapeClearFocusWindow
                );
                assert_field_offset!(config_nav_cursor_visible_auto, ConfigNavCursorVisibleAuto);
                assert_field_offset!(
                    config_nav_cursor_visible_always,
                    ConfigNavCursorVisibleAlways
                );
                assert_field_offset!(config_drag_click_to_input_text, ConfigDragClickToInputText);
                assert_field_offset!(
                    config_windows_copy_contents_with_ctrl_c,
                    ConfigWindowsCopyContentsWithCtrlC
                );
                assert_field_offset!(
                    config_debug_highlight_id_conflicts_show_item_picker,
                    ConfigDebugHighlightIdConflictsShowItemPicker
                );
                assert_field_offset!(config_debug_ignore_focus_loss, ConfigDebugIgnoreFocusLoss);
                assert_field_offset!(config_debug_ini_settings, ConfigDebugIniSettings);
                assert_field_offset!(font_global_scale, FontGlobalScale);
                assert_field_offset!(get_clipboard_text_fn, GetClipboardTextFn);
                assert_field_offset!(set_clipboard_text_fn, SetClipboardTextFn);
                assert_field_offset!(clipboard_user_data, ClipboardUserData);
            }

            #[cfg(feature = "docking")]
            {
                assert_field_offset!(mouse_hovered_viewport, MouseHoveredViewport);
                assert_field_offset!(config_docking_no_split, ConfigDockingNoSplit);
                assert_field_offset!(config_docking_with_shift, ConfigDockingWithShift);
                assert_field_offset!(config_docking_always_tab_bar, ConfigDockingAlwaysTabBar);
                assert_field_offset!(
                    config_docking_transparent_payload,
                    ConfigDockingTransparentPayload
                );
                assert_field_offset!(config_viewports_no_auto_merge, ConfigViewportsNoAutoMerge);
                assert_field_offset!(
                    config_viewports_no_task_bar_icon,
                    ConfigViewportsNoTaskBarIcon
                );
                assert_field_offset!(config_viewports_no_decoration, ConfigViewportsNoDecoration);
                assert_field_offset!(
                    config_viewports_no_default_parent,
                    ConfigViewportsNoDefaultParent
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
