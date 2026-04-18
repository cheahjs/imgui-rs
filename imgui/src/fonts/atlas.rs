use bitflags::bitflags;
use std::cell;
use std::f32;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::slice;

use crate::fonts::font::Font;
use crate::fonts::glyph_ranges::FontGlyphRanges;
use crate::internal::RawCast;
use crate::sys;

bitflags! {
    /// Font atlas configuration flags
    #[repr(transparent)]
    pub struct FontAtlasFlags: u32 {
        /// Don't round the height to next power of two
        const NO_POWER_OF_TWO_HEIGHT = sys::ImFontAtlasFlags_NoPowerOfTwoHeight;
        /// Don't build software mouse cursors into the atlas
        const NO_MOUSE_CURSORS = sys::ImFontAtlasFlags_NoMouseCursors;
        /// Don't build thick line textures into the atlas
        const NO_BAKED_LINES = sys::ImFontAtlasFlags_NoBakedLines;
    }
}

/// A font identifier
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FontId(pub(crate) *const Font);

/// Transparent newtype wrapper around [`sys::ImFontAtlas`] — layout matches by construction,
/// so the atlas can safely cross the DLL boundary to arcdps.
///
/// imgui 1.92 made fonts "dynamic" (rasterized on demand, uploaded by the backend via
/// `ImTextureData`). Several legacy APIs were removed:
/// - `GetTexDataAsAlpha8` / `GetTexDataAsRGBA32` (raw pixel data was replaced by `TexRef`)
/// - `IsBuilt`
/// - `GetGlyphRanges*` helpers (moved to per-source config)
///
/// Those methods are omitted from this wrapper. For low-level access, field the underlying
/// `sys::ImFontAtlas` via `Deref`.
#[repr(transparent)]
pub struct FontAtlas(pub sys::ImFontAtlas);

unsafe impl RawCast<sys::ImFontAtlas> for FontAtlas {}

