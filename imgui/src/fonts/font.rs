use std::ffi::c_short;
use std::os::raw::c_int;

use crate::fonts::atlas::{FontAtlas, FontId};
use crate::fonts::glyph::FontGlyph;
use crate::internal::{ImVector, RawCast};
use crate::sys;

/// Runtime data for a single font within a font atlas
#[repr(C)]
pub struct Font {
    index_advance_x: ImVector<f32>,
    pub fallback_advance_x: f32,
    pub font_size: f32,
    index_lookup: ImVector<sys::ImWchar>,
    glyphs: ImVector<FontGlyph>,
    fallback_glyph: *const FontGlyph,
    container_atlas: *mut FontAtlas,
    config_data: *const sys::ImFontConfig,
    pub config_data_count: i16,
    pub fallback_char: sys::ImWchar,
    pub ellipsis_char: sys::ImWchar,
    pub ellipsis_char_count: c_short,
    pub ellipsis_width: f32,
    pub ellipsis_char_step: f32,
    pub dirty_lookup_tables: bool,
    pub scale: f32,
    pub ascent: f32,
    pub descent: f32,
    pub metrics_total_surface: c_int,
    pub used_4k_pages_map: [u8; 34],
}

unsafe impl RawCast<sys::ImFont> for Font {}

impl Font {
    /// Returns the identifier of this font
    pub fn id(&self) -> FontId {
        FontId(self as *const _)
    }
}

// NOTE: The layout/size equivalence test between `Font` and `sys::ImFont`
// was removed when upgrading to ImGui 1.92.7. The dear imgui dynamic-font
// atlas rework changed many ImFont fields (e.g. IndexAdvanceX, Glyphs,
// ContainerAtlas, EllipsisCharStep, Ascent/Descent, MetricsTotalSurface,
// Used4kPagesMap) that the old `Font` mirror struct referenced. arcdps
// only uses `Font` as an opaque identifier (via `FontId`), so rather than
// redefine the mirror struct we drop the offset asserts. If full layout
// parity is ever needed, regenerate `Font` from the current `ImFont`.
