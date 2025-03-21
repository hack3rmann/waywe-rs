//! Clients can use this interface to assign the surface_layer role to
//! wl_surfaces. Such surfaces are assigned to a "layer" of the output and
//! rendered with a defined z-depth respective to each other. They may also be
//! anchored to the edges and corners of a screen and specify input handling
//! semantics. This interface should be suitable for the implementation of
//! many desktop shell components, and a broad number of other applications
//! that interact with the desktop.

pub mod request {
    use super::wl_enum::Layer;
    use crate::WlObjectId;
    use crate::interface::{ObjectParent, Request};
    use crate::object::{HasObjectType, WlObjectType};
    use crate::sys::object::dispatch::State;
    use crate::sys::object_storage::WlObjectStorage;
    use crate::sys::wire::{MessageBuffer, OpCode, WlMessage};
    use std::ffi::CStr;

    /// Create a layer surface for an existing surface. This assigns the role of
    /// layer_surface, or raises a protocol error if another role is already
    /// assigned.
    ///
    /// Creating a layer surface from a wl_surface which has a buffer attached
    /// or committed is a client error, and any attempts by a client to attach
    /// or manipulate a buffer prior to the first layer_surface.configure call
    /// must also be treated as errors.
    ///
    /// After creating a layer_surface object and setting it up, the client
    /// must perform an initial commit without any buffer attached.
    /// The compositor will reply with a layer_surface.configure event.
    /// The client must acknowledge it and is then allowed to attach a buffer
    /// to map the surface.
    ///
    /// You may pass NULL for output to allow the compositor to decide which
    /// output to use. Generally this will be the one that the user most
    /// recently interacted with.
    ///
    /// Clients can specify a namespace that defines the purpose of the layer
    /// surface.
    #[derive(Debug, Clone, Copy)]
    pub struct GetLayerSurface<'a> {
        pub surface: WlObjectId,
        pub output: Option<WlObjectId>,
        pub layer: Layer,
        pub namespace: &'a CStr,
    }

    impl ObjectParent for GetLayerSurface<'_> {
        const CHILD_TYPE: WlObjectType = WlObjectType::ZwlrLayerSurfaceV1;
    }

    impl HasObjectType for GetLayerSurface<'_> {
        const OBJECT_TYPE: WlObjectType = WlObjectType::ZwlrLayerShellV1;
    }

    impl<'s> Request<'s> for GetLayerSurface<'s> {
        const CODE: OpCode = 0;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(WlObjectType::ZwlrLayerSurfaceV1);

        fn build_message<'m, S: State>(
            self,
            buf: &'m mut impl MessageBuffer,
            storage: &'m WlObjectStorage<'_, S>,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf)
                .opcode(Self::CODE)
                .new_id()
                .object(storage.get_proxy(self.surface).unwrap())
                .maybe_object(self.output.map(|h| storage.get_proxy(h).unwrap()))
                .uint(self.layer.into())
                .str(self.namespace)
                .build()
        }
    }

    /// This request indicates that the client will not use the layer_shell
    /// object any more. Objects that have been created through this instance
    /// are not affected.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Destroy;

    impl HasObjectType for Destroy {
        const OBJECT_TYPE: WlObjectType = WlObjectType::ZwlrLayerShellV1;
    }

    impl<'s> Request<'s> for Destroy {
        const CODE: OpCode = 1;

        fn build_message<'m, S: State>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage<'_, S>,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf).opcode(Self::CODE).build()
        }
    }
}

pub mod wl_enum {
    use thiserror::Error;

    /// These values indicate which layers a surface can be rendered in. They
    /// are ordered by z depth, bottom-most first. Traditional shell surfaces
    /// will typically be rendered between the bottom and top layers.
    /// Fullscreen shell surfaces are typically rendered at the top layer.
    /// Multiple surfaces can share a single layer, and ordering within a
    /// single layer is undefined.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Layer {
        Background = 0,
        Bottom = 1,
        Top = 2,
        Overlay = 3,
    }

    impl TryFrom<u32> for Layer {
        type Error = WrongEnumVariant;

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            Ok(match value {
                0 => Layer::Background,
                1 => Layer::Bottom,
                2 => Layer::Top,
                3 => Layer::Overlay,
                _ => return Err(WrongEnumVariant(value)),
            })
        }
    }

    #[derive(Debug, Error)]
    #[error("no Layer enum variant for {0} value")]
    pub struct WrongEnumVariant(pub u32);

    impl From<Layer> for u32 {
        fn from(value: Layer) -> Self {
            value as u32
        }
    }
}
