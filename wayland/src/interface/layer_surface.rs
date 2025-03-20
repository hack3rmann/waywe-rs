use crate::sys::wire::{WlMessage, MessageBuffer, OpCode};

pub mod request {
    use super::wl_enum::{Anchor, KeyboardInteractivity};
    use super::*;
    use crate::interface::Request;
    use crate::object::{HasObjectType, WlObjectType};
    use crate::sys::object::dispatch::State;
    use crate::sys::object_storage::WlObjectStorage;

    /// Sets the size of the surface in surface-local coordinates. The
    /// compositor will display the surface centered with respect to its
    /// anchors.
    ///
    /// If you pass 0 for either value, the compositor will assign it and
    /// inform you of the assignment in the configure event. You must set your
    /// anchor to opposite edges in the dimensions you omit; not doing so is a
    /// protocol error. Both values are 0 by default.
    ///
    /// Size is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetSize {
        pub width: u32,
        pub height: u32,
    }

    impl HasObjectType for SetSize {
        const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
    }

    impl<'s> Request<'s> for SetSize {
        const CODE: OpCode = 0;

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
                .uint(self.width)
                .uint(self.height)
                .build()
        }
    }

    /// Requests that the compositor anchor the surface to the specified edges
    /// and corners. If two orthogonal edges are specified (e.g. 'top' and
    /// 'left'), then the anchor point will be the intersection of the edges
    /// (e.g. the top left corner of the output); otherwise the anchor point
    /// will be centered on that edge, or in the center if none is specified.
    ///
    /// Anchor is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetAnchor {
        pub anchor: Anchor,
    }

    impl HasObjectType for SetAnchor {
        const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
    }

    impl<'s> Request<'s> for SetAnchor {
        const CODE: OpCode = 1;

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
                .uint(self.anchor.bits())
                .build()
        }
    }

    /// Requests that the compositor avoids occluding an area with other
    /// surfaces. The compositor's use of this information is
    /// implementation-dependent - do not assume that this region will not
    /// actually be occluded.
    ///
    /// A positive value is only meaningful if the surface is anchored to one
    /// edge or an edge and both perpendicular edges. If the surface is not
    /// anchored, anchored to only two perpendicular edges (a corner), anchored
    /// to only two parallel edges or anchored to all edges, a positive value
    /// will be treated the same as zero.
    ///
    /// A positive zone is the distance from the edge in surface-local
    /// coordinates to consider exclusive.
    ///
    /// Surfaces that do not wish to have an exclusive zone may instead specify
    /// how they should interact with surfaces that do. If set to zero, the
    /// surface indicates that it would like to be moved to avoid occluding
    /// surfaces with a positive exclusive zone. If set to -1, the surface
    /// indicates that it would not like to be moved to accommodate for other
    /// surfaces, and the compositor should extend it all the way to the edges
    /// it is anchored to.
    ///
    /// For example, a panel might set its exclusive zone to 10, so that
    /// maximized shell surfaces are not shown on top of it. A notification
    /// might set its exclusive zone to 0, so that it is moved to avoid
    /// occluding the panel, but shell surfaces are shown underneath it. A
    /// wallpaper or lock screen might set their exclusive zone to -1, so that
    /// they stretch below or over the panel.
    ///
    /// The default value is 0.
    ///
    /// Exclusive zone is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetExclusiveZone {
        pub zone: i32,
    }

    impl HasObjectType for SetExclusiveZone {
        const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
    }

    impl<'s> Request<'s> for SetExclusiveZone {
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
                .int(self.zone)
                .build()
        }
    }

    /// Requests that the surface be placed some distance away from the anchor
    /// point on the output, in surface-local coordinates. Setting this value
    /// for edges you are not anchored to has no effect.
    ///
    /// The exclusive zone includes the margin.
    ///
    /// Margin is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetMargin {
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
        pub left: i32,
    }

    impl SetMargin {
        pub const fn zero() -> Self {
            Self {
                top: 0,
                right: 0,
                bottom: 0,
                left: 0,
            }
        }
    }

    impl HasObjectType for SetMargin {
        const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
    }

    impl<'s> Request<'s> for SetMargin {
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
                .int(self.top)
                .int(self.right)
                .int(self.bottom)
                .int(self.left)
                .build()
        }
    }

    /// Set how keyboard events are delivered to this surface. By default,
    /// layer shell surfaces do not receive keyboard events; this request can
    /// be used to change this.
    ///
    /// This setting is inherited by child surfaces set by the get_popup
    /// request.
    ///
    /// Layer surfaces receive pointer, touch, and tablet events normally. If
    /// you do not want to receive them, set the input region on your surface
    /// to an empty region.
    ///
    /// Keyboard interactivity is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetKeyboardInteractivity {
        pub keyboard_interactivity: KeyboardInteractivity,
    }

    impl HasObjectType for SetKeyboardInteractivity {
        const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
    }

    impl<'s> Request<'s> for SetKeyboardInteractivity {
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
                .uint(self.keyboard_interactivity.into())
                .build()
        }
    }

    /// When a configure event is received, if a client commits the
    /// surface in response to the configure event, then the client
    /// must make an ack_configure request sometime before the commit
    /// request, passing along the serial of the configure event.
    ///
    /// If the client receives multiple configure events before it
    /// can respond to one, it only has to ack the last configure event.
    ///
    /// A client is not required to commit immediately after sending
    /// an ack_configure request - it may even ack_configure several times
    /// before its next surface commit.
    ///
    /// A client may send multiple ack_configure requests before committing, but
    /// only the last request sent before a commit indicates which configure
    /// event the client really is responding to.
    pub struct AckConfigure {
        /// The serial from the configure event
        pub serial: u32,
    }

    impl HasObjectType for AckConfigure {
        const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
    }

    impl<'s> Request<'s> for AckConfigure {
        const CODE: OpCode = 6;

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
    use crate::{interface::Event, sys::wire::{WlMessage, OpCode}};

    /// The configure event asks the client to resize its surface.
    ///
    /// Clients should arrange their surface for the new states, and then send
    /// an ack_configure request with the serial sent in this configure event at
    /// some point before committing the new surface.
    ///
    /// The client is free to dismiss all but the last configure event it
    /// received.
    ///
    /// The width and height arguments specify the size of the window in
    /// surface-local coordinates.
    ///
    /// The size is a hint, in the sense that the client is free to ignore it if
    /// it doesn't resize, pick a smaller size (to satisfy aspect ratio or
    /// resize in steps of NxM pixels). If the client picks a smaller size and
    /// is anchored to two opposite anchors (e.g. 'top' and 'bottom'), the
    /// surface will be centered on this axis.
    ///
    /// If the width or height arguments are zero, it means the client should
    /// decide its own window dimension.
    #[derive(Clone, Default, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub struct Configure {
        pub serial: u32,
        pub width: u32,
        pub height: u32,
    }

    impl<'s> Event<'s> for Configure {
        const CODE: OpCode = 0;

        fn from_message(message: WlMessage<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();

            let serial = unsafe { reader.read::<u32>()? };
            let width = unsafe { reader.read::<u32>()? };
            let height = unsafe { reader.read::<u32>()? };

            Some(Self { serial, width, height })
        }
    }
}

