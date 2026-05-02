use std::{marker::PhantomData, slice};

use crate::internal::{ImVector, RawCast, RawWrapper};
use crate::math::MintVec2;
use crate::render::renderer::TextureId;
use crate::sys;

/// All draw data to render a Dear ImGui frame.
#[repr(C)]
pub struct DrawData {
    /// Only valid after render() is called and before the next new frame() is called.
    valid: bool,
    /// Number of DrawList to render.
    cmd_lists_count: i32,
    /// For convenience, sum of all draw list index buffer sizes.
    pub total_idx_count: i32,
    /// For convenience, sum of all draw list vertex buffer sizes.
    pub total_vtx_count: i32,
    // Array of DrawList.
    cmd_lists: ImVector<DrawList>,
    /// Upper-left position of the viewport to render.
    ///
    /// (= upper-left corner of the orthogonal projection matrix to use)
    pub display_pos: [f32; 2],
    /// Size of the viewport to render.
    ///
    /// (= display_pos + display_size == lower-right corner of the orthogonal matrix to use)
    pub display_size: [f32; 2],
    /// Amount of pixels for each unit of display_size.
    ///
    /// Based on io.display_frame_buffer_scale. Typically [1.0, 1.0] on normal displays, and
    /// [2.0, 2.0] on Retina displays, but fractional values are also possible.
    pub framebuffer_scale: [f32; 2],

    /// Viewport carrying the DrawData instance, might be of use to the renderer (generally not).
    owner_viewport: *mut sys::ImGuiViewport,

    /// Textures referenced by the draw lists. Owned by the imgui context.
    textures: *mut sys::ImVector_ImTextureDataPtr,
}

unsafe impl RawCast<sys::ImDrawData> for DrawData {}

impl DrawData {
    /// Returns an iterator over the draw lists included in the draw data.
    #[inline]
    pub fn draw_lists(&self) -> DrawListIterator<'_> {
        unsafe {
            DrawListIterator {
                iter: self.cmd_lists().iter(),
            }
        }
    }
    /// Returns the number of draw lists included in the draw data.
    #[inline]
    pub fn draw_lists_count(&self) -> usize {
        self.cmd_lists_count.try_into().unwrap()
    }
    /// Returns texture update requests referenced by this draw data.
    ///
    /// Renderers that set [`BackendFlags::RENDERER_HAS_TEXTURES`](crate::BackendFlags::RENDERER_HAS_TEXTURES)
    /// should process these requests before drawing and update each texture's status.
    #[cfg(not(feature = "docking"))]
    #[inline]
    pub fn textures(&self) -> TextureDataIterator<'_> {
        unsafe { TextureDataIterator::from_raw_vector(self.textures) }
    }
    #[inline]
    pub(crate) unsafe fn cmd_lists(&self) -> &[*const DrawList] {
        if self.cmd_lists_count <= 0 || self.cmd_lists.data.is_null() {
            return &[];
        }
        slice::from_raw_parts(
            self.cmd_lists.data as *const *const DrawList,
            self.cmd_lists_count as usize,
        )
    }
    /// Converts all buffers from indexed to non-indexed, in case you cannot render indexed
    /// buffers.
    ///
    /// **This is slow and most likely a waste of resources. Always prefer indexed rendering!**
    #[doc(alias = "DeIndexAllBuffers")]
    pub fn deindex_all_buffers(&mut self) {
        unsafe {
            sys::ImDrawData_DeIndexAllBuffers(self.raw_mut());
        }
    }
    /// Scales the clip rect of each draw command.
    ///
    /// Can be used if your final output buffer is at a different scale than imgui-rs expects, or
    /// if there is a difference between your window resolution and framebuffer resolution.
    #[doc(alias = "ScaleClipRects")]
    pub fn scale_clip_rects(&mut self, fb_scale: MintVec2) {
        unsafe {
            sys::ImDrawData_ScaleClipRects(self.raw_mut(), fb_scale.into());
        }
    }
}

