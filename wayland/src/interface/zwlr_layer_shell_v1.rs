//! Clients can use this interface to assign the surface_layer role to
//! wl_surfaces. Such surfaces are assigned to a "layer" of the output and
//! rendered with a defined z-depth respective to each other. They may also be
//! anchored to the edges and corners of a screen and specify input handling
//! semantics. This interface should be suitable for the implementation of
//! many desktop shell components, and a broad number of other applications
//! that interact with the desktop.

pub mod request {
    use crate::{
        interface::{NewId, Request},
        object::ObjectId,
        wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
    };

    use super::wl_enum::Layer;

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
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct GetLayerSurface<'a> {
        pub object_id: ObjectId,
        /// id of the zwlr_layer_surface_v1
        pub id: NewId<'a>,
        pub surface: ObjectId,
        pub output: Option<ObjectId>,
        pub layer: Layer,
        pub namespace: &'a str,
    }

    impl Request for GetLayerSurface<'_> {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 0,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(self.header_desc())
                .new_id(self.id)
                .uint(self.surface.into())
                .uint(self.output.unwrap_or(ObjectId::new(0)).into())
                .uint(self.layer.into())
                .str(self.namespace)
                .build()
        }
    }

    ///This request indicates that the client will not use the layer_shell
    ///object any more. Objects that have been created through this instance
    ///are not affected.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Destroy {
        pub object_id: ObjectId,
    }

    impl Request for Destroy {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 1,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf).header(self.header_desc()).build()
        }
    }
}

pub mod wl_enum {

    ///These values indicate which layers a surface can be rendered in. They
    ///are ordered by z depth, bottom-most first. Traditional shell surfaces
    ///will typically be rendered between the bottom and top layers.
    ///Fullscreen shell surfaces are typically rendered at the top layer.
    ///Multiple surfaces can share a single layer, and ordering within a
    ///single layer is undefined.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Layer {
        Background = 0,
        Bottom = 1,
        Top = 2,
        Overlay = 3,
    }

    impl From<u32> for Layer {
        fn from(value: u32) -> Self {
            match value {
                0 => Layer::Background,
                1 => Layer::Bottom,
                2 => Layer::Top,
                3 => Layer::Overlay,
                _ => panic!("inappropriate variant for layer enum"),
            }
        }
    }

    impl From<Layer> for u32 {
        fn from(value: Layer) -> Self {
            match value {
                Layer::Background => 0,
                Layer::Bottom => 1,
                Layer::Top => 2,
                Layer::Overlay => 3,
            }
        }
    }
}
