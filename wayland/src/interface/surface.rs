//! A surface is a rectangular area that may be displayed on zero
//! or more outputs, and shown any number of times at the compositor's
//! discretion. They can present wl_buffers, receive user input, and
//! define a local coordinate system.
//!
//! The size of a surface (and relative positions on it) is described
//! in surface-local coordinates, which may differ from the buffer
//! coordinates of the pixel content, in case a buffer_transform
//! or a buffer_scale is used.
//!
//! A surface without a "role" is fairly useless: a compositor does
//! not know where, when or how to present it. The role is the
//! purpose of a wl_surface. Examples of roles are a cursor for a
//! pointer (as set by wl_pointer.set_cursor), a drag icon
//! (wl_data_device.start_drag), a sub-surface
//! (wl_subcompositor.get_subsurface), and a window as defined by a
//! shell protocol (e.g. wl_shell.get_shell_surface).
//!
//! A surface can have only one role at a time. Initially a
//! wl_surface does not have a role. Once a wl_surface is given a
//! role, it is set permanently for the whole lifetime of the
//! wl_surface object. Giving the current role again is allowed,
//! unless explicitly forbidden by the relevant interface
//! specification.
//!
//! Surface roles are given by requests in other interfaces such as
//! wl_pointer.set_cursor. The request should explicitly mention
//! that this request gives a role to a wl_surface. Often, this
//! request also creates a new protocol object that represents the
//! role and adds additional functionality to wl_surface. When a
//! client wants to destroy a wl_surface, they must destroy this role
//! object before the wl_surface, otherwise a defunct_role_object error is
//! sent.
//!
//! Destroying the role object does not remove the role from the
//! wl_surface, but it may stop the wl_surface from "playing the role".
//! For instance, if a wl_subsurface object is destroyed, the wl_surface
//! it was created for will be unmapped and forget its position and
//! z-order. It is allowed to create a wl_subsurface for the same
//! wl_surface again, but it is not allowed to use the wl_surface as
//! a cursor (cursor is a different role than sub-surface, and role
//! switching is not allowed).

// TODO(ArnoDarkrose): implement the rest of the requests and events for wl_surface

use crate::sys::wire::{Message, MessageBuffer};

pub mod request {
    use super::*;
    use crate::{
        interface::{ObjectParent, Request},
        sys::{
            HasObjectType, ObjectType,
            object::{WlObjectHandle, buffer::WlBuffer, callback::WlCallback, region::WlRegion},
            object_storage::WlObjectStorage,
            wire::OpCode,
        },
    };

    /// Deletes the surface and invalidates its object ID.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Destroy;