#[cfg(not(feature = "docking"))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextureStatus {
    Ok,
    Destroyed,
    WantCreate,
    WantUpdates,
    WantDestroy,
}

#[cfg(not(feature = "docking"))]
impl TextureStatus {
    #[inline]
    fn from_raw(status: sys::ImTextureStatus) -> Self {
        match status {
            sys::ImTextureStatus_OK => Self::Ok,
            sys::ImTextureStatus_Destroyed => Self::Destroyed,
            sys::ImTextureStatus_WantCreate => Self::WantCreate,
            sys::ImTextureStatus_WantUpdates => Self::WantUpdates,
            sys::ImTextureStatus_WantDestroy => Self::WantDestroy,
            _ => panic!("unknown ImTextureStatus value: {status}"),
        }
    }

    #[inline]
    fn raw(self) -> sys::ImTextureStatus {
        match self {
            Self::Ok => sys::ImTextureStatus_OK,
            Self::Destroyed => sys::ImTextureStatus_Destroyed,
            Self::WantCreate => sys::ImTextureStatus_WantCreate,
            Self::WantUpdates => sys::ImTextureStatus_WantUpdates,
            Self::WantDestroy => sys::ImTextureStatus_WantDestroy,
        }
    }
}

#[cfg(not(feature = "docking"))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextureFormat {
    Rgba32,
    Alpha8,
}

#[cfg(not(feature = "docking"))]
impl TextureFormat {
    #[inline]
    fn from_raw(format: sys::ImTextureFormat) -> Self {
        match format {
            sys::ImTextureFormat_RGBA32 => Self::Rgba32,
            sys::ImTextureFormat_Alpha8 => Self::Alpha8,
            _ => panic!("unknown ImTextureFormat value: {format}"),
        }
    }
}

#[cfg(not(feature = "docking"))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TextureRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[cfg(not(feature = "docking"))]
impl From<sys::ImTextureRect> for TextureRect {
    #[inline]
    fn from(rect: sys::ImTextureRect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.w,
            height: rect.h,
        }
    }
}

/// A Dear ImGui texture update request.
#[cfg(not(feature = "docking"))]
#[derive(Copy, Clone)]
pub struct TextureData<'a> {
    raw: *mut sys::ImTextureData,
    _marker: PhantomData<&'a sys::ImTextureData>,
}

#[cfg(not(feature = "docking"))]
impl<'a> TextureData<'a> {
    #[inline]
    unsafe fn from_raw(raw: *mut sys::ImTextureData) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn raw(&self) -> &sys::ImTextureData {
        unsafe { &*self.raw }
    }

    #[inline]
    pub fn id(&self) -> i32 {
        self.raw().UniqueID
    }

    #[inline]
    pub fn status(&self) -> TextureStatus {
        TextureStatus::from_raw(self.raw().Status)
    }

    #[inline]
    pub fn set_status(&self, status: TextureStatus) {
        unsafe { sys::ImTextureData_SetStatus(self.raw, status.raw()) };
    }

    #[inline]
    pub fn texture_id(&self) -> TextureId {
        TextureId::from(unsafe { sys::ImTextureData_GetTexID(self.raw) as usize })
    }

    #[inline]
    pub fn set_texture_id(&self, texture_id: TextureId) {
        unsafe { sys::ImTextureData_SetTexID(self.raw, texture_id.id() as sys::ImTextureID) };
    }

    #[inline]
    pub fn format(&self) -> TextureFormat {
        TextureFormat::from_raw(self.raw().Format)
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.raw().Width.max(0) as usize
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.raw().Height.max(0) as usize
    }

    #[inline]
    pub fn bytes_per_pixel(&self) -> usize {
        self.raw().BytesPerPixel.max(0) as usize
    }

    #[inline]
    pub fn pitch(&self) -> usize {
        unsafe { sys::ImTextureData_GetPitch(self.raw).max(0) as usize }
    }

