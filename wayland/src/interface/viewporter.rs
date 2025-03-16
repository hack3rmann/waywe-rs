//! The global interface exposing surface cropping and scaling
//! capabilities is used to instantiate an interface extension for a
//! wl_surface object. This extended interface will then allow
//! cropping and scaling the surface contents, effectively
//! disconnecting the direct relationship between the buffer and the
//! surface size.

pub mod request {
    use crate::{
        interface::{ObjectParent, Request},
        object::{HasObjectType, WlObjectType},
        sys::{
            object::{default_impl::{WlSurface, WlViewport}, WlObjectHandle},
            object_storage::WlObjectStorage,
            wire::{MessageBuffer, OpCode, WlMessage},
        },
    };

    /// Instantiate an interface extension for the given wl_surface to
    /// crop and scale its content. If the given wl_surface already has
    /// a wp_viewport object associated, the viewport_exists
    /// protocol error is raised.
    #[derive(Debug, Clone, PartialEq, Copy, Hash)]
    pub struct GetViewport {
        /// The surface
        pub surface: WlObjectHandle<WlSurface>,
    }

    impl ObjectParent for GetViewport {
        type Child = WlViewport;
    }

    impl HasObjectType for GetViewport {
        const OBJECT_TYPE: WlObjectType = WlObjectType::Viewporter;
    }

    impl<'s> Request<'s> for GetViewport {
        const CODE: OpCode = 1;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(WlObjectType::Viewport);

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            storage: &'m WlObjectStorage,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf)
                .opcode(Self::CODE)
                .new_id()
                .object(storage.object(self.surface).proxy())
                .build()
        }
    }
}
