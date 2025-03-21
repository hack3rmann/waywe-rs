pub mod request {
    use crate::{
        HasObjectType, MessageBuffer, OpCode, WlObjectStorage, WlObjectType,
        interface::Request,
        sys::{object::dispatch::State, wire::WlMessage},
    };
    use std::ffi::CStr;

    #[derive(Clone, Debug, Default, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct SetTitle<'s>(pub &'s CStr);

    impl HasObjectType for SetTitle<'_> {
        const OBJECT_TYPE: WlObjectType = WlObjectType::XdgToplevel;
    }

    impl<'s> Request<'s> for SetTitle<'s> {
        const CODE: OpCode = 2;

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
                .str(self.0)
                .build()
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct SetAppId<'s>(pub &'s CStr);

    impl HasObjectType for SetAppId<'_> {
        const OBJECT_TYPE: WlObjectType = WlObjectType::XdgToplevel;
    }

    impl<'s> Request<'s> for SetAppId<'s> {
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
                .str(self.0)
                .build()
        }
    }
}

pub mod event {
    use super::wl_enum::State;
    use crate::{
        interface::Event,
        sys::wire::{OpCode, WlMessage},
    };

    #[derive(Clone, Debug, Default, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Configure<'s> {
        pub width: u32,
        pub height: u32,
        pub states: &'s [State],
    }

    impl<'s> Event<'s> for Configure<'s> {
        const CODE: OpCode = 0;

        fn from_message(message: WlMessage<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();

            let width = unsafe { reader.read::<u32>()? };
            let height = unsafe { reader.read::<u32>()? };
            let states = unsafe { reader.read::<&[State]>()? };

            Some(Self {
                width,
                height,
                states,
            })
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Close;

    impl<'s> Event<'s> for Close {
        const CODE: OpCode = 1;

        fn from_message(message: WlMessage<'s>) -> Option<Self> {
            (message.opcode == Self::CODE).then_some(Self)
        }
    }
}

pub mod wl_enum {
    /// The different state values used on the surface. This is designed for
    /// state values like maximized, fullscreen. It is paired with the
    /// configure event to ensure that both the client and the compositor
    /// setting the state can be synchronized.
    ///
    /// States set in this way are double-buffered, see wl_surface.commit.
    #[repr(u32)]
    #[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub enum State {
        /// The surface is maximized. The window geometry specified in the configure
        /// event must be obeyed by the client, or the xdg_wm_base.invalid_surface_state
        /// error is raised.
        ///
        /// The client should draw without shadow or other
        /// decoration outside of the window geometry.
        Maximized = 1,
        /// The surface is fullscreen. The window geometry specified in the
        /// configure event is a maximum; the client cannot resize beyond it. For
        /// a surface to cover the whole fullscreened area, the geometry
        /// dimensions must be obeyed by the client. For more details, see
        /// xdg_toplevel.set_fullscreen.
        Fullscreen = 2,
        /// The surface is being resized. The window geometry specified in the
        /// configure event is a maximum; the client cannot resize beyond it.
        /// Clients that have aspect ratio or cell sizing configuration can use
        /// a smaller size, however.
        Resizing = 3,
        /// Client window decorations should be painted as if the window is
        /// active. Do not assume this means that the window actually has
        /// keyboard or pointer focus.
        Activated = 4,
        /// The window is currently in a tiled layout and the left edge is
        /// considered to be adjacent to another part of the tiling grid.
        ///
        /// The client should draw without shadow or other decoration outside of
        /// the window geometry on the left edge.
        TiltedLeft = 5,
        /// The window is currently in a tiled layout and the right edge is
        /// considered to be adjacent to another part of the tiling grid.
        ///
        /// The client should draw without shadow or other decoration outside of
        /// the window geometry on the right edge.
        TiltedRight = 6,
        /// The window is currently in a tiled layout and the top edge is
        /// considered to be adjacent to another part of the tiling grid.
        ///
        /// The client should draw without shadow or other decoration outside of
        /// the window geometry on the top edge.
        TiltedTop = 7,
        /// The window is currently in a tiled layout and the bottom edge is
        /// considered to be adjacent to another part of the tiling grid.
        ///
        /// The client should draw without shadow or other decoration outside of
        /// the window geometry on the bottom edge.
        TiltedBottom = 8,
        /// The surface is currently not ordinarily being repainted; for
        /// example because its content is occluded by another window, or its
        /// outputs are switched off due to screen locking.
        Suspended = 9,
    }
}
