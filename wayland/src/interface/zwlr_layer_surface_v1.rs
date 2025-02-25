pub mod request {
    use crate::{
        interface::Request,
        object::ObjectId,
        wire::{Message, MessageBuffer, MessageBuildError, MessageHeaderDesc},
    };

    use super::wl_enum::{Anchor, KeyboardInteractivity};

    ///Sets the size of the surface in surface-local coordinates. The
    ///compositor will display the surface centered with respect to its
    ///anchors.
    ///
    ///If you pass 0 for either value, the compositor will assign it and
    ///inform you of the assignment in the configure event. You must set your
    ///anchor to opposite edges in the dimensions you omit; not doing so is a
    ///protocol error. Both values are 0 by default.
    ///
    ///Size is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetSize {
        pub object_id: ObjectId,
        pub width: u32,
        pub height: u32,
    }

    impl Request for SetSize {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 0,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(self.header_desc())
                .uint(self.width)
                .uint(self.height)
                .build()
        }
    }

    ///Requests that the compositor anchor the surface to the specified edges
    ///and corners. If two orthogonal edges are specified (e.g. 'top' and
    ///'left'), then the anchor point will be the intersection of the edges
    ///(e.g. the top left corner of the output); otherwise the anchor point
    ///will be centered on that edge, or in the center if none is specified.
    ///
    ///Anchor is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetAnchor {
        pub object_id: ObjectId,
        pub anchor: Anchor,
    }

    impl Request for SetAnchor {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 1,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(self.header_desc())
                .uint(self.anchor.bits())
                .build()
        }
    }

    ///Requests that the compositor avoids occluding an area with other
    ///surfaces. The compositor's use of this information is
    ///implementation-dependent - do not assume that this region will not
    ///actually be occluded.
    ///
    ///A positive value is only meaningful if the surface is anchored to one
    ///edge or an edge and both perpendicular edges. If the surface is not
    ///anchored, anchored to only two perpendicular edges (a corner), anchored
    ///to only two parallel edges or anchored to all edges, a positive value
    ///will be treated the same as zero.
    ///
    ///A positive zone is the distance from the edge in surface-local
    ///coordinates to consider exclusive.
    ///
    ///Surfaces that do not wish to have an exclusive zone may instead specify
    ///how they should interact with surfaces that do. If set to zero, the
    ///surface indicates that it would like to be moved to avoid occluding
    ///surfaces with a positive exclusive zone. If set to -1, the surface
    ///indicates that it would not like to be moved to accommodate for other
    ///surfaces, and the compositor should extend it all the way to the edges
    ///it is anchored to.
    ///
    ///For example, a panel might set its exclusive zone to 10, so that
    ///maximized shell surfaces are not shown on top of it. A notification
    ///might set its exclusive zone to 0, so that it is moved to avoid
    ///occluding the panel, but shell surfaces are shown underneath it. A
    ///wallpaper or lock screen might set their exclusive zone to -1, so that
    ///they stretch below or over the panel.
    ///
    ///The default value is 0.
    ///
    ///Exclusive zone is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetExclusiveZone {
        pub object_id: ObjectId,
        pub zone: i32,
    }

    impl Request for SetExclusiveZone {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 2,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(self.header_desc())
                .int(self.zone)
                .build()
        }
    }

    ///Requests that the surface be placed some distance away from the anchor
    ///point on the output, in surface-local coordinates. Setting this value
    ///for edges you are not anchored to has no effect.
    ///
    ///The exclusive zone includes the margin.
    ///
    ///Margin is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetMargine {
        pub object_id: ObjectId,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
        pub left: i32,
    }

    impl Request for SetMargine {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 3,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(self.header_desc())
                .int(self.top)
                .int(self.right)
                .int(self.bottom)
                .int(self.left)
                .build()
        }
    }

    ///Set how keyboard events are delivered to this surface. By default,
    ///layer shell surfaces do not receive keyboard events; this request can
    ///be used to change this.
    ///
    ///This setting is inherited by child surfaces set by the get_popup
    ///request.
    ///
    ///Layer surfaces receive pointer, touch, and tablet events normally. If
    ///you do not want to receive them, set the input region on your surface
    ///to an empty region.
    ///
    ///Keyboard interactivity is double-buffered, see wl_surface.commit.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SetKeyboardInteractivity {
        object_id: ObjectId,
        keyboard_interactivity: KeyboardInteractivity,
    }

    impl Request for SetKeyboardInteractivity {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 4,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(self.header_desc())
                .uint(self.keyboard_interactivity.into())
                .build()
        }
    }

    ///This assigns an xdg_popup's parent to this layer_surface.  This popup
    ///should have been created via xdg_surface::get_popup with the parent set
    ///to NULL, and this request must be invoked before committing the popup's
    ///initial state.
    ///
    ///See the documentation of xdg_popup for more details about what an
    ///xdg_popup is and how it is used.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct GetPopup {
        object_id: ObjectId,
        /// popup object of xdg_popup interface
        popup: ObjectId,
    }

    impl Request for GetPopup {
        fn header_desc(self) -> MessageHeaderDesc {
            MessageHeaderDesc {
                object_id: self.object_id,
                opcode: 5,
            }
        }

        fn build_message(self, buf: &mut MessageBuffer) -> Result<Message<'_>, MessageBuildError> {
            Message::builder(buf)
                .header(self.header_desc())
                .uint(self.popup.into())
                .build()
        }
    }
}

pub mod wl_enum {
    bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Anchor:u32 {
            const Top = 0b00000001;
            const Bottom = 0b00000010;
            const Left = 0b00000100;
            const Right = 0b00001000;
        }
    }

    ///Types of keyboard interaction possible for layer shell surfaces. The
    ///rationale for this is twofold: (1) some applications are not interested
    ///in keyboard events and not allowing them to be focused can improve the
    ///desktop experience; (2) some applications will want to take exclusive
    ///keyboard focus.
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum KeyboardInteractivity {
        /// no keyboard focus is possible
        None = 0,
        /// request regular keyboard focus semantics
        Exclusive = 1,
        /// request regular keyboard focus semantics
        OnDemand = 2,
    }

    impl From<u32> for KeyboardInteractivity {
        fn from(value: u32) -> Self {
            match value {
                0 => Self::None,
                1 => Self::Exclusive,
                2 => Self::OnDemand,
                _ => panic!("wrong enum variant"),
            }
        }
    }

    impl From<KeyboardInteractivity> for u32 {
        fn from(value: KeyboardInteractivity) -> Self {
            match value {
                KeyboardInteractivity::None => 0,
                KeyboardInteractivity::Exclusive => 1,
                KeyboardInteractivity::OnDemand => 2,
            }
        }
    }
}
