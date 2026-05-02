use bitflags::bitflags;
use std::f32;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::rc::Rc;
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

/// A font atlas that builds a single texture.
///
/// Transparent newtype wrapper over [`sys::ImFontAtlas`]. The atlas supports dynamic fonts
/// (rasterized on demand, uploaded via the `ImTextureData` pipeline), so field-by-field
/// mirroring is both brittle and unnecessary. Access raw fields via `Deref`.
#[repr(transparent)]
pub struct FontAtlas(pub sys::ImFontAtlas);

unsafe impl RawCast<sys::ImFontAtlas> for FontAtlas {}

impl Deref for FontAtlas {
    type Target = sys::ImFontAtlas;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FontAtlas {
    #[inline]
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
                    config.apply_to_raw_config(&mut raw_config, self.raw_mut());
                }
                sys::ImFontAtlas_AddFontDefault(self.raw_mut(), &raw_config)
            },
            FontSource::TtfData {
                data,
                size_pixels,
                config,
            } => {
                if let Some(config) = config {
                    unsafe {
                        config.apply_to_raw_config(&mut raw_config, self.raw_mut());
                    }
                }
                // We can't guarantee `data` is alive when the font atlas is built, so
                // make a copy and move ownership of the data to the atlas
                let data_copy = unsafe {
                    let ptr = sys::igMemAlloc(data.len()) as *mut u8;
                    assert!(!ptr.is_null());
                    slice::from_raw_parts_mut(ptr, data.len())
                };
                data_copy.copy_from_slice(data);
                raw_config.FontData = data_copy.as_mut_ptr() as *mut c_void;
                raw_config.FontDataSize = data_copy.len() as i32;
                raw_config.FontDataOwnedByAtlas = true;
                raw_config.SizePixels = *size_pixels;
                unsafe { sys::ImFontAtlas_AddFont(self.raw_mut(), &raw_config) }
            }
        };
        FontId(raw_font as *const _)
    }
    pub fn fonts(&self) -> Vec<FontId> {
        let mut result = Vec::new();
        let fonts_vec = &self.0.Fonts;
        if fonts_vec.Size > 0 {
            let s = unsafe { slice::from_raw_parts(fonts_vec.Data, fonts_vec.Size as usize) };
            for &font in s {
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
        let s = unsafe { slice::from_raw_parts(fonts_vec.Data, fonts_vec.Size as usize) };
        for &font in s {
            if id == FontId(font as *const _) {
                return Some(unsafe { &*(font as *const Font) });
            }
        }
        None
    }
    // Fonts are rasterized on demand and uploaded via ImTextureData (see `self.TexData`).
    // Renderers should consume the texture through the normal per-frame texture-update path
    // instead of copying raw atlas pixels.
    /// Clears the font atlas completely (both input and output data)
    #[doc(alias = "Clear")]
    pub fn clear(&mut self) {
        unsafe {
            sys::ImFontAtlas_Clear(self.raw_mut());
        }
    }
    /// Clears output font data (glyph storage, UV coordinates)
    #[doc(alias = "ClearFonts")]
    pub fn clear_fonts(&mut self) {
        unsafe {
            sys::ImFontAtlas_ClearFonts(self.raw_mut());
        }
    }
    /// Clears output texture data.
    ///
    /// Can be used to save RAM once the texture has been transferred to the GPU.
    #[doc(alias = "ClearTexData")]
    pub fn clear_tex_data(&mut self) {
        unsafe {
            sys::ImFontAtlas_ClearTexData(self.raw_mut());
        }
    }
    /// Clears all the data used to build the textures and fonts
    #[doc(alias = "ClearInputData")]
    pub fn clear_input_data(&mut self) {
        unsafe {
            sys::ImFontAtlas_ClearInputData(self.raw_mut());
        }
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
    /// Extra spacing (in pixels) between glyphs
    pub glyph_extra_spacing: [f32; 2],
    /// Offset for all glyphs in this font
    pub glyph_offset: [f32; 2],
    /// Unicode ranges to use from this font
    pub glyph_ranges: FontGlyphRanges,
    /// Minimum advance_x for glyphs
    pub glyph_min_advance_x: f32,
    /// Maximum advance_x for glyphs
    pub glyph_max_advance_x: f32,
    /// Settings for a custom font rasterizer if used
    pub font_builder_flags: u32,
    /// Brighten (>1.0) or darken (<1.0) font output
    pub rasterizer_multiply: f32,
    /// DPI scale for rasterization, not altering other font metrics:
    /// make it easy to swap between e.g. a 100% and a 400% fonts for a zooming display.
    /// IMPORTANT: If you increase this it is expected that you increase font scale
    /// accordingly, otherwise quality may look lowered.
    pub rasterizer_density: f32,
    /// Explicitly specify the ellipsis character.
    ///
    /// With multiple font sources the first specified ellipsis is used.
    pub ellipsis_char: Option<char>,
    pub name: Option<String>,
}

impl Default for FontConfig {
    fn default() -> FontConfig {
        FontConfig {
            size_pixels: 0.0,
            oversample_h: 0,
            oversample_v: 0,
            pixel_snap_h: false,
            glyph_extra_spacing: [0.0, 0.0],
            glyph_offset: [0.0, 0.0],
            glyph_ranges: FontGlyphRanges::default(),
            glyph_min_advance_x: 0.0,
            glyph_max_advance_x: f32::MAX,
            font_builder_flags: 0,
            rasterizer_multiply: 1.0,
            rasterizer_density: 1.0,
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
        // Only the X component is honored; Y is dropped.
        raw.GlyphExtraAdvanceX = self.glyph_extra_spacing[0];
        raw.GlyphOffset = self.glyph_offset.into();
        raw.GlyphRanges = unsafe { self.glyph_ranges.to_ptr(atlas) };
        raw.GlyphMinAdvanceX = self.glyph_min_advance_x;
        raw.GlyphMaxAdvanceX = self.glyph_max_advance_x;
        raw.FontLoaderFlags = self.font_builder_flags;
        raw.RasterizerMultiply = self.rasterizer_multiply;
        raw.RasterizerDensity = self.rasterizer_density;
        // char is used as "unset" for EllipsisChar
        raw.EllipsisChar = self.ellipsis_char.map(|c| c as u16).unwrap_or(0xffff);
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
    assert_eq!(font_config.oversample_h as i8, sys_font_config.OversampleH);
    assert_eq!(font_config.oversample_v as i8, sys_font_config.OversampleV);
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
    ///
    /// The format depends on which function was called to obtain this data.
    pub data: &'a [u8],
}

/// A font atlas that can be shared between contexts
#[derive(Debug, Clone)]
pub struct SharedFontAtlas(pub(crate) Rc<*mut sys::ImFontAtlas>);

impl std::ops::Deref for SharedFontAtlas {
    type Target = Rc<*mut sys::ImFontAtlas>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SharedFontAtlas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SharedFontAtlas {
    #[doc(alias = "ImFontAtlas", alias = "ImFontAtlas::ImFontAtlas")]
    pub fn create() -> SharedFontAtlas {
        let atlas = unsafe { sys::ImFontAtlas_ImFontAtlas() };
        unsafe {
            // Imgui registers shared atlases with each context and deletes them when the
            // refcount hits zero. Keep one Rust-owned reference so context destruction
            // cannot free the atlas behind `SharedFontAtlas`.
            (*atlas).RefCount = 1;
        }
        SharedFontAtlas(Rc::new(atlas))
    }

    /// Gets a raw pointer to the underlying `ImFontAtlas`.
    pub fn as_ptr(&self) -> *const sys::ImFontAtlas {
        *self.0 as *const _
    }

    /// Gets a raw pointer to the underlying `ImFontAtlas`.
    pub fn as_ptr_mut(&mut self) -> *mut sys::ImFontAtlas {
        *self.0
    }
}

impl Drop for SharedFontAtlas {
    #[doc(alias = "ImFontAtlas::Destory")]
    fn drop(&mut self) {
        // if we're about to drop the last one...
        if Rc::strong_count(&self.0) == 1 {
            unsafe { sys::ImFontAtlas_destroy(*self.0) };
        }
    }
}