    impl HasObjectType for Destroy {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for Destroy {
        const CODE: OpCode = 0;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf).opcode(Self::CODE).build()
        }
    }

    /// Set a buffer as the content of this surface.
    ///
    /// The new size of the surface is calculated based on the buffer
    /// size transformed by the inverse buffer_transform and the
    /// inverse buffer_scale. This means that at commit time the supplied
    /// buffer size must be an integer multiple of the buffer_scale. If
    /// that's not the case, an invalid_size error is sent.
    ///
    /// The x and y arguments specify the location of the new pending
    /// buffer's upper left corner, relative to the current buffer's upper
    /// left corner, in surface-local coordinates. In other words, the
    /// x and y, combined with the new surface size define in which
    /// directions the surface's size changes. Setting anything other than 0
    /// as x and y arguments is discouraged, and should instead be replaced
    /// with using the separate wl_surface.offset request.
    ///
    /// When the bound wl_surface version is 5 or higher, passing any
    /// non-zero x or y is a protocol violation, and will result in an
    /// 'invalid_offset' error being raised. The x and y arguments are ignored
    /// and do not change the pending state. To achieve equivalent semantics,
    /// use wl_surface.offset.
    ///
    /// Surface contents are double-buffered state, see wl_surface.commit.
    ///
    /// The initial surface contents are void; there is no content.
    /// wl_surface.attach assigns the given wl_buffer as the pending
    /// wl_buffer. wl_surface.commit makes the pending wl_buffer the new
    /// surface contents, and the size of the surface becomes the size
    /// calculated from the wl_buffer, as described above. After commit,
    /// there is no pending buffer until the next attach.
    ///
    /// Committing a pending wl_buffer allows the compositor to read the
    /// pixels in the wl_buffer. The compositor may access the pixels at
    /// any time after the wl_surface.commit request. When the compositor
    /// will not access the pixels anymore, it will send the
    /// wl_buffer.release event. Only after receiving wl_buffer.release,
    /// the client may reuse the wl_buffer. A wl_buffer that has been
    /// attached and then replaced by another attach instead of committed
    /// will not receive a release event, and is not used by the
    /// compositor.
    ///
    /// If a pending wl_buffer has been committed to more than one wl_surface,
    /// the delivery of wl_buffer.release events becomes undefined. A well
    /// behaved client should not rely on wl_buffer.release events in this
    /// case. Alternatively, a client could create multiple wl_buffer objects
    /// from the same backing storage or use wp_linux_buffer_release.
    ///
    /// Destroying the wl_buffer after wl_buffer.release does not change
    /// the surface contents. Destroying the wl_buffer before wl_buffer.release
    /// is allowed as long as the underlying buffer storage isn't re-used (this
    /// can happen e.g. on client process termination). However, if the client
    /// destroys the wl_buffer before receiving the wl_buffer.release event and
    /// mutates the underlying buffer storage, the surface contents become
    /// undefined immediately.
    ///
    /// If wl_surface.attach is sent with a NULL wl_buffer, the
    /// following wl_surface.commit will remove the surface content.
    ///
    /// If a pending wl_buffer has been destroyed, the result is not specified.
    /// Many compositors are known to remove the surface content on the following
    /// wl_surface.commit, but this behaviour is not universal. Clients seeking to
    /// maximise compatibility should not destroy pending buffers and should
    /// ensure that they explicitly remove content from surfaces, even after
    /// destroying buffers.
    #[derive(Clone, Copy, Default, Debug)]
    pub struct Attach {
        /// buffer of surface contents
        pub buffer: Option<WlObjectHandle<WlBuffer>>,
        /// surface-local x coordinate
        pub x: i32,
        /// surface-local y coordinate
        pub y: i32,
    }

    impl HasObjectType for Attach {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for Attach {
        const CODE: OpCode = 1;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            storage: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf)
                .opcode(Self::CODE)
                .maybe_object(self.buffer.map(|h| storage.object(h).proxy()))
                .int(self.x)
                .int(self.x)
                .build()
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Damage {
        /// surface-local x coordinate
        pub x: i32,
        /// surface-local y coordinate
        pub y: i32,
        /// width of damage rectangle
        pub width: i32,
        /// height of damage rectangle
        pub height: i32,
    }

    impl HasObjectType for Damage {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for Damage {
        const CODE: OpCode = 2;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf)
                .opcode(Self::CODE)
                .int(self.x)
                .int(self.y)
                .int(self.width)
                .int(self.height)
                .build()
        }
    }

    /// Request a notification when it is a good time to start drawing a new
    /// frame, by creating a frame callback. This is useful for throttling
    /// redrawing operations, and driving animations.
    ///
    /// When a client is animating on a wl_surface, it can use the 'frame'
    /// request to get notified when it is a good time to draw and commit the
    /// next frame of animation. If the client commits an update earlier than
    /// that, it is likely that some updates will not make it to the display,
    /// and the client is wasting resources by drawing too often.
    ///
    /// The frame request will take effect on the next wl_surface.commit.
    /// The notification will only be posted for one frame unless
    /// requested again. For a wl_surface, the notifications are posted in
    /// the order the frame requests were committed.
    ///
    /// The server must send the notifications so that a client
    /// will not send excessive updates, while still allowing
    /// the highest possible update rate for clients that wait for the reply
    /// before drawing again. The server should give some time for the client
    /// to draw and commit after sending the frame callback events to let it
    /// hit the next output refresh.
    ///
    /// A server should avoid signaling the frame callbacks if the
    /// surface is not visible in any way, e.g. the surface is off-screen,
    /// or completely obscured by other opaque surfaces.
    ///
    /// The object returned by this request will be destroyed by the
    /// compositor after the callback is fired and as such the client must not
    /// attempt to use it after that point.
    ///
    /// The callback_data passed in the callback is the current time, in
    /// milliseconds, with an undefined base.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
    pub struct Frame;

    impl ObjectParent for Frame {
        type Child = WlCallback;
    }

    impl HasObjectType for Frame {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for Frame {
        const CODE: OpCode = 3;
        const OUTGOING_INTERFACE: Option<ObjectType> = Some(ObjectType::Callback);

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf).opcode(Self::CODE).new_id().build()
        }
    }

    /// This request sets the region of the surface that contains
    /// opaque content.
    ///
    /// The opaque region is an optimization hint for the compositor
    /// that lets it optimize the redrawing of content behind opaque
    /// regions.  Setting an opaque region is not required for correct
    /// behaviour, but marking transparent content as opaque will result
    /// in repaint artifacts.
    ///
    /// The opaque region is specified in surface-local coordinates.
    ///
    /// The compositor ignores the parts of the opaque region that fall
    /// outside of the surface.
    ///
    /// Opaque region is double-buffered state, see wl_surface.commit.
    ///
    /// wl_surface.set_opaque_region changes the pending opaque region.
    /// wl_surface.commit copies the pending region to the current region.
    /// Otherwise, the pending and current regions are never changed.
    ///
    /// The initial value for an opaque region is empty. Setting the pending
    /// opaque region has copy semantics, and the wl_region object can be
    /// destroyed immediately. A NULL wl_region causes the pending opaque
    /// region to be set to empty.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct SetOpaqueRegion {
        /// opaque region of the surface
        pub region: Option<WlObjectHandle<WlRegion>>,
    }

    impl HasObjectType for SetOpaqueRegion {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for SetOpaqueRegion {
        const CODE: OpCode = 4;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            storage: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf)
                .opcode(Self::CODE)
                .maybe_object(self.region.map(|h| storage.object(h).proxy()))
                .build()
        }
    }

    /// This request sets the region of the surface that can receive
    /// pointer and touch events.
    ///
    /// Input events happening outside of this region will try the next
    /// surface in the server surface stack. The compositor ignores the
    /// parts of the input region that fall outside of the surface.
    ///
    /// The input region is specified in surface-local coordinates.
    ///
    /// Input region is double-buffered state, see wl_surface.commit.
    ///
    /// wl_surface.set_input_region changes the pending input region.
    /// wl_surface.commit copies the pending region to the current region.
    /// Otherwise the pending and current regions are never changed,
    /// except cursor and icon surfaces are special cases, see
    /// wl_pointer.set_cursor and wl_data_device.start_drag.
    ///
    /// The initial value for an input region is infinite. That means the
    /// whole surface will accept input. Setting the pending input region
    /// has copy semantics, and the wl_region object can be destroyed
    /// immediately. A NULL wl_region causes the input region to be set
    /// to infinite.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct SetInputRegion {
        /// input region of the surface
        pub region: Option<WlObjectHandle<WlRegion>>,
    }

    impl HasObjectType for SetInputRegion {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for SetInputRegion {
        const CODE: OpCode = 5;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            storage: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf)
                .opcode(Self::CODE)
                .maybe_object(self.region.map(|h| storage.object(h).proxy()))
                .build()
        }
    }

    /// Surface state (input, opaque, and damage regions, attached buffers,
    /// etc.) is double-buffered. Protocol requests modify the pending state,
    /// as opposed to the active state in use by the compositor.
    ///
    /// A commit request atomically creates a content update from the pending
    /// state, even if the pending state has not been touched. The content
    /// update is placed in a queue until it becomes active. After commit, the
    /// new pending state is as documented for each related request.
    ///
    /// When the content update is applied, the wl_buffer is applied before all
    /// other state. This means that all coordinates in double-buffered state
    /// are relative to the newly attached wl_buffers, except for
    /// wl_surface.attach itself. If there is no newly attached wl_buffer, the
    /// coordinates are relative to the previous content update.
    ///
    /// All requests that need a commit to become effective are documented
    /// to affect double-buffered state.
    ///
    /// Other interfaces may add further double-buffered surface state.
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Default, Eq, Ord)]
    pub struct Commit;

    impl HasObjectType for Commit {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for Commit {
        const CODE: OpCode = 6;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf).opcode(Self::CODE).build()
        }
    }

    /// Sets the buffer scaling factor
    ///
    /// This request sets an optional scaling factor on how the compositor
    /// interprets the contents of the buffer attached to the window.
    ///
    /// Buffer scale is double-buffered state, see wl_surface.commit.
    ///
    /// A newly created surface has its buffer scale set to 1.
    ///
    /// wl_surface.set_buffer_scale changes the pending buffer scale.
    /// wl_surface.commit copies the pending buffer scale to the current one.
    /// Otherwise, the pending and current values are never changed.
    ///
    /// The purpose of this request is to allow clients to supply higher
    /// resolution buffer data for use on high resolution outputs. It is
    /// intended that you pick the same buffer scale as the scale of the
    /// output that the surface is displayed on. This means the compositor
    /// can avoid scaling when rendering the surface on that output.
    ///
    /// Note that if the scale is larger than 1, then you have to attach
    /// a buffer that is larger (by a factor of scale in each dimension)
    /// than the desired surface size.
    ///
    /// If scale is not positive the invalid_scale protocol error is
    /// raised.
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Default, Eq, Ord)]
    pub struct SetBufferScale {
        pub scale: i32,
    }

    impl HasObjectType for SetBufferScale {
        const OBJECT_TYPE: ObjectType = ObjectType::Surface;
    }

    impl<'s> Request<'s> for SetBufferScale {
        const CODE: OpCode = 8;

        fn build_message<'m>(
            self,
            buf: &'m mut impl MessageBuffer,
            _: &'m WlObjectStorage,
        ) -> Message<'m>
        where
            's: 'm,
        {
            Message::builder(buf)
                .opcode(Self::CODE)
                .int(self.scale)
                .build()
        }
    }
}

