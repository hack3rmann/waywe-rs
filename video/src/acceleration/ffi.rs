use std::{
    ffi::{c_char, c_void},
    os::fd::RawFd,
};

/// Window system dependent
pub type Display = *mut c_void;
/// Generic ID type, can be re-typed for specific implementation
pub type GenericId = u32;
pub type SurfaceId = GenericId;
/// Return status type from functions
pub type Status = i32;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct DrmPrimeSurfaceDescriptorObject {
    /// DRM PRIME file descriptor for this object.
    pub fd: RawFd,
    /// Total size of this object (may include regions which are not part of the surface).
    pub size: u32,
    /// Format modifier applied to this object.
    pub drm_format_modifier: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct DrmPrimeSurfaceDescriptorLayer {
    /// DRM format fourcc of this layer (DRM_FOURCC_*).
    pub drm_format: u32,
    /// Number of planes in this layer.
    pub num_planes: u32,
    /// Index in the objects array of the object containing each plane.
    pub object_index: [u32; 4],
    /// Offset within the object of each plane.
    pub offset: [u32; 4],
    /// Pitch of each plane.
    pub pitch: [u32; 4],
}

/// External buffer descriptor for a DRM PRIME surface with flags
///
/// This structure is an extention for VADRMPRIMESurfaceDescriptor,
/// it has the same behavior as if used with VA_SURFACE_ATTRIB_MEM_TYPE_DRM_PRIME_2.
///
/// The field "flags" is added, see "Surface external buffer descriptor flags".
/// To use this structure, use VA_SURFACE_ATTRIB_MEM_TYPE_DRM_PRIME_3 instead.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct DrmPrimeDescriptor {
    /// Pixel format fourcc of the whole surface (VA_FOURCC_*).
    pub fourcc: u32,
    /// Width of the surface in pixels.
    pub width: u32,
    /// Height of the surface in pixels.
    pub height: u32,
    /// Number of distinct DRM objects making up the surface.
    pub num_objects: u32,
    /// Description of each object.
    pub objects: [DrmPrimeSurfaceDescriptorObject; 4],
    /// Number of layers making up the surface.
    pub num_layers: u32,
    /// Description of each layer in the surface.
    pub layers: [DrmPrimeSurfaceDescriptorLayer; 4],
}

/// VAAPI connection details.
///
/// Allocated as AVHWDeviceContext.hwctx
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AvVaApiDeviceContext {
    /// The VADisplay handle, to be filled by the user.
    pub display: Display,
    /// Driver quirks to apply - this is filled by av_hwdevice_ctx_init(),
    /// with reference to a table of known drivers, unless the
    /// AV_VAAPI_DRIVER_QUIRK_USER_SET bit is already present. The user
    /// may need to refer to this field when performing any later
    /// operations using VAAPI with the same VADisplay.
    pub driver_quirks: u32,
}

pub use binds::{
    vaErrorStr as error_str, vaExportSurfaceHandle as export_surface_handle,
    vaSyncSurface as sync_surface,
};

pub mod binds {
    use super::*;

    unsafe extern "C" {
        /// Export a handle to a surface for use with an external API
        ///
        /// The exported handles are owned by the caller, and the caller is
        /// responsible for freeing them when no longer needed (e.g. by closing
        /// DRM PRIME file descriptors).
        ///
        /// This does not perform any synchronisation.  If the contents of the
        /// surface will be read, vaSyncSurface() must be called before doing so.
        /// If the contents of the surface are written, then all operations must
        /// be completed externally before using the surface again by via VA-API
        /// functions.
        ///
        /// # Parameters
        ///
        /// | Parameter     | Type | Description                         |
        /// | ------------- | ---- | ----------------------------------- |
        /// | `display`     | in   | VA display                          |
        /// | `surface_id`  | in   | Surface to export                   |
        /// | `memoru_type` | in   | Memory type to export to            |
        /// | `flags`       | in   | Combination of flags to apply       |
        /// | `descriptor`  | out  | Pointer to the descriptor structure |
        ///
        /// `descriptor` is a pointer to the descriptor structure is to fill with the
        /// handle details.  The type of this structure depends on the
        /// value of `memory_type`.
        ///
        /// # Return
        ///
        /// Status code:
        ///
        /// - `VA_STATUS_SUCCESS`:    Success.
        /// - `VA_STATUS_ERROR_INVALID_DISPLAY`:  The display is not valid.
        /// - `VA_STATUS_ERROR_UNIMPLEMENTED`:  The driver does not implement
        ///   this interface.
        /// - `VA_STATUS_ERROR_INVALID_SURFACE`:  The surface is not valid, or
        ///   the surface is not exportable in the specified way.
        /// - `VA_STATUS_ERROR_UNSUPPORTED_MEMORY_TYPE`:  The driver does not
        ///   support exporting surfaces to the specified memory type.
        pub fn vaExportSurfaceHandle(
            display: Display,
            surface_id: SurfaceId,
            memory_type: u32,
            flags: u32,
            descriptor: *mut c_void,
        ) -> Status;

        /// Returns a short english description of `status`
        pub safe fn vaErrorStr(status: Status) -> *const c_char;

        /// This function blocks until all pending operations on the render target
        /// have been completed. Upon return it is safe to use the render target for a
        /// different picture.
        pub fn vaSyncSurface(display: Display, render_target: SurfaceId) -> Status;
    }
}