    #[inline]
    pub fn pixels(&self) -> &[u8] {
        let size = unsafe { sys::ImTextureData_GetSizeInBytes(self.raw) };
        if size <= 0 {
            return &[];
        }
        let pixels = unsafe { sys::ImTextureData_GetPixels(self.raw) as *const u8 };
        if pixels.is_null() {
            return &[];
        }
        unsafe { slice::from_raw_parts(pixels, size as usize) }
    }

    #[inline]
    pub fn update_rect(&self) -> TextureRect {
        self.raw().UpdateRect.into()
    }

    #[inline]
    pub fn updates(&self) -> TextureRectIterator<'_> {
        let updates = &self.raw().Updates;
        let iter = if updates.Size <= 0 || updates.Data.is_null() {
            [].iter()
        } else {
            unsafe { slice::from_raw_parts(updates.Data, updates.Size as usize) }.iter()
        };
        TextureRectIterator { iter }
    }

    #[inline]
    pub fn ref_count(&self) -> u16 {
        self.raw().RefCount
    }

    #[inline]
    pub fn uses_colors(&self) -> bool {
        self.raw().UseColors
    }
}

#[cfg(not(feature = "docking"))]
pub struct TextureRectIterator<'a> {
    iter: slice::Iter<'a, sys::ImTextureRect>,
}

#[cfg(not(feature = "docking"))]
impl Iterator for TextureRectIterator<'_> {
    type Item = TextureRect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied().map(TextureRect::from)
    }
}

#[cfg(not(feature = "docking"))]
pub struct TextureDataIterator<'a> {
    iter: slice::Iter<'a, *mut sys::ImTextureData>,
}

#[cfg(not(feature = "docking"))]
impl<'a> TextureDataIterator<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw_vector(vector: *const sys::ImVector_ImTextureDataPtr) -> Self {
        let textures = if vector.is_null() || (*vector).Size <= 0 || (*vector).Data.is_null() {
            &[]
        } else {
            slice::from_raw_parts((*vector).Data, (*vector).Size as usize)
        };
        Self {
            iter: textures.iter(),
        }
    }
}

#[cfg(not(feature = "docking"))]
impl<'a> Iterator for TextureDataIterator<'a> {
    type Item = TextureData<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .copied()
            .filter(|ptr| !ptr.is_null())
            .map(|ptr| unsafe { TextureData::from_raw(ptr) })
    }
}

/// Iterator over draw lists
pub struct DrawListIterator<'a> {
    iter: std::slice::Iter<'a, *const DrawList>,
}

impl<'a> Iterator for DrawListIterator<'a> {
    type Item = &'a DrawList;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&ptr| unsafe { &*ptr })
    }
}

#[test]
#[cfg(test)]
fn test_drawdata_memory_layout() {
    use std::mem;
    assert_eq!(
        mem::size_of::<DrawData>(),
        mem::size_of::<sys::ImDrawData>()
    );
    assert_eq!(
        mem::align_of::<DrawData>(),
        mem::align_of::<sys::ImDrawData>()
    );
    use sys::ImDrawData;
    macro_rules! assert_field_offset {
        ($l:ident, $r:ident) => {
            assert_eq!(
                memoffset::offset_of!(DrawData, $l),
                memoffset::offset_of!(ImDrawData, $r)
            );
        };
    }
    assert_field_offset!(valid, Valid);
    assert_field_offset!(cmd_lists, CmdLists);
    assert_field_offset!(cmd_lists_count, CmdListsCount);
    assert_field_offset!(total_idx_count, TotalIdxCount);
    assert_field_offset!(total_vtx_count, TotalVtxCount);
    assert_field_offset!(display_pos, DisplayPos);
    assert_field_offset!(display_size, DisplaySize);
    assert_field_offset!(framebuffer_scale, FramebufferScale);
    assert_field_offset!(owner_viewport, OwnerViewport);
    assert_field_offset!(textures, Textures);
}

/// Draw command list
#[repr(transparent)]
pub struct DrawList(sys::ImDrawList);