impl Deref for FontAtlas {
    type Target = sys::ImFontAtlas;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FontAtlas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FontAtlas {
    #[doc(alias = "AddFontDefault", alias = "AddFont")]
    pub fn add_font(&mut self, font_sources: &[FontSource<'_>]) -> FontId {
        let (head, tail) = font_sources.split_first().unwrap();
        let font_id = self.add_font_internal(head, false);
        for font in tail {
            self.add_font_internal(font, true);
        }
        font_id
    }

    fn add_font_internal(&mut self, font_source: &FontSource<'_>, merge_mode: bool) -> FontId {
        let mut raw_config = sys_font_config_default();
        raw_config.MergeMode = merge_mode;
        let raw_font = match font_source {
            FontSource::DefaultFontData { config } => unsafe {
                if let Some(config) = config {
                    config.apply_to_raw_config(&mut raw_config, &mut self.0);
                }
                sys::ImFontAtlas_AddFontDefault(&mut self.0, &raw_config)
            },
            FontSource::TtfData {
                data,
                size_pixels,
                config,
            } => unsafe {
                if let Some(config) = config {
                    config.apply_to_raw_config(&mut raw_config, &mut self.0);
                }
                let data_copy = {
                    let ptr = sys::igMemAlloc(data.len()) as *mut u8;
                    assert!(!ptr.is_null());
                    slice::from_raw_parts_mut(ptr, data.len())
                };
                data_copy.copy_from_slice(data);
                raw_config.FontData = data_copy.as_mut_ptr() as *mut c_void;
                raw_config.FontDataSize = data_copy.len() as i32;
                raw_config.FontDataOwnedByAtlas = true;
                raw_config.SizePixels = *size_pixels;
                sys::ImFontAtlas_AddFont(&mut self.0, &raw_config)
            }
        };
        FontId(raw_font as *const _)
    }

    pub fn fonts(&self) -> Vec<FontId> {
        let mut result = Vec::new();
        let fonts_vec = &self.0.Fonts;
        if fonts_vec.Size > 0 {
            let slice =
                unsafe { slice::from_raw_parts(fonts_vec.Data, fonts_vec.Size as usize) };
            for &font in slice {
                result.push(FontId(font as *const _));
            }
        }
        result
    }

    pub fn get_font(&self, id: FontId) -> Option<&Font> {
        let fonts_vec = &self.0.Fonts;
        if fonts_vec.Size <= 0 {
            return None;
        }
        let slice =
            unsafe { slice::from_raw_parts(fonts_vec.Data, fonts_vec.Size as usize) };
        for &font in slice {
            if id == FontId(font as *const _) {
                return Some(unsafe { &*(font as *const Font) });
            }
        }
        None
    }

    /// Clears the font atlas completely (both input and output data)
    #[doc(alias = "Clear")]
    pub fn clear(&mut self) {
        unsafe { sys::ImFontAtlas_Clear(&mut self.0) };
    }

    /// Clears output font data (glyph storage, UV coordinates)
    #[doc(alias = "ClearFonts")]
    pub fn clear_fonts(&mut self) {
        unsafe { sys::ImFontAtlas_ClearFonts(&mut self.0) };
    }

    /// Clears output texture data.
    #[doc(alias = "ClearTexData")]
    pub fn clear_tex_data(&mut self) {
        unsafe { sys::ImFontAtlas_ClearTexData(&mut self.0) };
    }

    /// Clears all the data used to build the textures and fonts
    #[doc(alias = "ClearInputData")]
    pub fn clear_input_data(&mut self) {
        unsafe { sys::ImFontAtlas_ClearInputData(&mut self.0) };
    }
}

#[test]
#[cfg(test)]
fn test_font_atlas_layout_matches_sys() {
    use std::mem;
    assert_eq!(
        mem::size_of::<FontAtlas>(),
        mem::size_of::<sys::ImFontAtlas>()
    );
    assert_eq!(
        mem::align_of::<FontAtlas>(),
        mem::align_of::<sys::ImFontAtlas>()
    );
}

/// A source for binary font data
#[derive(Clone, Debug)]
pub enum FontSource<'a> {
    /// Default font included with the library (ProggyClean.ttf)
    DefaultFontData { config: Option<FontConfig> },
    /// Binary TTF/OTF font data
    TtfData {
        data: &'a [u8],
        size_pixels: f32,
        config: Option<FontConfig>,
    },
}

/// Configuration settings for a font
#[derive(Clone, Debug)]
pub struct FontConfig {
    /// Size in pixels for the rasterizer
    pub size_pixels: f32,
    /// Horizontal oversampling
    pub oversample_h: i32,
    /// Vertical oversampling
    pub oversample_v: i32,
    /// Align every glyph to pixel boundary
    pub pixel_snap_h: bool,
    /// Extra horizontal advance (in pixels) added per glyph.
    ///
    /// imgui 1.92 replaced the old `GlyphExtraSpacing` [x, y] pair with a scalar
    /// `GlyphExtraAdvanceX`. We keep the field as [f32; 2] for API compatibility;
    /// the Y component is ignored.
    pub glyph_extra_spacing: [f32; 2],
    /// Offset for all glyphs in this font
    pub glyph_offset: [f32; 2],
    /// Unicode ranges to use from this font
    pub glyph_ranges: FontGlyphRanges,
    /// Minimum advance_x for glyphs
    pub glyph_min_advance_x: f32,
    /// Maximum advance_x for glyphs
    pub glyph_max_advance_x: f32,
    /// Settings for a custom font rasterizer if used.
    ///
    /// imgui 1.92 removed the shared `RasterizerFlags` field; FreeType-specific flags are now
    /// routed through `FontLoaderFlags`. This field is forwarded to `FontLoaderFlags` as a
    /// compatibility shim.
    pub rasterizer_flags: u32,
    /// Brighten (>1.0) or darken (<1.0) font output
    pub rasterizer_multiply: f32,
    /// Explicitly specify the ellipsis character.
    pub ellipsis_char: Option<char>,
    pub name: Option<String>,
}

impl Default for FontConfig {
    fn default() -> FontConfig {
        FontConfig {
            size_pixels: 0.0,
            oversample_h: 3,
            oversample_v: 1,
            pixel_snap_h: false,
            glyph_extra_spacing: [0.0, 0.0],
            glyph_offset: [0.0, 0.0],
            glyph_ranges: FontGlyphRanges::default(),
            glyph_min_advance_x: 0.0,
            glyph_max_advance_x: f32::MAX,
            rasterizer_flags: 0,
            rasterizer_multiply: 1.0,
            ellipsis_char: None,
            name: None,
        }
    }
}

impl FontConfig {
    fn apply_to_raw_config(&self, raw: &mut sys::ImFontConfig, atlas: *mut sys::ImFontAtlas) {
        raw.SizePixels = self.size_pixels;
        raw.OversampleH = self.oversample_h as i8;
        raw.OversampleV = self.oversample_v as i8;
        raw.PixelSnapH = self.pixel_snap_h;
        // imgui 1.92: GlyphExtraSpacing (ImVec2) -> GlyphExtraAdvanceX (f32), Y dropped.
        raw.GlyphExtraAdvanceX = self.glyph_extra_spacing[0];
        raw.GlyphOffset = sys::ImVec2 {
            x: self.glyph_offset[0],
            y: self.glyph_offset[1],
        };
        raw.GlyphRanges = unsafe { self.glyph_ranges.to_ptr(atlas) };
        raw.GlyphMinAdvanceX = self.glyph_min_advance_x;
        raw.GlyphMaxAdvanceX = self.glyph_max_advance_x;
        // imgui 1.92: RasterizerFlags replaced by FontLoaderFlags.
        raw.FontLoaderFlags = self.rasterizer_flags;
        raw.RasterizerMultiply = self.rasterizer_multiply;
        raw.EllipsisChar = self.ellipsis_char.map(|x| x as u16).unwrap_or(0xffff);
        if let Some(name) = self.name.as_ref() {
            let bytes = name.as_bytes();
            let mut len = bytes.len().min(raw.Name.len() - 1);
            while !name.is_char_boundary(len) {
                len -= 1;
            }
            unsafe {
                bytes.as_ptr().copy_to(raw.Name.as_mut_ptr() as _, len);
                raw.Name[len] = 0;
            }
        }
    }
}

fn sys_font_config_default() -> sys::ImFontConfig {
    unsafe {
        let heap_allocated = sys::ImFontConfig_ImFontConfig();
        let copy = *heap_allocated;
        sys::ImFontConfig_destroy(heap_allocated);
        copy
    }
}

#[test]
fn test_font_config_default() {
    let sys_font_config = sys_font_config_default();
    let font_config = FontConfig::default();
    assert_eq!(font_config.size_pixels, sys_font_config.SizePixels);
    assert_eq!(
        font_config.oversample_h as i8,
        sys_font_config.OversampleH
    );
    assert_eq!(
        font_config.oversample_v as i8,
        sys_font_config.OversampleV
    );
    assert_eq!(font_config.pixel_snap_h, sys_font_config.PixelSnapH);
    assert_eq!(font_config.glyph_offset[0], sys_font_config.GlyphOffset.x);
    assert_eq!(font_config.glyph_offset[1], sys_font_config.GlyphOffset.y);
    assert_eq!(
        font_config.glyph_min_advance_x,
        sys_font_config.GlyphMinAdvanceX
    );
    assert_eq!(
        font_config.glyph_max_advance_x,
        sys_font_config.GlyphMaxAdvanceX
    );
    assert_eq!(
        font_config.rasterizer_multiply,
        sys_font_config.RasterizerMultiply
    );
}

/// Handle to a font atlas texture
#[derive(Clone, Debug)]
pub struct FontAtlasTexture<'a> {
    /// Texture width (in pixels)
    pub width: u32,
    /// Texture height (in pixels)
    pub height: u32,
    /// Raw texture data (in bytes).
    pub data: &'a [u8],
}

