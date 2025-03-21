use crate::sys::wire::{MessageBuffer, OpCode, WlMessage};

pub mod request {
    use super::*;
    use crate::interface::{ObjectParent, Request};
    use crate::object::{HasObjectType, WlObjectType};
    use crate::sys::object::dispatch::State;
    use crate::sys::object_storage::WlObjectStorage;

    /// This creates an xdg_toplevel object for the given xdg_surface and gives
    /// the associated wl_surface the xdg_toplevel role.
    ///
    /// See the documentation of xdg_toplevel for more details about what an
    /// xdg_toplevel is and how it is used.
    #[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct GetToplevel;

    impl HasObjectType for GetToplevel {
        const OBJECT_TYPE: WlObjectType = WlObjectType::XdgSurface;
    }

    impl ObjectParent for GetToplevel {
        const CHILD_TYPE: WlObjectType = WlObjectType::XdgToplevel;
    }

    impl<'s> Request<'s> for GetToplevel {
        const CODE: OpCode = 1;
        const OUTGOING_INTERFACE: Option<WlObjectType> = Some(Self::CHILD_TYPE);

        fn build_message<'m, S: State>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage<'_, S>,
        ) -> WlMessage<'m>
        where
            's: 'm,
        {
            WlMessage::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }

    /// When a configure event is received, if a client commits the
    /// surface in response to the configure event, then the client
    /// must make an ack_configure request sometime before the commit
    /// request, passing along the serial of the configure event.
    ///
    /// For instance, for toplevel surfaces the compositor might use this
    /// information to move a surface to the top left only when the client has
    /// drawn itself for the maximized or fullscreen state.
    ///
    /// If the client receives multiple configure events before it
    /// can respond to one, it only has to ack the last configure event.
    /// Acking a configure event that was never sent raises an invalid_serial
    /// error.
    ///
    /// A client is not required to commit immediately after sending
    /// an ack_configure request - it may even ack_configure several times
    /// before its next surface commit.
    ///
    /// A client may send multiple ack_configure requests before committing, but
    /// only the last request sent before a commit indicates which configure
    /// event the client really is responding to.
    ///
    /// Sending an ack_configure request consumes the serial number sent with
    /// the request, as well as serial numbers sent by all configure events
    /// sent on this xdg_surface prior to the configure event referenced by
    /// the committed serial.
    ///
    /// It is an error to issue multiple ack_configure requests referencing a
    /// serial from the same configure event, or to issue an ack_configure
    /// request referencing a serial from a configure event issued before the
    /// event identified by the last ack_configure request for the same
    /// xdg_surface. Doing so will raise an invalid_serial error.
    #[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct AckConfigure {
        /// The serial from the configure event
        pub serial: u32,
    }

    impl HasObjectType for AckConfigure {
        const OBJECT_TYPE: WlObjectType = WlObjectType::XdgSurface;
    }

    impl<'s> Request<'s> for AckConfigure {
        const CODE: OpCode = 4;

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

    /// The configure event marks the end of a configure sequence. A configure
    /// sequence is a set of one or more events configuring the state of the
    /// xdg_surface, including the final xdg_surface.configure event.
    ///
    /// Where applicable, xdg_surface surface roles will during a configure
    /// sequence extend this event as a latched state sent as events before the
    /// xdg_surface.configure event. Such events should be considered to make up
    /// a set of atomically applied configuration states, where the
    /// xdg_surface.configure commits the accumulated state.
    ///
    /// Clients should arrange their surface for the new states, and then send
    /// an ack_configure request with the serial sent in this configure event at
    /// some point before committing the new surface.
    ///
    /// If the client receives multiple configure events before it can respond
    /// to one, it is free to discard all but the last event it received.
    #[derive(Clone, Default, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Configure {
        /// Serial of the configure event
        pub serial: u32,
    }

    impl<'s> Event<'s> for Configure {
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