impl RawWrapper for DrawList {
    type Raw = sys::ImDrawList;
    #[inline]
    unsafe fn raw(&self) -> &sys::ImDrawList {
        &self.0
    }
    #[inline]
    unsafe fn raw_mut(&mut self) -> &mut sys::ImDrawList {
        &mut self.0
    }
}

impl DrawList {
    #[inline]
    pub(crate) unsafe fn cmd_buffer(&self) -> &[sys::ImDrawCmd] {
        if self.0.CmdBuffer.Size <= 0 || self.0.CmdBuffer.Data.is_null() {
            return &[];
        }

        slice::from_raw_parts(
            self.0.CmdBuffer.Data as *const sys::ImDrawCmd,
            self.0.CmdBuffer.Size as usize,
        )
    }

    #[inline]
    pub fn idx_buffer(&self) -> &[DrawIdx] {
        unsafe {
            if self.0.IdxBuffer.Size <= 0 || self.0.IdxBuffer.Data.is_null() {
                return &[];
            }

            slice::from_raw_parts(
                self.0.IdxBuffer.Data as *const DrawIdx,
                self.0.IdxBuffer.Size as usize,
            )
        }
    }

    #[inline]
    pub fn vtx_buffer(&self) -> &[DrawVert] {
        unsafe {
            if self.0.VtxBuffer.Size <= 0 || self.0.VtxBuffer.Data.is_null() {
                return &[];
            }

            slice::from_raw_parts(
                self.0.VtxBuffer.Data as *const DrawVert,
                self.0.VtxBuffer.Size as usize,
            )
        }
    }

    /// # Safety
    /// This is equivalent to `transmute(self.vtx_buffer())` with a little more
    /// checking, and thus inherits the safety considerations of `transmute`ing
    /// slices.
    pub unsafe fn transmute_vtx_buffer<VTy: Copy>(&self) -> &[VTy] {
        // these checks are constant and thus are removed from release builds
        assert_eq!(
            core::mem::size_of::<VTy>(),
            core::mem::size_of::<DrawVert>(),
        );
        assert!(core::mem::align_of::<VTy>() <= core::mem::align_of::<DrawVert>());
        if self.0.VtxBuffer.Size <= 0 || self.0.VtxBuffer.Data.is_null() {
            return &[];
        }

        slice::from_raw_parts(
            self.0.VtxBuffer.Data as *const VTy,
            self.0.VtxBuffer.Size as usize,
        )
    }

    #[inline]
    pub fn commands(&self) -> DrawCmdIterator<'_> {
        unsafe {
            DrawCmdIterator {
                iter: self.cmd_buffer().iter(),
            }
        }
    }
}

pub struct DrawCmdIterator<'a> {
    iter: std::slice::Iter<'a, sys::ImDrawCmd>,
}

impl Iterator for DrawCmdIterator<'_> {
    type Item = DrawCmd;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|cmd| {
            let texture_id = unsafe {
                TextureId::from(sys::ImDrawCmd_GetTexID(cmd as *const _ as *mut _) as usize)
            };
            let cmd_params = DrawCmdParams {
                clip_rect: cmd.ClipRect.into(),
                texture_id,
                vtx_offset: cmd.VtxOffset as usize,
                idx_offset: cmd.IdxOffset as usize,
            };
            match cmd.UserCallback {
                Some(raw_callback) if raw_callback as usize == -8isize as usize => {
                    DrawCmd::ResetRenderState
                }
                Some(raw_callback) => DrawCmd::RawCallback {
                    callback: raw_callback,
                    raw_cmd: cmd,
                },
                None => DrawCmd::Elements {
                    count: cmd.ElemCount as usize,
                    cmd_params,
                },
            }
        })
    }
}