pub mod event {
    use super::*;
    use crate::{
        interface::Event,
        sys::{proxy::WlProxyQuery, wire::OpCode},
    };

    /// This is emitted whenever a surface's creation, movement, or resizing
    /// results in some part of it being within the scanout region of an
    /// output.
    ///
    /// Note that a surface may be overlapping with zero or more outputs.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Enter {
        /// Output entered by the surface
        pub output: WlProxyQuery,
    }

    impl<'s> Event<'s> for Enter {
        const CODE: OpCode = 0;

        fn from_message(message: Message<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();
            let output = unsafe { reader.read::<WlProxyQuery>()? };

            Some(Self { output })
        }
    }

    ///This is emitted whenever a surface's creation, movement, or resizing
    ///results in it no longer having any part of it within the scanout region
    ///of an output.
    ///
    ///Clients should not use the number of outputs the surface is on for frame
    ///throttling purposes. The surface might be hidden even if no leave event
    ///has been sent, and the compositor might expect new surface content
    ///updates even if no enter event has been sent. The frame event should be
    ///used instead.
    #[derive(Debug, Clone, Copy)]
    pub struct Leave {
        /// output left by the surface
        pub output: WlProxyQuery,
    }

    impl<'s> Event<'s> for Leave {
        const CODE: OpCode = 1;

        fn from_message(message: Message<'s>) -> Option<Self> {
            if message.opcode != Self::CODE {
                return None;
            }

            let mut reader = message.reader();
            let output = unsafe { reader.read::<WlProxyQuery>()? };

            Some(Self { output })
        }
    }
}

pub mod wl_enum {
    /// These errors can be emitted in response to wl_surface requests.
    #[derive(Clone, Debug, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
    pub enum Error {
        /// buffer scale value is invalid
        InvalidScale = 0,
        /// buffer transform value is invalid
        InvalidTransform = 1,
        /// buffer size is invalid
        InvalidSize = 2,
        /// buffer offset is invalid
        InvalidOffset = 3,
        /// surface was destroyed before its role object
        DefunctRoleObject = 4,
    }
}
