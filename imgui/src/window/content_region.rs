use crate::sys;
use crate::Ui;

/// # Content region
impl<'ui> Ui<'ui> {
    /// Equal to `ui.content_region_max()` - `ui.cursor_pos()`
    #[doc(alias = "GetContentRegionAvail")]
    pub fn content_region_avail(&self) -> [f32; 2] {
        let v = unsafe { sys::igGetContentRegionAvail() };
        [v.x, v.y]
    }
}