pub mod wl_enum {
    use thiserror::Error;

    bitflags::bitflags! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Anchor: u32 {
            const TOP = 0x1;
            const BOTTOM = 0x2;
            const LEFT = 0x4;
            const RIGHT = 0x8;
        }
    }

    /// Types of keyboard interaction possible for layer shell surfaces. The
    /// rationale for this is twofold: (1) some applications are not interested
    /// in keyboard events and not allowing them to be focused can improve the
    /// desktop experience; (2) some applications will want to take exclusive
    /// keyboard focus.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum KeyboardInteractivity {
        /// no keyboard focus is possible
        None = 0,
        /// request regular keyboard focus semantics
        Exclusive = 1,
        /// request regular keyboard focus semantics
        OnDemand = 2,
    }

    impl TryFrom<u32> for KeyboardInteractivity {
        type Error = WrongEnumVariant;

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            Ok(match value {
                0 => Self::None,
                1 => Self::Exclusive,
                2 => Self::OnDemand,
                _ => return Err(WrongEnumVariant(value)),
            })
        }
    }

    #[derive(Debug, Error)]
    #[error("no KeyboardInteractivity enum variant for {0} value")]
    pub struct WrongEnumVariant(pub u32);

    impl From<KeyboardInteractivity> for u32 {
        fn from(value: KeyboardInteractivity) -> Self {
            value as u32
        }
    }
}
