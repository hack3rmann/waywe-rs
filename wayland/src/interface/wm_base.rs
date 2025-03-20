//! The xdg_wm_base interface is exposed as a global object enabling clients
//! to turn their wl_surfaces into windows in a desktop environment. It
//! defines the basic functionality needed for clients and the compositor to
//! create windows that can be dragged, resized, maximized, etc, as well as
//! creating transient windows such as popup menus.

pub mod request {
    use crate::{
        MessageBuffer, WlObjectId,
        interface::{ObjectParent, Request},
        object::{HasObjectType, WlObjectType},
        sys::{
            object::dispatch::State,
            object_storage::WlObjectStorage,
            wire::{OpCode, WlMessage},
        },
    };

    /// This creates an xdg_surface for the given surface. While xdg_surface
    /// itself is not a role, the corresponding surface may only be assigned
    /// a role extending xdg_surface, such as xdg_toplevel or xdg_popup. It is
    /// illegal to create an xdg_surface for a wl_surface which already has an
    /// assigned role and this will result in a role error.
    ///
    /// This creates an xdg_surface for the given surface. An xdg_surface is
    /// used as basis to define a role to a given surface, such as xdg_toplevel
    /// or xdg_popup. It also manages functionality shared between xdg_surface
    /// based surface roles.
    ///
    /// See the documentation of xdg_surface for more details about what an
    /// xdg_surface is and how it is used.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct GetXdgSurface {
        /// `wl_surface`
        pub surface: WlObjectId,
    }

    impl HasObjectType for GetXdgSurface {
        const OBJECT_TYPE: WlObjectType = WlObjectType::WmBase;
    }

    impl ObjectParent for GetXdgSurface {
        const CHILD_TYPE: WlObjectType = WlObjectType::XdgSurface;
    }

    impl<'s> Request<'s> for GetXdgSurface {
        const CODE: OpCode = 2;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(WlObjectType::XdgSurface);

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
                .build()
        }
    }

    /// A client must respond to a ping event with a pong request or
    /// the client may be deemed unresponsive. See xdg_wm_base.ping
    /// and xdg_wm_base.error.unresponsive.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Pong {
        /// Serial for the ping event
        pub serial: u32,
    }

    impl HasObjectType for Pong {
        const OBJECT_TYPE: WlObjectType = WlObjectType::WmBase;
    }

    impl<'s> Request<'s> for Pong {
        const CODE: OpCode = 3;

        fn build_message<'m, S: State>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage<'_, S>,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf)
                .opcode(Self::CODE)
                .uint(self.serial)
                .build()
        }
    }
}

pub mod event {
    use crate::{
        interface::Event,
        sys::wire::{OpCode, WlMessage},
    };

    /// The ping event asks the client if it's still alive. Pass the
    /// serial specified in the event back to the compositor by sending
    /// a "pong" request back with the specified serial. See xdg_wm_base.pong.
    ///
    /// Compositors can use this to determine if the client is still
    /// alive. It's unspecified what will happen if the client doesn't
    /// respond to the ping request, or in what timeframe. Clients should
    /// try to respond in a reasonable amount of time. The “unresponsive”
    /// error is provided for compositors that wish to disconnect unresponsive
    /// clients.
    ///
    /// A compositor is free to ping in any way it wants, but a client must
    /// always respond to any xdg_wm_base object it created.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Ping {
        /// Pass this to the pong request
        pub serial: u32,
    }

    impl<'s> Event<'s> for Ping {
        const CODE: OpCode = 0;

        fn from_message(message: WlMessage<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();
            let serial = unsafe { reader.read::<u32>()? };

            Some(Self { serial })
        }
    }
}