#[test]
#[cfg(all(test, not(feature = "docking")))]
fn test_draw_cmd_texture_id_uses_tex_ref_data() {
    unsafe {
        let tex = sys::ImTextureData_ImTextureData();
        sys::ImTextureData_SetTexID(tex, 123);

        let cmd_ptr = sys::ImDrawCmd_ImDrawCmd();
        (*cmd_ptr).TexRef._TexData = tex;
        (*cmd_ptr).TexRef._TexID = 0;
        (*cmd_ptr).ElemCount = 1;

        let cmds = [*cmd_ptr];
        let mut iter = DrawCmdIterator { iter: cmds.iter() };
        match iter.next().unwrap() {
            DrawCmd::Elements { cmd_params, .. } => {
                assert_eq!(cmd_params.texture_id.id(), 123);
            }
            _ => panic!("expected elements draw command"),
        }

        sys::ImDrawCmd_destroy(cmd_ptr);
        sys::ImTextureData_destroy(tex);
    }
}

/// A vertex index
pub type DrawIdx = sys::ImDrawIdx;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DrawCmdParams {
    /// left, up, right, down
    pub clip_rect: [f32; 4],
    pub texture_id: TextureId,
    pub vtx_offset: usize,
    pub idx_offset: usize,
}

/// A draw command
pub enum DrawCmd {
    Elements {
        /// The number of indices used for this draw command
        count: usize,
        cmd_params: DrawCmdParams,
    },
    ResetRenderState,
    RawCallback {
        callback: unsafe extern "C" fn(*const sys::ImDrawList, cmd: *const sys::ImDrawCmd),
        raw_cmd: *const sys::ImDrawCmd,
    },
}

/// A single vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DrawVert {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [u8; 4],
}

#[test]
#[cfg(test)]
fn test_drawvert_memory_layout() {
    use std::mem;
    assert_eq!(
        mem::size_of::<DrawVert>(),
        mem::size_of::<sys::ImDrawVert>()
    );
    assert_eq!(
        mem::align_of::<DrawVert>(),
        mem::align_of::<sys::ImDrawVert>()
    );
    use sys::ImDrawVert;
    macro_rules! assert_field_offset {
        ($l:ident, $r:ident) => {
            assert_eq!(
                memoffset::offset_of!(DrawVert, $l),
                memoffset::offset_of!(ImDrawVert, $r)
            );
        };
    }
    assert_field_offset!(pos, pos);
    assert_field_offset!(uv, uv);
    assert_field_offset!(col, col);
}

/// A container for a heap-allocated deep copy of a `DrawData` struct.
///
/// Can be used to retain draw data for rendering on a different thread.
/// The underlying copy is released when this struct is dropped.
pub struct OwnedDrawData {
    draw_data: *mut sys::ImDrawData,
}

impl OwnedDrawData {
    /// If this struct contains a `DrawData` object, then this function returns a reference to it.
    ///
    /// Otherwise, this struct is empty and so this function returns `None`.
    #[inline]
    pub fn draw_data(&self) -> Option<&DrawData> {
        if !self.draw_data.is_null() {
            Some(unsafe { DrawData::from_raw(&*self.draw_data) })
        } else {
            None
        }
    }
}

impl Default for OwnedDrawData {
    /// The default `OwnedDrawData` struct is empty.
    #[inline]
    fn default() -> Self {
        Self {
            draw_data: std::ptr::null_mut(),
        }
    }
}

impl From<&DrawData> for OwnedDrawData {
    /// Construct `OwnedDrawData` from `DrawData` by creating a heap-allocated deep copy of the given `DrawData`
    fn from(value: &DrawData) -> Self {
        OwnedDrawData {
            draw_data: unsafe {
                let other_ptr = value.raw();
                let result = sys::ImDrawData_ImDrawData();
                (*result).Valid = other_ptr.Valid;
                (*result).TotalIdxCount = other_ptr.TotalIdxCount;
                (*result).TotalVtxCount = other_ptr.TotalVtxCount;
                (*result).DisplayPos = other_ptr.DisplayPos;
                (*result).DisplaySize = other_ptr.DisplaySize;
                (*result).FramebufferScale = other_ptr.FramebufferScale;
                (*result).OwnerViewport = other_ptr.OwnerViewport;
                // Textures is a pointer to a vector owned by ImGuiPlatformIO; the
                // textures themselves outlive any single frame, so propagating the
                // pointer is sufficient (and matches imgui's own ImDrawData copy).
                (*result).Textures = other_ptr.Textures;

                (*result).CmdListsCount = 0;
                for i in 0..other_ptr.CmdListsCount as usize {
                    sys::ImDrawData_AddDrawList(result, *other_ptr.CmdLists.Data.add(i));
                    (*result).CmdListsCount += 1;
                }
                result
            },
        }
    }
}