/// A font atlas that can be shared between contexts
#[derive(Debug)]
pub struct SharedFontAtlas(pub(crate) *mut sys::ImFontAtlas);

impl SharedFontAtlas {
    #[doc(alias = "ImFontAtlas", alias = "ImFontAtlas::ImFontAtlas")]
    pub fn create() -> SharedFontAtlas {
        SharedFontAtlas(unsafe { sys::ImFontAtlas_ImFontAtlas() })
    }
}

impl Drop for SharedFontAtlas {
    #[doc(alias = "ImFontAtlas::Destory")]
    fn drop(&mut self) {
        unsafe { sys::ImFontAtlas_destroy(self.0) };
    }
}

impl Deref for SharedFontAtlas {
    type Target = FontAtlas;
    fn deref(&self) -> &FontAtlas {
        unsafe { &*(self.0 as *const FontAtlas) }
    }
}

impl DerefMut for SharedFontAtlas {
    fn deref_mut(&mut self) -> &mut FontAtlas {
        unsafe { &mut *(self.0 as *mut FontAtlas) }
    }
}

/// An immutably borrowed reference to a (possibly shared) font atlas
pub enum FontAtlasRef<'a> {
    Owned(&'a FontAtlas),
    Shared(&'a cell::RefMut<'a, SharedFontAtlas>),
}

impl<'a> Deref for FontAtlasRef<'a> {
    type Target = FontAtlas;
    fn deref(&self) -> &FontAtlas {
        use self::FontAtlasRef::*;
        match self {
            Owned(atlas) => atlas,
            Shared(cell) => cell,
        }
    }
}

/// A mutably borrowed reference to a (possibly shared) font atlas
pub enum FontAtlasRefMut<'a> {
    Owned(&'a mut FontAtlas),
    Shared(cell::RefMut<'a, SharedFontAtlas>),
}

impl<'a> Deref for FontAtlasRefMut<'a> {
    type Target = FontAtlas;
    fn deref(&self) -> &FontAtlas {
        use self::FontAtlasRefMut::*;
        match self {
            Owned(atlas) => atlas,
            Shared(cell) => cell,
        }
    }
}

impl<'a> DerefMut for FontAtlasRefMut<'a> {
    fn deref_mut(&mut self) -> &mut FontAtlas {
        use self::FontAtlasRefMut::*;
        match self {
            Owned(atlas) => atlas,
            Shared(cell) => cell,
        }
    }
}
