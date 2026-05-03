use crate::fonts::atlas::FontId;
use crate::internal::RawCast;
use crate::sys;

/// Runtime data for a single font within a font atlas.
///
/// Treated as opaque (transparent wrapper around `sys::ImFont`); arcdps consumers
/// only need this as a stable identifier (see [`Font::id`]).
#[repr(transparent)]
pub struct Font(sys::ImFont);

unsafe impl RawCast<sys::ImFont> for Font {}

impl Font {
    /// Returns the identifier of this font
    pub fn id(&self) -> FontId {
        FontId(self as *const _)
    }
}