impl Drop for OwnedDrawData {
    /// Releases any heap-allocated memory consumed by this `OwnedDrawData` object
    fn drop(&mut self) {
        unsafe {
            if !self.draw_data.is_null() {
                if !(*self.draw_data).CmdLists.Data.is_null() {
                    for i in 0..(*self.draw_data).CmdListsCount as usize {
                        let ptr = *(*self.draw_data).CmdLists.Data.add(i);
                        if !ptr.is_null() {
                            sys::ImDrawList_destroy(ptr);
                        }
                    }
                    sys::igMemFree((*self.draw_data).CmdLists.Data as *mut std::ffi::c_void);
                }
                sys::ImDrawData_destroy(self.draw_data);
                self.draw_data = std::ptr::null_mut();
            }
        }
    }
}

#[test]
#[cfg(test)]
fn test_owneddrawdata_default() {
    let default = OwnedDrawData::default();
    assert!(default.draw_data().is_none());
}

#[test]
#[cfg(test)]
fn test_owneddrawdata_from_drawdata() {
    let (_guard, _ctx) = crate::test::test_ctx();

    // Build a dummy draw data object
    let mut draw_list = sys::ImDrawList::default();
    let mut draw_lists_raw = [std::ptr::addr_of_mut!(draw_list)];
    let draw_data_raw = sys::ImDrawData {
        Valid: true,
        CmdListsCount: 1,
        CmdLists: sys::ImVector_ImDrawListPtr {
            Size: 1,
            Capacity: 1,
            Data: draw_lists_raw.as_mut_ptr(),
        },
        TotalIdxCount: 123,
        TotalVtxCount: 456,
        DisplayPos: sys::ImVec2 { x: 123.0, y: 456.0 },
        DisplaySize: sys::ImVec2 { x: 789.0, y: 012.0 },
        FramebufferScale: sys::ImVec2 { x: 3.0, y: 7.0 },
        OwnerViewport: unsafe { std::ptr::null_mut::<sys::ImGuiViewport>().offset(123) },
        Textures: std::ptr::null_mut(),
    };
    let draw_data = unsafe { DrawData::from_raw(&draw_data_raw) };

    // Clone it, and ensure the underlying properties have been cloned
    let owned_draw_data: OwnedDrawData = draw_data.into();
    let inner_draw_data = owned_draw_data.draw_data();
    assert!(inner_draw_data.is_some());
    let owned_draw_data_raw = unsafe { inner_draw_data.unwrap().raw() };
    assert_eq!(draw_data_raw.Valid, owned_draw_data_raw.Valid);
    assert_eq!(
        draw_data_raw.CmdListsCount,
        owned_draw_data_raw.CmdListsCount
    );
    assert!(!draw_data_raw.CmdLists.Data.is_null());
    assert_eq!(
        draw_data_raw.TotalIdxCount,
        owned_draw_data_raw.TotalIdxCount
    );
    assert_eq!(
        draw_data_raw.TotalVtxCount,
        owned_draw_data_raw.TotalVtxCount
    );
    assert_eq!(draw_data_raw.DisplayPos, owned_draw_data_raw.DisplayPos);
    assert_eq!(draw_data_raw.DisplaySize, owned_draw_data_raw.DisplaySize);
    assert_eq!(
        draw_data_raw.FramebufferScale,
        owned_draw_data_raw.FramebufferScale
    );

    #[cfg(feature = "docking")]
    assert_eq!(
        draw_data_raw.OwnerViewport,
        owned_draw_data_raw.OwnerViewport
    );
}
