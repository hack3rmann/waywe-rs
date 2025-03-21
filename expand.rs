pub mod generated {
    pub enum WlObjectType {
        #[default]
        Display,
        Registry,
        Callback,
        Compositor,
        ShmPool,
        Shm,
        Buffer,
        DataOffer,
        DataSource,
        DataDevice,
        DataDeviceManager,
        Shell,
        ShellSurface,
        Surface,
        Seat,
        Pointer,
        Keyboard,
        Touch,
        Output,
        Region,
        Subcompositor,
        Subsurface,
        XdgWmBase,
        XdgPositioner,
        XdgSurface,
        XdgToplevel,
        XdgPopup,
        WpViewporter,
        WpViewport,
        LayerShell,
        LayerSurface,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for WlObjectType {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    WlObjectType::Display => "Display",
                    WlObjectType::Registry => "Registry",
                    WlObjectType::Callback => "Callback",
                    WlObjectType::Compositor => "Compositor",
                    WlObjectType::ShmPool => "ShmPool",
                    WlObjectType::Shm => "Shm",
                    WlObjectType::Buffer => "Buffer",
                    WlObjectType::DataOffer => "DataOffer",
                    WlObjectType::DataSource => "DataSource",
                    WlObjectType::DataDevice => "DataDevice",
                    WlObjectType::DataDeviceManager => "DataDeviceManager",
                    WlObjectType::Shell => "Shell",
                    WlObjectType::ShellSurface => "ShellSurface",
                    WlObjectType::Surface => "Surface",
                    WlObjectType::Seat => "Seat",
                    WlObjectType::Pointer => "Pointer",
                    WlObjectType::Keyboard => "Keyboard",
                    WlObjectType::Touch => "Touch",
                    WlObjectType::Output => "Output",
                    WlObjectType::Region => "Region",
                    WlObjectType::Subcompositor => "Subcompositor",
                    WlObjectType::Subsurface => "Subsurface",
                    WlObjectType::XdgWmBase => "XdgWmBase",
                    WlObjectType::XdgPositioner => "XdgPositioner",
                    WlObjectType::XdgSurface => "XdgSurface",
                    WlObjectType::XdgToplevel => "XdgToplevel",
                    WlObjectType::XdgPopup => "XdgPopup",
                    WlObjectType::WpViewporter => "WpViewporter",
                    WlObjectType::WpViewport => "WpViewport",
                    WlObjectType::LayerShell => "LayerShell",
                    WlObjectType::LayerSurface => "LayerSurface",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for WlObjectType {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for WlObjectType {
        #[inline]
        fn eq(&self, other: &WlObjectType) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for WlObjectType {
        #[inline]
        fn default() -> WlObjectType {
            Self::Display
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for WlObjectType {
        #[inline]
        fn clone(&self) -> WlObjectType {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for WlObjectType {}
    #[automatically_derived]
    impl ::core::cmp::Eq for WlObjectType {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for WlObjectType {
        #[inline]
        fn partial_cmp(
            &self,
            other: &WlObjectType,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for WlObjectType {
        #[inline]
        fn cmp(&self, other: &WlObjectType) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for WlObjectType {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    pub mod wayland {
        pub mod display {
            pub mod request {
                pub struct Sync;
                impl crate::interface::ObjectParent for Sync {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Callback;
                }
                impl crate::object::HasObjectType for Sync {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Display;
                }
                impl<'s> crate::interface::Request<'s> for Sync {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Callback,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct GetRegistry;
                impl crate::interface::ObjectParent for GetRegistry {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Registry;
                }
                impl crate::object::HasObjectType for GetRegistry {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Display;
                }
                impl<'s> crate::interface::Request<'s> for GetRegistry {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Registry,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                /**These errors are global and can be emitted in response to any
	server request.*/
                #[repr(u32)]
                pub enum Error {
                    ///server couldn't find object
                    InvalidObject = 0u32,
                    ///method doesn't exist on the specified interface or malformed request
                    InvalidMethod = 1u32,
                    ///server is out of memory
                    NoMemory = 2u32,
                    ///implementation error in compositor
                    Implementation = 3u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::InvalidObject => "InvalidObject",
                                Error::InvalidMethod => "InvalidMethod",
                                Error::NoMemory => "NoMemory",
                                Error::Implementation => "Implementation",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidObject,
                                1u32 => Self::InvalidMethod,
                                2u32 => Self::NoMemory,
                                3u32 => Self::Implementation,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod registry {
            pub mod request {
                pub struct Bind<'s> {
                    name: u32,
                    interface: &'s ::std::ffi::CStr,
                    version: u32,
                }
                impl crate::object::HasObjectType for Bind<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Registry;
                }
                impl<'s> crate::interface::Request<'s> for Bind<'s> {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.name)
                            .str(self.interface)
                            .uint(self.version)
                            .new_id()
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {}
        }
        pub mod callback {
            pub mod request {}
            pub mod event {}
            pub mod wl_enum {}
        }
        pub mod compositor {
            pub mod request {
                pub struct CreateSurface;
                impl crate::interface::ObjectParent for CreateSurface {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl crate::object::HasObjectType for CreateSurface {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Compositor;
                }
                impl<'s> crate::interface::Request<'s> for CreateSurface {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Surface,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct CreateRegion;
                impl crate::interface::ObjectParent for CreateRegion {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Region;
                }
                impl crate::object::HasObjectType for CreateRegion {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Compositor;
                }
                impl<'s> crate::interface::Request<'s> for CreateRegion {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Region,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {}
        }
        pub mod shm_pool {
            pub mod request {
                pub struct CreateBuffer {
                    offset: i32,
                    width: i32,
                    height: i32,
                    stride: i32,
                    format: u32,
                }
                impl crate::interface::ObjectParent for CreateBuffer {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Buffer;
                }
                impl crate::object::HasObjectType for CreateBuffer {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShmPool;
                }
                impl<'s> crate::interface::Request<'s> for CreateBuffer {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Buffer,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .int(self.offset)
                            .int(self.width)
                            .int(self.height)
                            .int(self.stride)
                            .uint(self.format)
                            .build()
                    }
                }
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShmPool;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct Resize {
                    size: i32,
                }
                impl crate::object::HasObjectType for Resize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShmPool;
                }
                impl<'s> crate::interface::Request<'s> for Resize {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.size)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {}
        }
        pub mod shm {
            pub mod request {
                pub struct CreatePool<'s> {
                    fd: ::std::os::fd::BorrowedFd<'s>,
                    size: i32,
                }
                impl crate::interface::ObjectParent for CreatePool<'_> {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShmPool;
                }
                impl crate::object::HasObjectType for CreatePool<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Shm;
                }
                impl<'s> crate::interface::Request<'s> for CreatePool<'s> {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::ShmPool,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .fd(self.fd)
                            .int(self.size)
                            .build()
                    }
                }
                pub struct Release;
                impl crate::object::HasObjectType for Release {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Shm;
                }
                impl<'s> crate::interface::Request<'s> for Release {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                ///These errors can be emitted in response to wl_shm requests.
                #[repr(u32)]
                pub enum Error {
                    ///buffer format is not known
                    InvalidFormat = 0u32,
                    ///invalid size or stride during pool or buffer creation
                    InvalidStride = 1u32,
                    ///mmapping the file descriptor failed
                    InvalidFd = 2u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::InvalidFormat => "InvalidFormat",
                                Error::InvalidStride => "InvalidStride",
                                Error::InvalidFd => "InvalidFd",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidFormat,
                                1u32 => Self::InvalidStride,
                                2u32 => Self::InvalidFd,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
                /**This describes the memory layout of an individual pixel.

	All renderers should support argb8888 and xrgb8888 but any other
	formats are optional and may not be supported by the particular
	renderer in use.

	The drm format codes match the macros defined in drm_fourcc.h, except
	argb8888 and xrgb8888. The formats actually supported by the compositor
	will be reported by the format event.

	For all wl_shm formats and unless specified in another protocol
	extension, pre-multiplied alpha is used for pixel values.*/
                #[repr(u32)]
                pub enum Format {
                    ///32-bit ARGB format, [31:0] A:R:G:B 8:8:8:8 little endian
                    Argb8888 = 0u32,
                    ///32-bit RGB format, [31:0] x:R:G:B 8:8:8:8 little endian
                    Xrgb8888 = 1u32,
                    ///8-bit color index format, [7:0] C
                    C8 = 538982467u32,
                    ///8-bit RGB format, [7:0] R:G:B 3:3:2
                    Rgb332 = 943867730u32,
                    ///8-bit BGR format, [7:0] B:G:R 2:3:3
                    Bgr233 = 944916290u32,
                    ///16-bit xRGB format, [15:0] x:R:G:B 4:4:4:4 little endian
                    Xrgb4444 = 842093144u32,
                    ///16-bit xBGR format, [15:0] x:B:G:R 4:4:4:4 little endian
                    Xbgr4444 = 842089048u32,
                    ///16-bit RGBx format, [15:0] R:G:B:x 4:4:4:4 little endian
                    Rgbx4444 = 842094674u32,
                    ///16-bit BGRx format, [15:0] B:G:R:x 4:4:4:4 little endian
                    Bgrx4444 = 842094658u32,
                    ///16-bit ARGB format, [15:0] A:R:G:B 4:4:4:4 little endian
                    Argb4444 = 842093121u32,
                    ///16-bit ABGR format, [15:0] A:B:G:R 4:4:4:4 little endian
                    Abgr4444 = 842089025u32,
                    ///16-bit RBGA format, [15:0] R:G:B:A 4:4:4:4 little endian
                    Rgba4444 = 842088786u32,
                    ///16-bit BGRA format, [15:0] B:G:R:A 4:4:4:4 little endian
                    Bgra4444 = 842088770u32,
                    ///16-bit xRGB format, [15:0] x:R:G:B 1:5:5:5 little endian
                    Xrgb1555 = 892424792u32,
                    ///16-bit xBGR 1555 format, [15:0] x:B:G:R 1:5:5:5 little endian
                    Xbgr1555 = 892420696u32,
                    ///16-bit RGBx 5551 format, [15:0] R:G:B:x 5:5:5:1 little endian
                    Rgbx5551 = 892426322u32,
                    ///16-bit BGRx 5551 format, [15:0] B:G:R:x 5:5:5:1 little endian
                    Bgrx5551 = 892426306u32,
                    ///16-bit ARGB 1555 format, [15:0] A:R:G:B 1:5:5:5 little endian
                    Argb1555 = 892424769u32,
                    ///16-bit ABGR 1555 format, [15:0] A:B:G:R 1:5:5:5 little endian
                    Abgr1555 = 892420673u32,
                    ///16-bit RGBA 5551 format, [15:0] R:G:B:A 5:5:5:1 little endian
                    Rgba5551 = 892420434u32,
                    ///16-bit BGRA 5551 format, [15:0] B:G:R:A 5:5:5:1 little endian
                    Bgra5551 = 892420418u32,
                    ///16-bit RGB 565 format, [15:0] R:G:B 5:6:5 little endian
                    Rgb565 = 909199186u32,
                    ///16-bit BGR 565 format, [15:0] B:G:R 5:6:5 little endian
                    Bgr565 = 909199170u32,
                    ///24-bit RGB format, [23:0] R:G:B little endian
                    Rgb888 = 875710290u32,
                    ///24-bit BGR format, [23:0] B:G:R little endian
                    Bgr888 = 875710274u32,
                    ///32-bit xBGR format, [31:0] x:B:G:R 8:8:8:8 little endian
                    Xbgr8888 = 875709016u32,
                    ///32-bit RGBx format, [31:0] R:G:B:x 8:8:8:8 little endian
                    Rgbx8888 = 875714642u32,
                    ///32-bit BGRx format, [31:0] B:G:R:x 8:8:8:8 little endian
                    Bgrx8888 = 875714626u32,
                    ///32-bit ABGR format, [31:0] A:B:G:R 8:8:8:8 little endian
                    Abgr8888 = 875708993u32,
                    ///32-bit RGBA format, [31:0] R:G:B:A 8:8:8:8 little endian
                    Rgba8888 = 875708754u32,
                    ///32-bit BGRA format, [31:0] B:G:R:A 8:8:8:8 little endian
                    Bgra8888 = 875708738u32,
                    ///32-bit xRGB format, [31:0] x:R:G:B 2:10:10:10 little endian
                    Xrgb2101010 = 808669784u32,
                    ///32-bit xBGR format, [31:0] x:B:G:R 2:10:10:10 little endian
                    Xbgr2101010 = 808665688u32,
                    ///32-bit RGBx format, [31:0] R:G:B:x 10:10:10:2 little endian
                    Rgbx1010102 = 808671314u32,
                    ///32-bit BGRx format, [31:0] B:G:R:x 10:10:10:2 little endian
                    Bgrx1010102 = 808671298u32,
                    ///32-bit ARGB format, [31:0] A:R:G:B 2:10:10:10 little endian
                    Argb2101010 = 808669761u32,
                    ///32-bit ABGR format, [31:0] A:B:G:R 2:10:10:10 little endian
                    Abgr2101010 = 808665665u32,
                    ///32-bit RGBA format, [31:0] R:G:B:A 10:10:10:2 little endian
                    Rgba1010102 = 808665426u32,
                    ///32-bit BGRA format, [31:0] B:G:R:A 10:10:10:2 little endian
                    Bgra1010102 = 808665410u32,
                    ///packed YCbCr format, [31:0] Cr0:Y1:Cb0:Y0 8:8:8:8 little endian
                    Yuyv = 1448695129u32,
                    ///packed YCbCr format, [31:0] Cb0:Y1:Cr0:Y0 8:8:8:8 little endian
                    Yvyu = 1431918169u32,
                    ///packed YCbCr format, [31:0] Y1:Cr0:Y0:Cb0 8:8:8:8 little endian
                    Uyvy = 1498831189u32,
                    ///packed YCbCr format, [31:0] Y1:Cb0:Y0:Cr0 8:8:8:8 little endian
                    Vyuy = 1498765654u32,
                    ///packed AYCbCr format, [31:0] A:Y:Cb:Cr 8:8:8:8 little endian
                    Ayuv = 1448433985u32,
                    ///2 plane YCbCr Cr:Cb format, 2x2 subsampled Cr:Cb plane
                    Nv12 = 842094158u32,
                    ///2 plane YCbCr Cb:Cr format, 2x2 subsampled Cb:Cr plane
                    Nv21 = 825382478u32,
                    ///2 plane YCbCr Cr:Cb format, 2x1 subsampled Cr:Cb plane
                    Nv16 = 909203022u32,
                    ///2 plane YCbCr Cb:Cr format, 2x1 subsampled Cb:Cr plane
                    Nv61 = 825644622u32,
                    ///3 plane YCbCr format, 4x4 subsampled Cb (1) and Cr (2) planes
                    Yuv410 = 961959257u32,
                    ///3 plane YCbCr format, 4x4 subsampled Cr (1) and Cb (2) planes
                    Yvu410 = 961893977u32,
                    ///3 plane YCbCr format, 4x1 subsampled Cb (1) and Cr (2) planes
                    Yuv411 = 825316697u32,
                    ///3 plane YCbCr format, 4x1 subsampled Cr (1) and Cb (2) planes
                    Yvu411 = 825316953u32,
                    ///3 plane YCbCr format, 2x2 subsampled Cb (1) and Cr (2) planes
                    Yuv420 = 842093913u32,
                    ///3 plane YCbCr format, 2x2 subsampled Cr (1) and Cb (2) planes
                    Yvu420 = 842094169u32,
                    ///3 plane YCbCr format, 2x1 subsampled Cb (1) and Cr (2) planes
                    Yuv422 = 909202777u32,
                    ///3 plane YCbCr format, 2x1 subsampled Cr (1) and Cb (2) planes
                    Yvu422 = 909203033u32,
                    ///3 plane YCbCr format, non-subsampled Cb (1) and Cr (2) planes
                    Yuv444 = 875713881u32,
                    ///3 plane YCbCr format, non-subsampled Cr (1) and Cb (2) planes
                    Yvu444 = 875714137u32,
                    ///[7:0] R
                    R8 = 538982482u32,
                    ///[15:0] R little endian
                    R16 = 540422482u32,
                    ///[15:0] R:G 8:8 little endian
                    Rg88 = 943212370u32,
                    ///[15:0] G:R 8:8 little endian
                    Gr88 = 943215175u32,
                    ///[31:0] R:G 16:16 little endian
                    Rg1616 = 842221394u32,
                    ///[31:0] G:R 16:16 little endian
                    Gr1616 = 842224199u32,
                    ///[63:0] x:R:G:B 16:16:16:16 little endian
                    Xrgb16161616F = 1211388504u32,
                    ///[63:0] x:B:G:R 16:16:16:16 little endian
                    Xbgr16161616F = 1211384408u32,
                    ///[63:0] A:R:G:B 16:16:16:16 little endian
                    Argb16161616F = 1211388481u32,
                    ///[63:0] A:B:G:R 16:16:16:16 little endian
                    Abgr16161616F = 1211384385u32,
                    ///[31:0] X:Y:Cb:Cr 8:8:8:8 little endian
                    Xyuv8888 = 1448434008u32,
                    ///[23:0] Cr:Cb:Y 8:8:8 little endian
                    Vuy888 = 875713878u32,
                    ///Y followed by U then V, 10:10:10. Non-linear modifier only
                    Vuy101010 = 808670550u32,
                    ///[63:0] Cr0:0:Y1:0:Cb0:0:Y0:0 10:6:10:6:10:6:10:6 little endian per 2 Y pixels
                    Y210 = 808530521u32,
                    ///[63:0] Cr0:0:Y1:0:Cb0:0:Y0:0 12:4:12:4:12:4:12:4 little endian per 2 Y pixels
                    Y212 = 842084953u32,
                    ///[63:0] Cr0:Y1:Cb0:Y0 16:16:16:16 little endian per 2 Y pixels
                    Y216 = 909193817u32,
                    ///[31:0] A:Cr:Y:Cb 2:10:10:10 little endian
                    Y410 = 808531033u32,
                    ///[63:0] A:0:Cr:0:Y:0:Cb:0 12:4:12:4:12:4:12:4 little endian
                    Y412 = 842085465u32,
                    ///[63:0] A:Cr:Y:Cb 16:16:16:16 little endian
                    Y416 = 909194329u32,
                    ///[31:0] X:Cr:Y:Cb 2:10:10:10 little endian
                    Xvyu2101010 = 808670808u32,
                    ///[63:0] X:0:Cr:0:Y:0:Cb:0 12:4:12:4:12:4:12:4 little endian
                    Xvyu1216161616 = 909334104u32,
                    ///[63:0] X:Cr:Y:Cb 16:16:16:16 little endian
                    Xvyu16161616 = 942954072u32,
                    ///[63:0]   A3:A2:Y3:0:Cr0:0:Y2:0:A1:A0:Y1:0:Cb0:0:Y0:0  1:1:8:2:8:2:8:2:1:1:8:2:8:2:8:2 little endian
                    Y0L0 = 810299481u32,
                    ///[63:0]   X3:X2:Y3:0:Cr0:0:Y2:0:X1:X0:Y1:0:Cb0:0:Y0:0  1:1:8:2:8:2:8:2:1:1:8:2:8:2:8:2 little endian
                    X0L0 = 810299480u32,
                    ///[63:0]   A3:A2:Y3:Cr0:Y2:A1:A0:Y1:Cb0:Y0  1:1:10:10:10:1:1:10:10:10 little endian
                    Y0L2 = 843853913u32,
                    ///[63:0]   X3:X2:Y3:Cr0:Y2:X1:X0:Y1:Cb0:Y0  1:1:10:10:10:1:1:10:10:10 little endian
                    X0L2 = 843853912u32,
                    Yuv4208Bit = 942691673u32,
                    Yuv42010Bit = 808539481u32,
                    Xrgb8888A8 = 943805016u32,
                    Xbgr8888A8 = 943800920u32,
                    Rgbx8888A8 = 943806546u32,
                    Bgrx8888A8 = 943806530u32,
                    Rgb888A8 = 943798354u32,
                    Bgr888A8 = 943798338u32,
                    Rgb565A8 = 943797586u32,
                    Bgr565A8 = 943797570u32,
                    ///non-subsampled Cr:Cb plane
                    Nv24 = 875714126u32,
                    ///non-subsampled Cb:Cr plane
                    Nv42 = 842290766u32,
                    ///2x1 subsampled Cr:Cb plane, 10 bit per channel
                    P210 = 808530512u32,
                    ///2x2 subsampled Cr:Cb plane 10 bits per channel
                    P010 = 808530000u32,
                    ///2x2 subsampled Cr:Cb plane 12 bits per channel
                    P012 = 842084432u32,
                    ///2x2 subsampled Cr:Cb plane 16 bits per channel
                    P016 = 909193296u32,
                    ///[63:0] A:x:B:x:G:x:R:x 10:6:10:6:10:6:10:6 little endian
                    Axbxgxrx106106106106 = 808534593u32,
                    ///2x2 subsampled Cr:Cb plane
                    Nv15 = 892425806u32,
                    Q410 = 808531025u32,
                    Q401 = 825242705u32,
                    ///[63:0] x:R:G:B 16:16:16:16 little endian
                    Xrgb16161616 = 942953048u32,
                    ///[63:0] x:B:G:R 16:16:16:16 little endian
                    Xbgr16161616 = 942948952u32,
                    ///[63:0] A:R:G:B 16:16:16:16 little endian
                    Argb16161616 = 942953025u32,
                    ///[63:0] A:B:G:R 16:16:16:16 little endian
                    Abgr16161616 = 942948929u32,
                    ///[7:0] C0:C1:C2:C3:C4:C5:C6:C7 1:1:1:1:1:1:1:1 eight pixels/byte
                    C1 = 538980675u32,
                    ///[7:0] C0:C1:C2:C3 2:2:2:2 four pixels/byte
                    C2 = 538980931u32,
                    ///[7:0] C0:C1 4:4 two pixels/byte
                    C4 = 538981443u32,
                    ///[7:0] D0:D1:D2:D3:D4:D5:D6:D7 1:1:1:1:1:1:1:1 eight pixels/byte
                    D1 = 538980676u32,
                    ///[7:0] D0:D1:D2:D3 2:2:2:2 four pixels/byte
                    D2 = 538980932u32,
                    ///[7:0] D0:D1 4:4 two pixels/byte
                    D4 = 538981444u32,
                    ///[7:0] D
                    D8 = 538982468u32,
                    ///[7:0] R0:R1:R2:R3:R4:R5:R6:R7 1:1:1:1:1:1:1:1 eight pixels/byte
                    R1 = 538980690u32,
                    ///[7:0] R0:R1:R2:R3 2:2:2:2 four pixels/byte
                    R2 = 538980946u32,
                    ///[7:0] R0:R1 4:4 two pixels/byte
                    R4 = 538981458u32,
                    ///[15:0] x:R 6:10 little endian
                    R10 = 540029266u32,
                    ///[15:0] x:R 4:12 little endian
                    R12 = 540160338u32,
                    ///[31:0] A:Cr:Cb:Y 8:8:8:8 little endian
                    Avuy8888 = 1498764865u32,
                    ///[31:0] X:Cr:Cb:Y 8:8:8:8 little endian
                    Xvuy8888 = 1498764888u32,
                    ///2x2 subsampled Cr:Cb plane 10 bits per channel packed
                    P030 = 808661072u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Format {
                    #[inline]
                    fn clone(&self) -> Format {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Format {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Format::Argb8888 => "Argb8888",
                                Format::Xrgb8888 => "Xrgb8888",
                                Format::C8 => "C8",
                                Format::Rgb332 => "Rgb332",
                                Format::Bgr233 => "Bgr233",
                                Format::Xrgb4444 => "Xrgb4444",
                                Format::Xbgr4444 => "Xbgr4444",
                                Format::Rgbx4444 => "Rgbx4444",
                                Format::Bgrx4444 => "Bgrx4444",
                                Format::Argb4444 => "Argb4444",
                                Format::Abgr4444 => "Abgr4444",
                                Format::Rgba4444 => "Rgba4444",
                                Format::Bgra4444 => "Bgra4444",
                                Format::Xrgb1555 => "Xrgb1555",
                                Format::Xbgr1555 => "Xbgr1555",
                                Format::Rgbx5551 => "Rgbx5551",
                                Format::Bgrx5551 => "Bgrx5551",
                                Format::Argb1555 => "Argb1555",
                                Format::Abgr1555 => "Abgr1555",
                                Format::Rgba5551 => "Rgba5551",
                                Format::Bgra5551 => "Bgra5551",
                                Format::Rgb565 => "Rgb565",
                                Format::Bgr565 => "Bgr565",
                                Format::Rgb888 => "Rgb888",
                                Format::Bgr888 => "Bgr888",
                                Format::Xbgr8888 => "Xbgr8888",
                                Format::Rgbx8888 => "Rgbx8888",
                                Format::Bgrx8888 => "Bgrx8888",
                                Format::Abgr8888 => "Abgr8888",
                                Format::Rgba8888 => "Rgba8888",
                                Format::Bgra8888 => "Bgra8888",
                                Format::Xrgb2101010 => "Xrgb2101010",
                                Format::Xbgr2101010 => "Xbgr2101010",
                                Format::Rgbx1010102 => "Rgbx1010102",
                                Format::Bgrx1010102 => "Bgrx1010102",
                                Format::Argb2101010 => "Argb2101010",
                                Format::Abgr2101010 => "Abgr2101010",
                                Format::Rgba1010102 => "Rgba1010102",
                                Format::Bgra1010102 => "Bgra1010102",
                                Format::Yuyv => "Yuyv",
                                Format::Yvyu => "Yvyu",
                                Format::Uyvy => "Uyvy",
                                Format::Vyuy => "Vyuy",
                                Format::Ayuv => "Ayuv",
                                Format::Nv12 => "Nv12",
                                Format::Nv21 => "Nv21",
                                Format::Nv16 => "Nv16",
                                Format::Nv61 => "Nv61",
                                Format::Yuv410 => "Yuv410",
                                Format::Yvu410 => "Yvu410",
                                Format::Yuv411 => "Yuv411",
                                Format::Yvu411 => "Yvu411",
                                Format::Yuv420 => "Yuv420",
                                Format::Yvu420 => "Yvu420",
                                Format::Yuv422 => "Yuv422",
                                Format::Yvu422 => "Yvu422",
                                Format::Yuv444 => "Yuv444",
                                Format::Yvu444 => "Yvu444",
                                Format::R8 => "R8",
                                Format::R16 => "R16",
                                Format::Rg88 => "Rg88",
                                Format::Gr88 => "Gr88",
                                Format::Rg1616 => "Rg1616",
                                Format::Gr1616 => "Gr1616",
                                Format::Xrgb16161616F => "Xrgb16161616F",
                                Format::Xbgr16161616F => "Xbgr16161616F",
                                Format::Argb16161616F => "Argb16161616F",
                                Format::Abgr16161616F => "Abgr16161616F",
                                Format::Xyuv8888 => "Xyuv8888",
                                Format::Vuy888 => "Vuy888",
                                Format::Vuy101010 => "Vuy101010",
                                Format::Y210 => "Y210",
                                Format::Y212 => "Y212",
                                Format::Y216 => "Y216",
                                Format::Y410 => "Y410",
                                Format::Y412 => "Y412",
                                Format::Y416 => "Y416",
                                Format::Xvyu2101010 => "Xvyu2101010",
                                Format::Xvyu1216161616 => "Xvyu1216161616",
                                Format::Xvyu16161616 => "Xvyu16161616",
                                Format::Y0L0 => "Y0L0",
                                Format::X0L0 => "X0L0",
                                Format::Y0L2 => "Y0L2",
                                Format::X0L2 => "X0L2",
                                Format::Yuv4208Bit => "Yuv4208Bit",
                                Format::Yuv42010Bit => "Yuv42010Bit",
                                Format::Xrgb8888A8 => "Xrgb8888A8",
                                Format::Xbgr8888A8 => "Xbgr8888A8",
                                Format::Rgbx8888A8 => "Rgbx8888A8",
                                Format::Bgrx8888A8 => "Bgrx8888A8",
                                Format::Rgb888A8 => "Rgb888A8",
                                Format::Bgr888A8 => "Bgr888A8",
                                Format::Rgb565A8 => "Rgb565A8",
                                Format::Bgr565A8 => "Bgr565A8",
                                Format::Nv24 => "Nv24",
                                Format::Nv42 => "Nv42",
                                Format::P210 => "P210",
                                Format::P010 => "P010",
                                Format::P012 => "P012",
                                Format::P016 => "P016",
                                Format::Axbxgxrx106106106106 => "Axbxgxrx106106106106",
                                Format::Nv15 => "Nv15",
                                Format::Q410 => "Q410",
                                Format::Q401 => "Q401",
                                Format::Xrgb16161616 => "Xrgb16161616",
                                Format::Xbgr16161616 => "Xbgr16161616",
                                Format::Argb16161616 => "Argb16161616",
                                Format::Abgr16161616 => "Abgr16161616",
                                Format::C1 => "C1",
                                Format::C2 => "C2",
                                Format::C4 => "C4",
                                Format::D1 => "D1",
                                Format::D2 => "D2",
                                Format::D4 => "D4",
                                Format::D8 => "D8",
                                Format::R1 => "R1",
                                Format::R2 => "R2",
                                Format::R4 => "R4",
                                Format::R10 => "R10",
                                Format::R12 => "R12",
                                Format::Avuy8888 => "Avuy8888",
                                Format::Xvuy8888 => "Xvuy8888",
                                Format::P030 => "P030",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Format {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Format {
                    #[inline]
                    fn eq(&self, other: &Format) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Format {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Format {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Format {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Format,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Format {
                    #[inline]
                    fn cmp(&self, other: &Format) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Format {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Format {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Format> for u32 {
                    fn from(value: Format) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Format")]
                pub struct FormatFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for FormatFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "FormatFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for FormatFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for FormatFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Format", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Format {
                    type Error = FormatFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Argb8888,
                                1u32 => Self::Xrgb8888,
                                538982467u32 => Self::C8,
                                943867730u32 => Self::Rgb332,
                                944916290u32 => Self::Bgr233,
                                842093144u32 => Self::Xrgb4444,
                                842089048u32 => Self::Xbgr4444,
                                842094674u32 => Self::Rgbx4444,
                                842094658u32 => Self::Bgrx4444,
                                842093121u32 => Self::Argb4444,
                                842089025u32 => Self::Abgr4444,
                                842088786u32 => Self::Rgba4444,
                                842088770u32 => Self::Bgra4444,
                                892424792u32 => Self::Xrgb1555,
                                892420696u32 => Self::Xbgr1555,
                                892426322u32 => Self::Rgbx5551,
                                892426306u32 => Self::Bgrx5551,
                                892424769u32 => Self::Argb1555,
                                892420673u32 => Self::Abgr1555,
                                892420434u32 => Self::Rgba5551,
                                892420418u32 => Self::Bgra5551,
                                909199186u32 => Self::Rgb565,
                                909199170u32 => Self::Bgr565,
                                875710290u32 => Self::Rgb888,
                                875710274u32 => Self::Bgr888,
                                875709016u32 => Self::Xbgr8888,
                                875714642u32 => Self::Rgbx8888,
                                875714626u32 => Self::Bgrx8888,
                                875708993u32 => Self::Abgr8888,
                                875708754u32 => Self::Rgba8888,
                                875708738u32 => Self::Bgra8888,
                                808669784u32 => Self::Xrgb2101010,
                                808665688u32 => Self::Xbgr2101010,
                                808671314u32 => Self::Rgbx1010102,
                                808671298u32 => Self::Bgrx1010102,
                                808669761u32 => Self::Argb2101010,
                                808665665u32 => Self::Abgr2101010,
                                808665426u32 => Self::Rgba1010102,
                                808665410u32 => Self::Bgra1010102,
                                1448695129u32 => Self::Yuyv,
                                1431918169u32 => Self::Yvyu,
                                1498831189u32 => Self::Uyvy,
                                1498765654u32 => Self::Vyuy,
                                1448433985u32 => Self::Ayuv,
                                842094158u32 => Self::Nv12,
                                825382478u32 => Self::Nv21,
                                909203022u32 => Self::Nv16,
                                825644622u32 => Self::Nv61,
                                961959257u32 => Self::Yuv410,
                                961893977u32 => Self::Yvu410,
                                825316697u32 => Self::Yuv411,
                                825316953u32 => Self::Yvu411,
                                842093913u32 => Self::Yuv420,
                                842094169u32 => Self::Yvu420,
                                909202777u32 => Self::Yuv422,
                                909203033u32 => Self::Yvu422,
                                875713881u32 => Self::Yuv444,
                                875714137u32 => Self::Yvu444,
                                538982482u32 => Self::R8,
                                540422482u32 => Self::R16,
                                943212370u32 => Self::Rg88,
                                943215175u32 => Self::Gr88,
                                842221394u32 => Self::Rg1616,
                                842224199u32 => Self::Gr1616,
                                1211388504u32 => Self::Xrgb16161616F,
                                1211384408u32 => Self::Xbgr16161616F,
                                1211388481u32 => Self::Argb16161616F,
                                1211384385u32 => Self::Abgr16161616F,
                                1448434008u32 => Self::Xyuv8888,
                                875713878u32 => Self::Vuy888,
                                808670550u32 => Self::Vuy101010,
                                808530521u32 => Self::Y210,
                                842084953u32 => Self::Y212,
                                909193817u32 => Self::Y216,
                                808531033u32 => Self::Y410,
                                842085465u32 => Self::Y412,
                                909194329u32 => Self::Y416,
                                808670808u32 => Self::Xvyu2101010,
                                909334104u32 => Self::Xvyu1216161616,
                                942954072u32 => Self::Xvyu16161616,
                                810299481u32 => Self::Y0L0,
                                810299480u32 => Self::X0L0,
                                843853913u32 => Self::Y0L2,
                                843853912u32 => Self::X0L2,
                                942691673u32 => Self::Yuv4208Bit,
                                808539481u32 => Self::Yuv42010Bit,
                                943805016u32 => Self::Xrgb8888A8,
                                943800920u32 => Self::Xbgr8888A8,
                                943806546u32 => Self::Rgbx8888A8,
                                943806530u32 => Self::Bgrx8888A8,
                                943798354u32 => Self::Rgb888A8,
                                943798338u32 => Self::Bgr888A8,
                                943797586u32 => Self::Rgb565A8,
                                943797570u32 => Self::Bgr565A8,
                                875714126u32 => Self::Nv24,
                                842290766u32 => Self::Nv42,
                                808530512u32 => Self::P210,
                                808530000u32 => Self::P010,
                                842084432u32 => Self::P012,
                                909193296u32 => Self::P016,
                                808534593u32 => Self::Axbxgxrx106106106106,
                                892425806u32 => Self::Nv15,
                                808531025u32 => Self::Q410,
                                825242705u32 => Self::Q401,
                                942953048u32 => Self::Xrgb16161616,
                                942948952u32 => Self::Xbgr16161616,
                                942953025u32 => Self::Argb16161616,
                                942948929u32 => Self::Abgr16161616,
                                538980675u32 => Self::C1,
                                538980931u32 => Self::C2,
                                538981443u32 => Self::C4,
                                538980676u32 => Self::D1,
                                538980932u32 => Self::D2,
                                538981444u32 => Self::D4,
                                538982468u32 => Self::D8,
                                538980690u32 => Self::R1,
                                538980946u32 => Self::R2,
                                538981458u32 => Self::R4,
                                540029266u32 => Self::R10,
                                540160338u32 => Self::R12,
                                1498764865u32 => Self::Avuy8888,
                                1498764888u32 => Self::Xvuy8888,
                                808661072u32 => Self::P030,
                                _ => return Err(FormatFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod buffer {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Buffer;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {}
        }
        pub mod data_offer {
            pub mod request {
                pub struct Accept<'s> {
                    serial: u32,
                    mime_type: &'s ::std::ffi::CStr,
                }
                impl crate::object::HasObjectType for Accept<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataOffer;
                }
                impl<'s> crate::interface::Request<'s> for Accept<'s> {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.serial)
                            .str(self.mime_type)
                            .build()
                    }
                }
                pub struct Receive<'s> {
                    mime_type: &'s ::std::ffi::CStr,
                    fd: ::std::os::fd::BorrowedFd<'s>,
                }
                impl crate::object::HasObjectType for Receive<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataOffer;
                }
                impl<'s> crate::interface::Request<'s> for Receive<'s> {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .str(self.mime_type)
                            .fd(self.fd)
                            .build()
                    }
                }
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataOffer;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct Finish;
                impl crate::object::HasObjectType for Finish {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataOffer;
                }
                impl<'s> crate::interface::Request<'s> for Finish {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetActions {
                    dnd_actions: u32,
                    preferred_action: u32,
                }
                impl crate::object::HasObjectType for SetActions {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataOffer;
                }
                impl<'s> crate::interface::Request<'s> for SetActions {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.dnd_actions)
                            .uint(self.preferred_action)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///finish request was called untimely
                    InvalidFinish = 0u32,
                    ///action mask contains invalid values
                    InvalidActionMask = 1u32,
                    ///action argument has an invalid value
                    InvalidAction = 2u32,
                    ///offer doesn't accept this request
                    InvalidOffer = 3u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::InvalidFinish => "InvalidFinish",
                                Error::InvalidActionMask => "InvalidActionMask",
                                Error::InvalidAction => "InvalidAction",
                                Error::InvalidOffer => "InvalidOffer",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidFinish,
                                1u32 => Self::InvalidActionMask,
                                2u32 => Self::InvalidAction,
                                3u32 => Self::InvalidOffer,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod data_source {
            pub mod request {
                pub struct Offer<'s> {
                    mime_type: &'s ::std::ffi::CStr,
                }
                impl crate::object::HasObjectType for Offer<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataSource;
                }
                impl<'s> crate::interface::Request<'s> for Offer<'s> {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .str(self.mime_type)
                            .build()
                    }
                }
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataSource;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetActions {
                    dnd_actions: u32,
                }
                impl crate::object::HasObjectType for SetActions {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataSource;
                }
                impl<'s> crate::interface::Request<'s> for SetActions {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.dnd_actions)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///action mask contains invalid values
                    InvalidActionMask = 0u32,
                    ///source doesn't accept this request
                    InvalidSource = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::InvalidActionMask => "InvalidActionMask",
                                Error::InvalidSource => "InvalidSource",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidActionMask,
                                1u32 => Self::InvalidSource,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod data_device {
            pub mod request {
                pub struct StartDrag {
                    source: ::std::option::Option<crate::object::WlObjectId>,
                    origin: crate::object::WlObjectId,
                    icon: ::std::option::Option<crate::object::WlObjectId>,
                    serial: u32,
                }
                impl crate::object::HasObjectType for StartDrag {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataDevice;
                }
                impl<'s> crate::interface::Request<'s> for StartDrag {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.source.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.origin).unwrap(),
                                ),
                            )
                            .maybe_object(
                                self.icon.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .uint(self.serial)
                            .build()
                    }
                }
                pub struct SetSelection {
                    source: ::std::option::Option<crate::object::WlObjectId>,
                    serial: u32,
                }
                impl crate::object::HasObjectType for SetSelection {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataDevice;
                }
                impl<'s> crate::interface::Request<'s> for SetSelection {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.source.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .uint(self.serial)
                            .build()
                    }
                }
                pub struct Release;
                impl crate::object::HasObjectType for Release {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataDevice;
                }
                impl<'s> crate::interface::Request<'s> for Release {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///given wl_surface has another role
                    Role = 0u32,
                    ///source has already been used
                    UsedSource = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::Role => "Role",
                                Error::UsedSource => "UsedSource",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Role,
                                1u32 => Self::UsedSource,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod data_device_manager {
            pub mod request {
                pub struct CreateDataSource;
                impl crate::interface::ObjectParent for CreateDataSource {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataSource;
                }
                impl crate::object::HasObjectType for CreateDataSource {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataDeviceManager;
                }
                impl<'s> crate::interface::Request<'s> for CreateDataSource {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::DataSource,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct GetDataDevice {
                    seat: crate::object::WlObjectId,
                }
                impl crate::interface::ObjectParent for GetDataDevice {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataDevice;
                }
                impl crate::object::HasObjectType for GetDataDevice {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::DataDeviceManager;
                }
                impl<'s> crate::interface::Request<'s> for GetDataDevice {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::DataDevice,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                /**This is a bitmask of the available/preferred actions in a
	drag-and-drop operation.

	In the compositor, the selected action is a result of matching the
	actions offered by the source and destination sides.  "action" events
	with a "none" action will be sent to both source and destination if
	there is no match. All further checks will effectively happen on
	(source actions ∩ destination actions).

	In addition, compositors may also pick different actions in
	reaction to key modifiers being pressed. One common design that
	is used in major toolkits (and the behavior recommended for
	compositors) is:

	- If no modifiers are pressed, the first match (in bit order)
	  will be used.
	- Pressing Shift selects "move", if enabled in the mask.
	- Pressing Control selects "copy", if enabled in the mask.

	Behavior beyond that is considered implementation-dependent.
	Compositors may for example bind other modifiers (like Alt/Meta)
	or drags initiated with other buttons than BTN_LEFT to specific
	actions (e.g. "ask").*/
                pub struct DndAction(
                    <DndAction as ::bitflags::__private::PublicFlags>::Internal,
                );
                #[automatically_derived]
                impl ::core::fmt::Debug for DndAction {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "DndAction",
                            &&self.0,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::clone::Clone for DndAction {
                    #[inline]
                    fn clone(&self) -> DndAction {
                        let _: ::core::clone::AssertParamIsClone<
                            <DndAction as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for DndAction {}
                #[automatically_derived]
                impl ::core::hash::Hash for DndAction {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        ::core::hash::Hash::hash(&self.0, state)
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for DndAction {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for DndAction {
                    #[inline]
                    fn eq(&self, other: &DndAction) -> bool {
                        self.0 == other.0
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Eq for DndAction {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {
                        let _: ::core::cmp::AssertParamIsEq<
                            <DndAction as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for DndAction {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &DndAction,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for DndAction {
                    #[inline]
                    fn cmp(&self, other: &DndAction) -> ::core::cmp::Ordering {
                        ::core::cmp::Ord::cmp(&self.0, &other.0)
                    }
                }
                impl DndAction {
                    ///no action
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const NONE: Self = Self::from_bits_retain(0);
                    ///copy action
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const COPY: Self = Self::from_bits_retain(1);
                    ///move action
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const MOVE: Self = Self::from_bits_retain(2);
                    ///ask action
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const ASK: Self = Self::from_bits_retain(4);
                }
                impl ::bitflags::Flags for DndAction {
                    const FLAGS: &'static [::bitflags::Flag<DndAction>] = &[
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("NONE", DndAction::NONE)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("COPY", DndAction::COPY)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("MOVE", DndAction::MOVE)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("ASK", DndAction::ASK)
                        },
                    ];
                    type Bits = u32;
                    fn bits(&self) -> u32 {
                        DndAction::bits(self)
                    }
                    fn from_bits_retain(bits: u32) -> DndAction {
                        DndAction::from_bits_retain(bits)
                    }
                }
                #[allow(
                    dead_code,
                    deprecated,
                    unused_doc_comments,
                    unused_attributes,
                    unused_mut,
                    unused_imports,
                    non_upper_case_globals,
                    clippy::assign_op_pattern,
                    clippy::indexing_slicing,
                    clippy::same_name_method,
                    clippy::iter_without_into_iter,
                )]
                const _: () = {
                    #[repr(transparent)]
                    pub struct InternalBitFlags(u32);
                    #[automatically_derived]
                    impl ::core::clone::Clone for InternalBitFlags {
                        #[inline]
                        fn clone(&self) -> InternalBitFlags {
                            let _: ::core::clone::AssertParamIsClone<u32>;
                            *self
                        }
                    }
                    #[automatically_derived]
                    impl ::core::marker::Copy for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::marker::StructuralPartialEq for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::cmp::PartialEq for InternalBitFlags {
                        #[inline]
                        fn eq(&self, other: &InternalBitFlags) -> bool {
                            self.0 == other.0
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Eq for InternalBitFlags {
                        #[inline]
                        #[doc(hidden)]
                        #[coverage(off)]
                        fn assert_receiver_is_total_eq(&self) -> () {
                            let _: ::core::cmp::AssertParamIsEq<u32>;
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::PartialOrd for InternalBitFlags {
                        #[inline]
                        fn partial_cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::option::Option<::core::cmp::Ordering> {
                            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Ord for InternalBitFlags {
                        #[inline]
                        fn cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::cmp::Ordering {
                            ::core::cmp::Ord::cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::hash::Hash for InternalBitFlags {
                        #[inline]
                        fn hash<__H: ::core::hash::Hasher>(
                            &self,
                            state: &mut __H,
                        ) -> () {
                            ::core::hash::Hash::hash(&self.0, state)
                        }
                    }
                    impl ::bitflags::__private::PublicFlags for DndAction {
                        type Primitive = u32;
                        type Internal = InternalBitFlags;
                    }
                    impl ::bitflags::__private::core::default::Default
                    for InternalBitFlags {
                        #[inline]
                        fn default() -> Self {
                            InternalBitFlags::empty()
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Debug for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            if self.is_empty() {
                                f.write_fmt(
                                    format_args!("{0:#x}", <u32 as ::bitflags::Bits>::EMPTY),
                                )
                            } else {
                                ::bitflags::__private::core::fmt::Display::fmt(self, f)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Display for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            ::bitflags::parser::to_writer(&DndAction(*self), f)
                        }
                    }
                    impl ::bitflags::__private::core::str::FromStr for InternalBitFlags {
                        type Err = ::bitflags::parser::ParseError;
                        fn from_str(
                            s: &str,
                        ) -> ::bitflags::__private::core::result::Result<
                            Self,
                            Self::Err,
                        > {
                            ::bitflags::parser::from_str::<DndAction>(s)
                                .map(|flags| flags.0)
                        }
                    }
                    impl ::bitflags::__private::core::convert::AsRef<u32>
                    for InternalBitFlags {
                        fn as_ref(&self) -> &u32 {
                            &self.0
                        }
                    }
                    impl ::bitflags::__private::core::convert::From<u32>
                    for InternalBitFlags {
                        fn from(bits: u32) -> Self {
                            Self::from_bits_retain(bits)
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl InternalBitFlags {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(<u32 as ::bitflags::Bits>::EMPTY) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            {
                                let mut truncated = <u32 as ::bitflags::Bits>::EMPTY;
                                let mut i = 0;
                                {
                                    {
                                        let flag = <DndAction as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <DndAction as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <DndAction as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <DndAction as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                let _ = i;
                                Self::from_bits_retain(truncated)
                            }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0 }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                let truncated = Self::from_bits_truncate(bits).0;
                                if truncated == bits {
                                    ::bitflags::__private::core::option::Option::Some(
                                        Self(bits),
                                    )
                                } else {
                                    ::bitflags::__private::core::option::Option::None
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits & Self::all().bits()) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                {
                                    if name == "NONE" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(DndAction::NONE.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "COPY" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(DndAction::COPY.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "MOVE" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(DndAction::MOVE.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "ASK" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(DndAction::ASK.bits()),
                                        );
                                    }
                                };
                                let _ = name;
                                ::bitflags::__private::core::option::Option::None
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.bits() == <u32 as ::bitflags::Bits>::EMPTY }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { Self::all().bits() | f.bits() == f.bits() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            {
                                f.bits() & other.bits() != <u32 as ::bitflags::Bits>::EMPTY
                            }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.bits() & other.bits() == other.bits() }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).union(other);
                            }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).difference(other);
                            }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits())
                                    .symmetric_difference(other);
                            }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            {
                                if value {
                                    f.insert(other);
                                } else {
                                    f.remove(other);
                                }
                            }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & other.bits()) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() | other.bits()) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & !other.bits()) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() ^ other.bits()) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self::from_bits_truncate(!f.bits()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: InternalBitFlags) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for InternalBitFlags {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for InternalBitFlags {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for InternalBitFlags {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for InternalBitFlags {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<InternalBitFlags>
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        InternalBitFlags,
                    > for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl InternalBitFlags {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<DndAction> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <DndAction as ::bitflags::Flags>::FLAGS,
                                DndAction::from_bits_retain(self.bits()),
                                DndAction::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<DndAction> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <DndAction as ::bitflags::Flags>::FLAGS,
                                DndAction::from_bits_retain(self.bits()),
                                DndAction::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for InternalBitFlags {
                        type Item = DndAction;
                        type IntoIter = ::bitflags::iter::Iter<DndAction>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                    impl InternalBitFlags {
                        /// Returns a mutable reference to the raw value of the flags currently stored.
                        #[inline]
                        pub fn bits_mut(&mut self) -> &mut u32 {
                            &mut self.0
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl DndAction {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(InternalBitFlags::empty()) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            { Self(InternalBitFlags::all()) }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0.bits() }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                match InternalBitFlags::from_bits(bits) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_truncate(bits)) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_retain(bits)) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                match InternalBitFlags::from_name(name) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.0.is_empty() }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { f.0.is_all() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.intersects(other.0) }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.contains(other.0) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.insert(other.0) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.remove(other.0) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.toggle(other.0) }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            { f.0.set(other.0, value) }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.intersection(other.0)) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.union(other.0)) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.difference(other.0)) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.symmetric_difference(other.0)) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self(f.0.complement()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for DndAction {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for DndAction {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex for DndAction {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex for DndAction {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for DndAction {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: DndAction) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign for DndAction {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for DndAction {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign for DndAction {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for DndAction {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign for DndAction {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for DndAction {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign for DndAction {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for DndAction {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<DndAction>
                    for DndAction {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<DndAction>
                    for DndAction {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl DndAction {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<DndAction> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <DndAction as ::bitflags::Flags>::FLAGS,
                                DndAction::from_bits_retain(self.bits()),
                                DndAction::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<DndAction> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <DndAction as ::bitflags::Flags>::FLAGS,
                                DndAction::from_bits_retain(self.bits()),
                                DndAction::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator for DndAction {
                        type Item = DndAction;
                        type IntoIter = ::bitflags::iter::Iter<DndAction>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                };
            }
        }
        pub mod shell {
            pub mod request {
                pub struct GetShellSurface {
                    surface: crate::object::WlObjectId,
                }
                impl crate::interface::ObjectParent for GetShellSurface {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl crate::object::HasObjectType for GetShellSurface {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Shell;
                }
                impl<'s> crate::interface::Request<'s> for GetShellSurface {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::ShellSurface,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.surface).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///given wl_surface has another role
                    Role = 0u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(f, "Role")
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        true
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        ::core::cmp::Ordering::Equal
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Role,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod shell_surface {
            pub mod request {
                pub struct Pong {
                    serial: u32,
                }
                impl crate::object::HasObjectType for Pong {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for Pong {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.serial)
                            .build()
                    }
                }
                pub struct Move {
                    seat: crate::object::WlObjectId,
                    serial: u32,
                }
                impl crate::object::HasObjectType for Move {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for Move {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .uint(self.serial)
                            .build()
                    }
                }
                pub struct Resize {
                    seat: crate::object::WlObjectId,
                    serial: u32,
                    edges: u32,
                }
                impl crate::object::HasObjectType for Resize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for Resize {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .uint(self.serial)
                            .uint(self.edges)
                            .build()
                    }
                }
                pub struct SetToplevel;
                impl crate::object::HasObjectType for SetToplevel {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetToplevel {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetTransient {
                    parent: crate::object::WlObjectId,
                    x: i32,
                    y: i32,
                    flags: u32,
                }
                impl crate::object::HasObjectType for SetTransient {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetTransient {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.parent).unwrap(),
                                ),
                            )
                            .int(self.x)
                            .int(self.y)
                            .uint(self.flags)
                            .build()
                    }
                }
                pub struct SetFullscreen {
                    method: u32,
                    framerate: u32,
                    output: ::std::option::Option<crate::object::WlObjectId>,
                }
                impl crate::object::HasObjectType for SetFullscreen {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetFullscreen {
                    const CODE: crate::sys::wire::OpCode = 5;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.method)
                            .uint(self.framerate)
                            .maybe_object(
                                self.output.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .build()
                    }
                }
                pub struct SetPopup {
                    seat: crate::object::WlObjectId,
                    serial: u32,
                    parent: crate::object::WlObjectId,
                    x: i32,
                    y: i32,
                    flags: u32,
                }
                impl crate::object::HasObjectType for SetPopup {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetPopup {
                    const CODE: crate::sys::wire::OpCode = 6;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .uint(self.serial)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.parent).unwrap(),
                                ),
                            )
                            .int(self.x)
                            .int(self.y)
                            .uint(self.flags)
                            .build()
                    }
                }
                pub struct SetMaximized {
                    output: ::std::option::Option<crate::object::WlObjectId>,
                }
                impl crate::object::HasObjectType for SetMaximized {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetMaximized {
                    const CODE: crate::sys::wire::OpCode = 7;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.output.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .build()
                    }
                }
                pub struct SetTitle<'s> {
                    title: &'s ::std::ffi::CStr,
                }
                impl crate::object::HasObjectType for SetTitle<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetTitle<'s> {
                    const CODE: crate::sys::wire::OpCode = 8;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .str(self.title)
                            .build()
                    }
                }
                pub struct SetClass<'s> {
                    class_: &'s ::std::ffi::CStr,
                }
                impl crate::object::HasObjectType for SetClass<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::ShellSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetClass<'s> {
                    const CODE: crate::sys::wire::OpCode = 9;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .str(self.class_)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                /**These values are used to indicate which edge of a surface
	is being dragged in a resize operation. The server may
	use this information to adapt its behavior, e.g. choose
	an appropriate cursor image.*/
                pub struct Resize(
                    <Resize as ::bitflags::__private::PublicFlags>::Internal,
                );
                #[automatically_derived]
                impl ::core::fmt::Debug for Resize {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Resize",
                            &&self.0,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Resize {
                    #[inline]
                    fn clone(&self) -> Resize {
                        let _: ::core::clone::AssertParamIsClone<
                            <Resize as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Resize {}
                #[automatically_derived]
                impl ::core::hash::Hash for Resize {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        ::core::hash::Hash::hash(&self.0, state)
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Resize {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Resize {
                    #[inline]
                    fn eq(&self, other: &Resize) -> bool {
                        self.0 == other.0
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Eq for Resize {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {
                        let _: ::core::cmp::AssertParamIsEq<
                            <Resize as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Resize {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Resize,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Resize {
                    #[inline]
                    fn cmp(&self, other: &Resize) -> ::core::cmp::Ordering {
                        ::core::cmp::Ord::cmp(&self.0, &other.0)
                    }
                }
                impl Resize {
                    ///no edge
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const NONE: Self = Self::from_bits_retain(0);
                    ///top edge
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const TOP: Self = Self::from_bits_retain(1);
                    ///bottom edge
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const BOTTOM: Self = Self::from_bits_retain(2);
                    ///left edge
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const LEFT: Self = Self::from_bits_retain(4);
                    ///top and left edges
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const TOP_LEFT: Self = Self::from_bits_retain(5);
                    ///bottom and left edges
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const BOTTOM_LEFT: Self = Self::from_bits_retain(6);
                    ///right edge
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const RIGHT: Self = Self::from_bits_retain(8);
                    ///top and right edges
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const TOP_RIGHT: Self = Self::from_bits_retain(9);
                    ///bottom and right edges
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const BOTTOM_RIGHT: Self = Self::from_bits_retain(10);
                }
                impl ::bitflags::Flags for Resize {
                    const FLAGS: &'static [::bitflags::Flag<Resize>] = &[
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("NONE", Resize::NONE)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("TOP", Resize::TOP)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("BOTTOM", Resize::BOTTOM)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("LEFT", Resize::LEFT)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("TOP_LEFT", Resize::TOP_LEFT)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("BOTTOM_LEFT", Resize::BOTTOM_LEFT)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("RIGHT", Resize::RIGHT)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("TOP_RIGHT", Resize::TOP_RIGHT)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("BOTTOM_RIGHT", Resize::BOTTOM_RIGHT)
                        },
                    ];
                    type Bits = u32;
                    fn bits(&self) -> u32 {
                        Resize::bits(self)
                    }
                    fn from_bits_retain(bits: u32) -> Resize {
                        Resize::from_bits_retain(bits)
                    }
                }
                #[allow(
                    dead_code,
                    deprecated,
                    unused_doc_comments,
                    unused_attributes,
                    unused_mut,
                    unused_imports,
                    non_upper_case_globals,
                    clippy::assign_op_pattern,
                    clippy::indexing_slicing,
                    clippy::same_name_method,
                    clippy::iter_without_into_iter,
                )]
                const _: () = {
                    #[repr(transparent)]
                    pub struct InternalBitFlags(u32);
                    #[automatically_derived]
                    impl ::core::clone::Clone for InternalBitFlags {
                        #[inline]
                        fn clone(&self) -> InternalBitFlags {
                            let _: ::core::clone::AssertParamIsClone<u32>;
                            *self
                        }
                    }
                    #[automatically_derived]
                    impl ::core::marker::Copy for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::marker::StructuralPartialEq for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::cmp::PartialEq for InternalBitFlags {
                        #[inline]
                        fn eq(&self, other: &InternalBitFlags) -> bool {
                            self.0 == other.0
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Eq for InternalBitFlags {
                        #[inline]
                        #[doc(hidden)]
                        #[coverage(off)]
                        fn assert_receiver_is_total_eq(&self) -> () {
                            let _: ::core::cmp::AssertParamIsEq<u32>;
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::PartialOrd for InternalBitFlags {
                        #[inline]
                        fn partial_cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::option::Option<::core::cmp::Ordering> {
                            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Ord for InternalBitFlags {
                        #[inline]
                        fn cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::cmp::Ordering {
                            ::core::cmp::Ord::cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::hash::Hash for InternalBitFlags {
                        #[inline]
                        fn hash<__H: ::core::hash::Hasher>(
                            &self,
                            state: &mut __H,
                        ) -> () {
                            ::core::hash::Hash::hash(&self.0, state)
                        }
                    }
                    impl ::bitflags::__private::PublicFlags for Resize {
                        type Primitive = u32;
                        type Internal = InternalBitFlags;
                    }
                    impl ::bitflags::__private::core::default::Default
                    for InternalBitFlags {
                        #[inline]
                        fn default() -> Self {
                            InternalBitFlags::empty()
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Debug for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            if self.is_empty() {
                                f.write_fmt(
                                    format_args!("{0:#x}", <u32 as ::bitflags::Bits>::EMPTY),
                                )
                            } else {
                                ::bitflags::__private::core::fmt::Display::fmt(self, f)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Display for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            ::bitflags::parser::to_writer(&Resize(*self), f)
                        }
                    }
                    impl ::bitflags::__private::core::str::FromStr for InternalBitFlags {
                        type Err = ::bitflags::parser::ParseError;
                        fn from_str(
                            s: &str,
                        ) -> ::bitflags::__private::core::result::Result<
                            Self,
                            Self::Err,
                        > {
                            ::bitflags::parser::from_str::<Resize>(s)
                                .map(|flags| flags.0)
                        }
                    }
                    impl ::bitflags::__private::core::convert::AsRef<u32>
                    for InternalBitFlags {
                        fn as_ref(&self) -> &u32 {
                            &self.0
                        }
                    }
                    impl ::bitflags::__private::core::convert::From<u32>
                    for InternalBitFlags {
                        fn from(bits: u32) -> Self {
                            Self::from_bits_retain(bits)
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl InternalBitFlags {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(<u32 as ::bitflags::Bits>::EMPTY) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            {
                                let mut truncated = <u32 as ::bitflags::Bits>::EMPTY;
                                let mut i = 0;
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Resize as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                let _ = i;
                                Self::from_bits_retain(truncated)
                            }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0 }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                let truncated = Self::from_bits_truncate(bits).0;
                                if truncated == bits {
                                    ::bitflags::__private::core::option::Option::Some(
                                        Self(bits),
                                    )
                                } else {
                                    ::bitflags::__private::core::option::Option::None
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits & Self::all().bits()) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                {
                                    if name == "NONE" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::NONE.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "TOP" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::TOP.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "BOTTOM" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::BOTTOM.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "LEFT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::LEFT.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "TOP_LEFT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::TOP_LEFT.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "BOTTOM_LEFT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::BOTTOM_LEFT.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "RIGHT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::RIGHT.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "TOP_RIGHT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::TOP_RIGHT.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "BOTTOM_RIGHT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Resize::BOTTOM_RIGHT.bits()),
                                        );
                                    }
                                };
                                let _ = name;
                                ::bitflags::__private::core::option::Option::None
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.bits() == <u32 as ::bitflags::Bits>::EMPTY }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { Self::all().bits() | f.bits() == f.bits() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            {
                                f.bits() & other.bits() != <u32 as ::bitflags::Bits>::EMPTY
                            }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.bits() & other.bits() == other.bits() }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).union(other);
                            }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).difference(other);
                            }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits())
                                    .symmetric_difference(other);
                            }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            {
                                if value {
                                    f.insert(other);
                                } else {
                                    f.remove(other);
                                }
                            }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & other.bits()) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() | other.bits()) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & !other.bits()) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() ^ other.bits()) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self::from_bits_truncate(!f.bits()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: InternalBitFlags) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for InternalBitFlags {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for InternalBitFlags {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for InternalBitFlags {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for InternalBitFlags {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<InternalBitFlags>
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        InternalBitFlags,
                    > for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl InternalBitFlags {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Resize> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Resize as ::bitflags::Flags>::FLAGS,
                                Resize::from_bits_retain(self.bits()),
                                Resize::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Resize> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Resize as ::bitflags::Flags>::FLAGS,
                                Resize::from_bits_retain(self.bits()),
                                Resize::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for InternalBitFlags {
                        type Item = Resize;
                        type IntoIter = ::bitflags::iter::Iter<Resize>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                    impl InternalBitFlags {
                        /// Returns a mutable reference to the raw value of the flags currently stored.
                        #[inline]
                        pub fn bits_mut(&mut self) -> &mut u32 {
                            &mut self.0
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl Resize {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(InternalBitFlags::empty()) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            { Self(InternalBitFlags::all()) }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0.bits() }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                match InternalBitFlags::from_bits(bits) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_truncate(bits)) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_retain(bits)) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                match InternalBitFlags::from_name(name) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.0.is_empty() }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { f.0.is_all() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.intersects(other.0) }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.contains(other.0) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.insert(other.0) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.remove(other.0) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.toggle(other.0) }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            { f.0.set(other.0, value) }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.intersection(other.0)) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.union(other.0)) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.difference(other.0)) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.symmetric_difference(other.0)) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self(f.0.complement()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for Resize {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for Resize {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex for Resize {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex for Resize {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for Resize {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: Resize) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign for Resize {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for Resize {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign for Resize {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for Resize {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign for Resize {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for Resize {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign for Resize {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for Resize {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<Resize> for Resize {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<Resize>
                    for Resize {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl Resize {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Resize> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Resize as ::bitflags::Flags>::FLAGS,
                                Resize::from_bits_retain(self.bits()),
                                Resize::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Resize> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Resize as ::bitflags::Flags>::FLAGS,
                                Resize::from_bits_retain(self.bits()),
                                Resize::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator for Resize {
                        type Item = Resize;
                        type IntoIter = ::bitflags::iter::Iter<Resize>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                };
                /**These flags specify details of the expected behaviour
	of transient surfaces. Used in the set_transient request.*/
                pub struct Transient(
                    <Transient as ::bitflags::__private::PublicFlags>::Internal,
                );
                #[automatically_derived]
                impl ::core::fmt::Debug for Transient {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Transient",
                            &&self.0,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Transient {
                    #[inline]
                    fn clone(&self) -> Transient {
                        let _: ::core::clone::AssertParamIsClone<
                            <Transient as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Transient {}
                #[automatically_derived]
                impl ::core::hash::Hash for Transient {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        ::core::hash::Hash::hash(&self.0, state)
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Transient {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Transient {
                    #[inline]
                    fn eq(&self, other: &Transient) -> bool {
                        self.0 == other.0
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Eq for Transient {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {
                        let _: ::core::cmp::AssertParamIsEq<
                            <Transient as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Transient {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Transient,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Transient {
                    #[inline]
                    fn cmp(&self, other: &Transient) -> ::core::cmp::Ordering {
                        ::core::cmp::Ord::cmp(&self.0, &other.0)
                    }
                }
                impl Transient {
                    ///do not set keyboard focus
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const INACTIVE: Self = Self::from_bits_retain(1);
                }
                impl ::bitflags::Flags for Transient {
                    const FLAGS: &'static [::bitflags::Flag<Transient>] = &[
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("INACTIVE", Transient::INACTIVE)
                        },
                    ];
                    type Bits = u32;
                    fn bits(&self) -> u32 {
                        Transient::bits(self)
                    }
                    fn from_bits_retain(bits: u32) -> Transient {
                        Transient::from_bits_retain(bits)
                    }
                }
                #[allow(
                    dead_code,
                    deprecated,
                    unused_doc_comments,
                    unused_attributes,
                    unused_mut,
                    unused_imports,
                    non_upper_case_globals,
                    clippy::assign_op_pattern,
                    clippy::indexing_slicing,
                    clippy::same_name_method,
                    clippy::iter_without_into_iter,
                )]
                const _: () = {
                    #[repr(transparent)]
                    pub struct InternalBitFlags(u32);
                    #[automatically_derived]
                    impl ::core::clone::Clone for InternalBitFlags {
                        #[inline]
                        fn clone(&self) -> InternalBitFlags {
                            let _: ::core::clone::AssertParamIsClone<u32>;
                            *self
                        }
                    }
                    #[automatically_derived]
                    impl ::core::marker::Copy for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::marker::StructuralPartialEq for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::cmp::PartialEq for InternalBitFlags {
                        #[inline]
                        fn eq(&self, other: &InternalBitFlags) -> bool {
                            self.0 == other.0
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Eq for InternalBitFlags {
                        #[inline]
                        #[doc(hidden)]
                        #[coverage(off)]
                        fn assert_receiver_is_total_eq(&self) -> () {
                            let _: ::core::cmp::AssertParamIsEq<u32>;
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::PartialOrd for InternalBitFlags {
                        #[inline]
                        fn partial_cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::option::Option<::core::cmp::Ordering> {
                            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Ord for InternalBitFlags {
                        #[inline]
                        fn cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::cmp::Ordering {
                            ::core::cmp::Ord::cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::hash::Hash for InternalBitFlags {
                        #[inline]
                        fn hash<__H: ::core::hash::Hasher>(
                            &self,
                            state: &mut __H,
                        ) -> () {
                            ::core::hash::Hash::hash(&self.0, state)
                        }
                    }
                    impl ::bitflags::__private::PublicFlags for Transient {
                        type Primitive = u32;
                        type Internal = InternalBitFlags;
                    }
                    impl ::bitflags::__private::core::default::Default
                    for InternalBitFlags {
                        #[inline]
                        fn default() -> Self {
                            InternalBitFlags::empty()
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Debug for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            if self.is_empty() {
                                f.write_fmt(
                                    format_args!("{0:#x}", <u32 as ::bitflags::Bits>::EMPTY),
                                )
                            } else {
                                ::bitflags::__private::core::fmt::Display::fmt(self, f)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Display for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            ::bitflags::parser::to_writer(&Transient(*self), f)
                        }
                    }
                    impl ::bitflags::__private::core::str::FromStr for InternalBitFlags {
                        type Err = ::bitflags::parser::ParseError;
                        fn from_str(
                            s: &str,
                        ) -> ::bitflags::__private::core::result::Result<
                            Self,
                            Self::Err,
                        > {
                            ::bitflags::parser::from_str::<Transient>(s)
                                .map(|flags| flags.0)
                        }
                    }
                    impl ::bitflags::__private::core::convert::AsRef<u32>
                    for InternalBitFlags {
                        fn as_ref(&self) -> &u32 {
                            &self.0
                        }
                    }
                    impl ::bitflags::__private::core::convert::From<u32>
                    for InternalBitFlags {
                        fn from(bits: u32) -> Self {
                            Self::from_bits_retain(bits)
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl InternalBitFlags {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(<u32 as ::bitflags::Bits>::EMPTY) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            {
                                let mut truncated = <u32 as ::bitflags::Bits>::EMPTY;
                                let mut i = 0;
                                {
                                    {
                                        let flag = <Transient as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                let _ = i;
                                Self::from_bits_retain(truncated)
                            }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0 }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                let truncated = Self::from_bits_truncate(bits).0;
                                if truncated == bits {
                                    ::bitflags::__private::core::option::Option::Some(
                                        Self(bits),
                                    )
                                } else {
                                    ::bitflags::__private::core::option::Option::None
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits & Self::all().bits()) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                {
                                    if name == "INACTIVE" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Transient::INACTIVE.bits()),
                                        );
                                    }
                                };
                                let _ = name;
                                ::bitflags::__private::core::option::Option::None
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.bits() == <u32 as ::bitflags::Bits>::EMPTY }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { Self::all().bits() | f.bits() == f.bits() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            {
                                f.bits() & other.bits() != <u32 as ::bitflags::Bits>::EMPTY
                            }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.bits() & other.bits() == other.bits() }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).union(other);
                            }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).difference(other);
                            }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits())
                                    .symmetric_difference(other);
                            }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            {
                                if value {
                                    f.insert(other);
                                } else {
                                    f.remove(other);
                                }
                            }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & other.bits()) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() | other.bits()) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & !other.bits()) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() ^ other.bits()) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self::from_bits_truncate(!f.bits()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: InternalBitFlags) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for InternalBitFlags {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for InternalBitFlags {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for InternalBitFlags {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for InternalBitFlags {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<InternalBitFlags>
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        InternalBitFlags,
                    > for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl InternalBitFlags {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Transient> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Transient as ::bitflags::Flags>::FLAGS,
                                Transient::from_bits_retain(self.bits()),
                                Transient::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Transient> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Transient as ::bitflags::Flags>::FLAGS,
                                Transient::from_bits_retain(self.bits()),
                                Transient::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for InternalBitFlags {
                        type Item = Transient;
                        type IntoIter = ::bitflags::iter::Iter<Transient>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                    impl InternalBitFlags {
                        /// Returns a mutable reference to the raw value of the flags currently stored.
                        #[inline]
                        pub fn bits_mut(&mut self) -> &mut u32 {
                            &mut self.0
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl Transient {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(InternalBitFlags::empty()) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            { Self(InternalBitFlags::all()) }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0.bits() }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                match InternalBitFlags::from_bits(bits) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_truncate(bits)) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_retain(bits)) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                match InternalBitFlags::from_name(name) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.0.is_empty() }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { f.0.is_all() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.intersects(other.0) }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.contains(other.0) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.insert(other.0) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.remove(other.0) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.toggle(other.0) }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            { f.0.set(other.0, value) }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.intersection(other.0)) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.union(other.0)) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.difference(other.0)) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.symmetric_difference(other.0)) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self(f.0.complement()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for Transient {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for Transient {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex for Transient {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex for Transient {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for Transient {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: Transient) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign for Transient {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for Transient {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign for Transient {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for Transient {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign for Transient {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for Transient {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign for Transient {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for Transient {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<Transient>
                    for Transient {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<Transient>
                    for Transient {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl Transient {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Transient> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Transient as ::bitflags::Flags>::FLAGS,
                                Transient::from_bits_retain(self.bits()),
                                Transient::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Transient> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Transient as ::bitflags::Flags>::FLAGS,
                                Transient::from_bits_retain(self.bits()),
                                Transient::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator for Transient {
                        type Item = Transient;
                        type IntoIter = ::bitflags::iter::Iter<Transient>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                };
                /**Hints to indicate to the compositor how to deal with a conflict
	between the dimensions of the surface and the dimensions of the
	output. The compositor is free to ignore this parameter.*/
                #[repr(u32)]
                pub enum FullscreenMethod {
                    ///no preference, apply default policy
                    Default = 0u32,
                    ///scale, preserve the surface's aspect ratio and center on output
                    Scale = 1u32,
                    ///switch output mode to the smallest mode that can fit the surface, add black borders to compensate size mismatch
                    Driver = 2u32,
                    ///no upscaling, center on output and add black borders to compensate size mismatch
                    Fill = 3u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for FullscreenMethod {
                    #[inline]
                    fn clone(&self) -> FullscreenMethod {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for FullscreenMethod {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                FullscreenMethod::Default => "Default",
                                FullscreenMethod::Scale => "Scale",
                                FullscreenMethod::Driver => "Driver",
                                FullscreenMethod::Fill => "Fill",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for FullscreenMethod {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for FullscreenMethod {
                    #[inline]
                    fn eq(&self, other: &FullscreenMethod) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for FullscreenMethod {}
                #[automatically_derived]
                impl ::core::cmp::Eq for FullscreenMethod {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for FullscreenMethod {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &FullscreenMethod,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for FullscreenMethod {
                    #[inline]
                    fn cmp(&self, other: &FullscreenMethod) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for FullscreenMethod {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl FullscreenMethod {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<FullscreenMethod> for u32 {
                    fn from(value: FullscreenMethod) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to FullscreenMethod")]
                pub struct FullscreenMethodFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for FullscreenMethodFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "FullscreenMethodFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for FullscreenMethodFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for FullscreenMethodFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to FullscreenMethod",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for FullscreenMethod {
                    type Error = FullscreenMethodFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Default,
                                1u32 => Self::Scale,
                                2u32 => Self::Driver,
                                3u32 => Self::Fill,
                                _ => return Err(FullscreenMethodFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod surface {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct Attach {
                    buffer: ::std::option::Option<crate::object::WlObjectId>,
                    x: i32,
                    y: i32,
                }
                impl crate::object::HasObjectType for Attach {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for Attach {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.buffer.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .int(self.x)
                            .int(self.y)
                            .build()
                    }
                }
                pub struct Damage {
                    x: i32,
                    y: i32,
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for Damage {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for Damage {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct Frame;
                impl crate::interface::ObjectParent for Frame {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Callback;
                }
                impl crate::object::HasObjectType for Frame {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for Frame {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Callback,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct SetOpaqueRegion {
                    region: ::std::option::Option<crate::object::WlObjectId>,
                }
                impl crate::object::HasObjectType for SetOpaqueRegion {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for SetOpaqueRegion {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.region.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .build()
                    }
                }
                pub struct SetInputRegion {
                    region: ::std::option::Option<crate::object::WlObjectId>,
                }
                impl crate::object::HasObjectType for SetInputRegion {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for SetInputRegion {
                    const CODE: crate::sys::wire::OpCode = 5;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.region.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .build()
                    }
                }
                pub struct Commit;
                impl crate::object::HasObjectType for Commit {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for Commit {
                    const CODE: crate::sys::wire::OpCode = 6;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetBufferTransform {
                    transform: i32,
                }
                impl crate::object::HasObjectType for SetBufferTransform {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for SetBufferTransform {
                    const CODE: crate::sys::wire::OpCode = 7;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.transform)
                            .build()
                    }
                }
                pub struct SetBufferScale {
                    scale: i32,
                }
                impl crate::object::HasObjectType for SetBufferScale {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for SetBufferScale {
                    const CODE: crate::sys::wire::OpCode = 8;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.scale)
                            .build()
                    }
                }
                pub struct DamageBuffer {
                    x: i32,
                    y: i32,
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for DamageBuffer {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for DamageBuffer {
                    const CODE: crate::sys::wire::OpCode = 9;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct Offset {
                    x: i32,
                    y: i32,
                }
                impl crate::object::HasObjectType for Offset {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Surface;
                }
                impl<'s> crate::interface::Request<'s> for Offset {
                    const CODE: crate::sys::wire::OpCode = 10;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                ///These errors can be emitted in response to wl_surface requests.
                #[repr(u32)]
                pub enum Error {
                    ///buffer scale value is invalid
                    InvalidScale = 0u32,
                    ///buffer transform value is invalid
                    InvalidTransform = 1u32,
                    ///buffer size is invalid
                    InvalidSize = 2u32,
                    ///buffer offset is invalid
                    InvalidOffset = 3u32,
                    ///surface was destroyed before its role object
                    DefunctRoleObject = 4u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::InvalidScale => "InvalidScale",
                                Error::InvalidTransform => "InvalidTransform",
                                Error::InvalidSize => "InvalidSize",
                                Error::InvalidOffset => "InvalidOffset",
                                Error::DefunctRoleObject => "DefunctRoleObject",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidScale,
                                1u32 => Self::InvalidTransform,
                                2u32 => Self::InvalidSize,
                                3u32 => Self::InvalidOffset,
                                4u32 => Self::DefunctRoleObject,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod seat {
            pub mod request {
                pub struct GetPointer;
                impl crate::interface::ObjectParent for GetPointer {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Pointer;
                }
                impl crate::object::HasObjectType for GetPointer {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Seat;
                }
                impl<'s> crate::interface::Request<'s> for GetPointer {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Pointer,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct GetKeyboard;
                impl crate::interface::ObjectParent for GetKeyboard {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Keyboard;
                }
                impl crate::object::HasObjectType for GetKeyboard {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Seat;
                }
                impl<'s> crate::interface::Request<'s> for GetKeyboard {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Keyboard,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct GetTouch;
                impl crate::interface::ObjectParent for GetTouch {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Touch;
                }
                impl crate::object::HasObjectType for GetTouch {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Seat;
                }
                impl<'s> crate::interface::Request<'s> for GetTouch {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Touch,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct Release;
                impl crate::object::HasObjectType for Release {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Seat;
                }
                impl<'s> crate::interface::Request<'s> for Release {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                /**This is a bitmask of capabilities this seat has; if a member is
	set, then it is present on the seat.*/
                pub struct Capability(
                    <Capability as ::bitflags::__private::PublicFlags>::Internal,
                );
                #[automatically_derived]
                impl ::core::fmt::Debug for Capability {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Capability",
                            &&self.0,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Capability {
                    #[inline]
                    fn clone(&self) -> Capability {
                        let _: ::core::clone::AssertParamIsClone<
                            <Capability as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Capability {}
                #[automatically_derived]
                impl ::core::hash::Hash for Capability {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        ::core::hash::Hash::hash(&self.0, state)
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Capability {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Capability {
                    #[inline]
                    fn eq(&self, other: &Capability) -> bool {
                        self.0 == other.0
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Eq for Capability {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {
                        let _: ::core::cmp::AssertParamIsEq<
                            <Capability as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Capability {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Capability,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Capability {
                    #[inline]
                    fn cmp(&self, other: &Capability) -> ::core::cmp::Ordering {
                        ::core::cmp::Ord::cmp(&self.0, &other.0)
                    }
                }
                impl Capability {
                    ///the seat has pointer devices
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const POINTER: Self = Self::from_bits_retain(1);
                    ///the seat has one or more keyboards
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const KEYBOARD: Self = Self::from_bits_retain(2);
                    ///the seat has touch devices
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const TOUCH: Self = Self::from_bits_retain(4);
                }
                impl ::bitflags::Flags for Capability {
                    const FLAGS: &'static [::bitflags::Flag<Capability>] = &[
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("POINTER", Capability::POINTER)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("KEYBOARD", Capability::KEYBOARD)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("TOUCH", Capability::TOUCH)
                        },
                    ];
                    type Bits = u32;
                    fn bits(&self) -> u32 {
                        Capability::bits(self)
                    }
                    fn from_bits_retain(bits: u32) -> Capability {
                        Capability::from_bits_retain(bits)
                    }
                }
                #[allow(
                    dead_code,
                    deprecated,
                    unused_doc_comments,
                    unused_attributes,
                    unused_mut,
                    unused_imports,
                    non_upper_case_globals,
                    clippy::assign_op_pattern,
                    clippy::indexing_slicing,
                    clippy::same_name_method,
                    clippy::iter_without_into_iter,
                )]
                const _: () = {
                    #[repr(transparent)]
                    pub struct InternalBitFlags(u32);
                    #[automatically_derived]
                    impl ::core::clone::Clone for InternalBitFlags {
                        #[inline]
                        fn clone(&self) -> InternalBitFlags {
                            let _: ::core::clone::AssertParamIsClone<u32>;
                            *self
                        }
                    }
                    #[automatically_derived]
                    impl ::core::marker::Copy for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::marker::StructuralPartialEq for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::cmp::PartialEq for InternalBitFlags {
                        #[inline]
                        fn eq(&self, other: &InternalBitFlags) -> bool {
                            self.0 == other.0
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Eq for InternalBitFlags {
                        #[inline]
                        #[doc(hidden)]
                        #[coverage(off)]
                        fn assert_receiver_is_total_eq(&self) -> () {
                            let _: ::core::cmp::AssertParamIsEq<u32>;
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::PartialOrd for InternalBitFlags {
                        #[inline]
                        fn partial_cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::option::Option<::core::cmp::Ordering> {
                            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Ord for InternalBitFlags {
                        #[inline]
                        fn cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::cmp::Ordering {
                            ::core::cmp::Ord::cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::hash::Hash for InternalBitFlags {
                        #[inline]
                        fn hash<__H: ::core::hash::Hasher>(
                            &self,
                            state: &mut __H,
                        ) -> () {
                            ::core::hash::Hash::hash(&self.0, state)
                        }
                    }
                    impl ::bitflags::__private::PublicFlags for Capability {
                        type Primitive = u32;
                        type Internal = InternalBitFlags;
                    }
                    impl ::bitflags::__private::core::default::Default
                    for InternalBitFlags {
                        #[inline]
                        fn default() -> Self {
                            InternalBitFlags::empty()
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Debug for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            if self.is_empty() {
                                f.write_fmt(
                                    format_args!("{0:#x}", <u32 as ::bitflags::Bits>::EMPTY),
                                )
                            } else {
                                ::bitflags::__private::core::fmt::Display::fmt(self, f)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Display for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            ::bitflags::parser::to_writer(&Capability(*self), f)
                        }
                    }
                    impl ::bitflags::__private::core::str::FromStr for InternalBitFlags {
                        type Err = ::bitflags::parser::ParseError;
                        fn from_str(
                            s: &str,
                        ) -> ::bitflags::__private::core::result::Result<
                            Self,
                            Self::Err,
                        > {
                            ::bitflags::parser::from_str::<Capability>(s)
                                .map(|flags| flags.0)
                        }
                    }
                    impl ::bitflags::__private::core::convert::AsRef<u32>
                    for InternalBitFlags {
                        fn as_ref(&self) -> &u32 {
                            &self.0
                        }
                    }
                    impl ::bitflags::__private::core::convert::From<u32>
                    for InternalBitFlags {
                        fn from(bits: u32) -> Self {
                            Self::from_bits_retain(bits)
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl InternalBitFlags {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(<u32 as ::bitflags::Bits>::EMPTY) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            {
                                let mut truncated = <u32 as ::bitflags::Bits>::EMPTY;
                                let mut i = 0;
                                {
                                    {
                                        let flag = <Capability as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Capability as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Capability as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                let _ = i;
                                Self::from_bits_retain(truncated)
                            }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0 }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                let truncated = Self::from_bits_truncate(bits).0;
                                if truncated == bits {
                                    ::bitflags::__private::core::option::Option::Some(
                                        Self(bits),
                                    )
                                } else {
                                    ::bitflags::__private::core::option::Option::None
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits & Self::all().bits()) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                {
                                    if name == "POINTER" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Capability::POINTER.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "KEYBOARD" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Capability::KEYBOARD.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "TOUCH" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Capability::TOUCH.bits()),
                                        );
                                    }
                                };
                                let _ = name;
                                ::bitflags::__private::core::option::Option::None
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.bits() == <u32 as ::bitflags::Bits>::EMPTY }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { Self::all().bits() | f.bits() == f.bits() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            {
                                f.bits() & other.bits() != <u32 as ::bitflags::Bits>::EMPTY
                            }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.bits() & other.bits() == other.bits() }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).union(other);
                            }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).difference(other);
                            }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits())
                                    .symmetric_difference(other);
                            }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            {
                                if value {
                                    f.insert(other);
                                } else {
                                    f.remove(other);
                                }
                            }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & other.bits()) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() | other.bits()) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & !other.bits()) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() ^ other.bits()) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self::from_bits_truncate(!f.bits()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: InternalBitFlags) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for InternalBitFlags {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for InternalBitFlags {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for InternalBitFlags {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for InternalBitFlags {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<InternalBitFlags>
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        InternalBitFlags,
                    > for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl InternalBitFlags {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Capability> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Capability as ::bitflags::Flags>::FLAGS,
                                Capability::from_bits_retain(self.bits()),
                                Capability::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Capability> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Capability as ::bitflags::Flags>::FLAGS,
                                Capability::from_bits_retain(self.bits()),
                                Capability::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for InternalBitFlags {
                        type Item = Capability;
                        type IntoIter = ::bitflags::iter::Iter<Capability>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                    impl InternalBitFlags {
                        /// Returns a mutable reference to the raw value of the flags currently stored.
                        #[inline]
                        pub fn bits_mut(&mut self) -> &mut u32 {
                            &mut self.0
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl Capability {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(InternalBitFlags::empty()) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            { Self(InternalBitFlags::all()) }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0.bits() }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                match InternalBitFlags::from_bits(bits) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_truncate(bits)) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_retain(bits)) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                match InternalBitFlags::from_name(name) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.0.is_empty() }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { f.0.is_all() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.intersects(other.0) }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.contains(other.0) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.insert(other.0) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.remove(other.0) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.toggle(other.0) }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            { f.0.set(other.0, value) }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.intersection(other.0)) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.union(other.0)) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.difference(other.0)) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.symmetric_difference(other.0)) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self(f.0.complement()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for Capability {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for Capability {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex for Capability {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex for Capability {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for Capability {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: Capability) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign for Capability {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for Capability {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign for Capability {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for Capability {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign for Capability {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for Capability {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign for Capability {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for Capability {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<Capability>
                    for Capability {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<Capability>
                    for Capability {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl Capability {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Capability> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Capability as ::bitflags::Flags>::FLAGS,
                                Capability::from_bits_retain(self.bits()),
                                Capability::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Capability> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Capability as ::bitflags::Flags>::FLAGS,
                                Capability::from_bits_retain(self.bits()),
                                Capability::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator for Capability {
                        type Item = Capability;
                        type IntoIter = ::bitflags::iter::Iter<Capability>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                };
                ///These errors can be emitted in response to wl_seat requests.
                #[repr(u32)]
                pub enum Error {
                    ///get_pointer, get_keyboard or get_touch called on seat without the matching capability
                    MissingCapability = 0u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(f, "MissingCapability")
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        true
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        ::core::cmp::Ordering::Equal
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::MissingCapability,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod pointer {
            pub mod request {
                pub struct SetCursor {
                    serial: u32,
                    surface: ::std::option::Option<crate::object::WlObjectId>,
                    hotspot_x: i32,
                    hotspot_y: i32,
                }
                impl crate::object::HasObjectType for SetCursor {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Pointer;
                }
                impl<'s> crate::interface::Request<'s> for SetCursor {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.serial)
                            .maybe_object(
                                self.surface.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .int(self.hotspot_x)
                            .int(self.hotspot_y)
                            .build()
                    }
                }
                pub struct Release;
                impl crate::object::HasObjectType for Release {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Pointer;
                }
                impl<'s> crate::interface::Request<'s> for Release {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///given wl_surface has another role
                    Role = 0u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(f, "Role")
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        true
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        ::core::cmp::Ordering::Equal
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Role,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
                /**Describes the physical state of a button that produced the button
	event.*/
                #[repr(u32)]
                pub enum ButtonState {
                    ///the button is not pressed
                    Released = 0u32,
                    ///the button is pressed
                    Pressed = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for ButtonState {
                    #[inline]
                    fn clone(&self) -> ButtonState {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for ButtonState {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                ButtonState::Released => "Released",
                                ButtonState::Pressed => "Pressed",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for ButtonState {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for ButtonState {
                    #[inline]
                    fn eq(&self, other: &ButtonState) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for ButtonState {}
                #[automatically_derived]
                impl ::core::cmp::Eq for ButtonState {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for ButtonState {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &ButtonState,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for ButtonState {
                    #[inline]
                    fn cmp(&self, other: &ButtonState) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for ButtonState {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl ButtonState {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<ButtonState> for u32 {
                    fn from(value: ButtonState) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to ButtonState")]
                pub struct ButtonStateFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ButtonStateFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ButtonStateFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ButtonStateFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ButtonStateFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to ButtonState",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for ButtonState {
                    type Error = ButtonStateFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Released,
                                1u32 => Self::Pressed,
                                _ => return Err(ButtonStateFromU32Error(value)),
                            },
                        )
                    }
                }
                ///Describes the axis types of scroll events.
                #[repr(u32)]
                pub enum Axis {
                    ///vertical axis
                    VerticalScroll = 0u32,
                    ///horizontal axis
                    HorizontalScroll = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Axis {
                    #[inline]
                    fn clone(&self) -> Axis {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Axis {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Axis::VerticalScroll => "VerticalScroll",
                                Axis::HorizontalScroll => "HorizontalScroll",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Axis {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Axis {
                    #[inline]
                    fn eq(&self, other: &Axis) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Axis {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Axis {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Axis {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Axis,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Axis {
                    #[inline]
                    fn cmp(&self, other: &Axis) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Axis {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Axis {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Axis> for u32 {
                    fn from(value: Axis) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Axis")]
                pub struct AxisFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for AxisFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "AxisFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for AxisFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for AxisFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Axis", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Axis {
                    type Error = AxisFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::VerticalScroll,
                                1u32 => Self::HorizontalScroll,
                                _ => return Err(AxisFromU32Error(value)),
                            },
                        )
                    }
                }
                /**Describes the source types for axis events. This indicates to the
	client how an axis event was physically generated; a client may
	adjust the user interface accordingly. For example, scroll events
	from a "finger" source may be in a smooth coordinate space with
	kinetic scrolling whereas a "wheel" source may be in discrete steps
	of a number of lines.

	The "continuous" axis source is a device generating events in a
	continuous coordinate space, but using something other than a
	finger. One example for this source is button-based scrolling where
	the vertical motion of a device is converted to scroll events while
	a button is held down.

	The "wheel tilt" axis source indicates that the actual device is a
	wheel but the scroll event is not caused by a rotation but a
	(usually sideways) tilt of the wheel.*/
                #[repr(u32)]
                pub enum AxisSource {
                    ///a physical wheel rotation
                    Wheel = 0u32,
                    ///finger on a touch surface
                    Finger = 1u32,
                    ///continuous coordinate space
                    Continuous = 2u32,
                    ///a physical wheel tilt
                    WheelTilt = 3u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for AxisSource {
                    #[inline]
                    fn clone(&self) -> AxisSource {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for AxisSource {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                AxisSource::Wheel => "Wheel",
                                AxisSource::Finger => "Finger",
                                AxisSource::Continuous => "Continuous",
                                AxisSource::WheelTilt => "WheelTilt",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for AxisSource {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for AxisSource {
                    #[inline]
                    fn eq(&self, other: &AxisSource) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for AxisSource {}
                #[automatically_derived]
                impl ::core::cmp::Eq for AxisSource {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for AxisSource {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &AxisSource,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for AxisSource {
                    #[inline]
                    fn cmp(&self, other: &AxisSource) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for AxisSource {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl AxisSource {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<AxisSource> for u32 {
                    fn from(value: AxisSource) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to AxisSource")]
                pub struct AxisSourceFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for AxisSourceFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "AxisSourceFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for AxisSourceFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for AxisSourceFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to AxisSource",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for AxisSource {
                    type Error = AxisSourceFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Wheel,
                                1u32 => Self::Finger,
                                2u32 => Self::Continuous,
                                3u32 => Self::WheelTilt,
                                _ => return Err(AxisSourceFromU32Error(value)),
                            },
                        )
                    }
                }
                /**This specifies the direction of the physical motion that caused a
	wl_pointer.axis event, relative to the wl_pointer.axis direction.*/
                #[repr(u32)]
                pub enum AxisRelativeDirection {
                    ///physical motion matches axis direction
                    Identical = 0u32,
                    ///physical motion is the inverse of the axis direction
                    Inverted = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for AxisRelativeDirection {
                    #[inline]
                    fn clone(&self) -> AxisRelativeDirection {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for AxisRelativeDirection {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                AxisRelativeDirection::Identical => "Identical",
                                AxisRelativeDirection::Inverted => "Inverted",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for AxisRelativeDirection {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for AxisRelativeDirection {
                    #[inline]
                    fn eq(&self, other: &AxisRelativeDirection) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for AxisRelativeDirection {}
                #[automatically_derived]
                impl ::core::cmp::Eq for AxisRelativeDirection {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for AxisRelativeDirection {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &AxisRelativeDirection,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for AxisRelativeDirection {
                    #[inline]
                    fn cmp(
                        &self,
                        other: &AxisRelativeDirection,
                    ) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for AxisRelativeDirection {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl AxisRelativeDirection {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<AxisRelativeDirection> for u32 {
                    fn from(value: AxisRelativeDirection) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to AxisRelativeDirection")]
                pub struct AxisRelativeDirectionFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for AxisRelativeDirectionFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "AxisRelativeDirectionFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error
                for AxisRelativeDirectionFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for AxisRelativeDirectionFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to AxisRelativeDirection",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for AxisRelativeDirection {
                    type Error = AxisRelativeDirectionFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Identical,
                                1u32 => Self::Inverted,
                                _ => return Err(AxisRelativeDirectionFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod keyboard {
            pub mod request {
                pub struct Release;
                impl crate::object::HasObjectType for Release {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Keyboard;
                }
                impl<'s> crate::interface::Request<'s> for Release {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                /**This specifies the format of the keymap provided to the
	client with the wl_keyboard.keymap event.*/
                #[repr(u32)]
                pub enum KeymapFormat {
                    ///no keymap; client must understand how to interpret the raw keycode
                    NoKeymap = 0u32,
                    ///libxkbcommon compatible, null-terminated string; to determine the xkb keycode, clients must add 8 to the key event keycode
                    XkbV1 = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for KeymapFormat {
                    #[inline]
                    fn clone(&self) -> KeymapFormat {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for KeymapFormat {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                KeymapFormat::NoKeymap => "NoKeymap",
                                KeymapFormat::XkbV1 => "XkbV1",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for KeymapFormat {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for KeymapFormat {
                    #[inline]
                    fn eq(&self, other: &KeymapFormat) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for KeymapFormat {}
                #[automatically_derived]
                impl ::core::cmp::Eq for KeymapFormat {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for KeymapFormat {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &KeymapFormat,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for KeymapFormat {
                    #[inline]
                    fn cmp(&self, other: &KeymapFormat) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for KeymapFormat {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl KeymapFormat {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<KeymapFormat> for u32 {
                    fn from(value: KeymapFormat) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to KeymapFormat")]
                pub struct KeymapFormatFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for KeymapFormatFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "KeymapFormatFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for KeymapFormatFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for KeymapFormatFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to KeymapFormat",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for KeymapFormat {
                    type Error = KeymapFormatFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::NoKeymap,
                                1u32 => Self::XkbV1,
                                _ => return Err(KeymapFormatFromU32Error(value)),
                            },
                        )
                    }
                }
                ///Describes the physical state of a key that produced the key event.
                #[repr(u32)]
                pub enum KeyState {
                    ///key is not pressed
                    Released = 0u32,
                    ///key is pressed
                    Pressed = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for KeyState {
                    #[inline]
                    fn clone(&self) -> KeyState {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for KeyState {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                KeyState::Released => "Released",
                                KeyState::Pressed => "Pressed",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for KeyState {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for KeyState {
                    #[inline]
                    fn eq(&self, other: &KeyState) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for KeyState {}
                #[automatically_derived]
                impl ::core::cmp::Eq for KeyState {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for KeyState {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &KeyState,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for KeyState {
                    #[inline]
                    fn cmp(&self, other: &KeyState) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for KeyState {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl KeyState {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<KeyState> for u32 {
                    fn from(value: KeyState) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to KeyState")]
                pub struct KeyStateFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for KeyStateFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "KeyStateFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for KeyStateFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for KeyStateFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to KeyState",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for KeyState {
                    type Error = KeyStateFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Released,
                                1u32 => Self::Pressed,
                                _ => return Err(KeyStateFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod touch {
            pub mod request {
                pub struct Release;
                impl crate::object::HasObjectType for Release {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Touch;
                }
                impl<'s> crate::interface::Request<'s> for Release {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {}
        }
        pub mod output {
            pub mod request {
                pub struct Release;
                impl crate::object::HasObjectType for Release {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Output;
                }
                impl<'s> crate::interface::Request<'s> for Release {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                /**This enumeration describes how the physical
	pixels on an output are laid out.*/
                #[repr(u32)]
                pub enum Subpixel {
                    ///unknown geometry
                    Unknown = 0u32,
                    ///no geometry
                    None = 1u32,
                    ///horizontal RGB
                    HorizontalRgb = 2u32,
                    ///horizontal BGR
                    HorizontalBgr = 3u32,
                    ///vertical RGB
                    VerticalRgb = 4u32,
                    ///vertical BGR
                    VerticalBgr = 5u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Subpixel {
                    #[inline]
                    fn clone(&self) -> Subpixel {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Subpixel {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Subpixel::Unknown => "Unknown",
                                Subpixel::None => "None",
                                Subpixel::HorizontalRgb => "HorizontalRgb",
                                Subpixel::HorizontalBgr => "HorizontalBgr",
                                Subpixel::VerticalRgb => "VerticalRgb",
                                Subpixel::VerticalBgr => "VerticalBgr",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Subpixel {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Subpixel {
                    #[inline]
                    fn eq(&self, other: &Subpixel) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Subpixel {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Subpixel {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Subpixel {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Subpixel,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Subpixel {
                    #[inline]
                    fn cmp(&self, other: &Subpixel) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Subpixel {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Subpixel {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Subpixel> for u32 {
                    fn from(value: Subpixel) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Subpixel")]
                pub struct SubpixelFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for SubpixelFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "SubpixelFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for SubpixelFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for SubpixelFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to Subpixel",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Subpixel {
                    type Error = SubpixelFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Unknown,
                                1u32 => Self::None,
                                2u32 => Self::HorizontalRgb,
                                3u32 => Self::HorizontalBgr,
                                4u32 => Self::VerticalRgb,
                                5u32 => Self::VerticalBgr,
                                _ => return Err(SubpixelFromU32Error(value)),
                            },
                        )
                    }
                }
                /**This describes transformations that clients and compositors apply to
	buffer contents.

	The flipped values correspond to an initial flip around a
	vertical axis followed by rotation.

	The purpose is mainly to allow clients to render accordingly and
	tell the compositor, so that for fullscreen surfaces, the
	compositor will still be able to scan out directly from client
	surfaces.*/
                #[repr(u32)]
                pub enum Transform {
                    ///no transform
                    Normal = 0u32,
                    ///90 degrees counter-clockwise
                    _90 = 1u32,
                    ///180 degrees counter-clockwise
                    _180 = 2u32,
                    ///270 degrees counter-clockwise
                    _270 = 3u32,
                    ///180 degree flip around a vertical axis
                    Flipped = 4u32,
                    ///flip and rotate 90 degrees counter-clockwise
                    Flipped90 = 5u32,
                    ///flip and rotate 180 degrees counter-clockwise
                    Flipped180 = 6u32,
                    ///flip and rotate 270 degrees counter-clockwise
                    Flipped270 = 7u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Transform {
                    #[inline]
                    fn clone(&self) -> Transform {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Transform {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Transform::Normal => "Normal",
                                Transform::_90 => "_90",
                                Transform::_180 => "_180",
                                Transform::_270 => "_270",
                                Transform::Flipped => "Flipped",
                                Transform::Flipped90 => "Flipped90",
                                Transform::Flipped180 => "Flipped180",
                                Transform::Flipped270 => "Flipped270",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Transform {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Transform {
                    #[inline]
                    fn eq(&self, other: &Transform) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Transform {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Transform {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Transform {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Transform,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Transform {
                    #[inline]
                    fn cmp(&self, other: &Transform) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Transform {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Transform {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Transform> for u32 {
                    fn from(value: Transform) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Transform")]
                pub struct TransformFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for TransformFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "TransformFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for TransformFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for TransformFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to Transform",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Transform {
                    type Error = TransformFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Normal,
                                1u32 => Self::_90,
                                2u32 => Self::_180,
                                3u32 => Self::_270,
                                4u32 => Self::Flipped,
                                5u32 => Self::Flipped90,
                                6u32 => Self::Flipped180,
                                7u32 => Self::Flipped270,
                                _ => return Err(TransformFromU32Error(value)),
                            },
                        )
                    }
                }
                /**These flags describe properties of an output mode.
	They are used in the flags bitfield of the mode event.*/
                pub struct Mode(<Mode as ::bitflags::__private::PublicFlags>::Internal);
                #[automatically_derived]
                impl ::core::fmt::Debug for Mode {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Mode",
                            &&self.0,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Mode {
                    #[inline]
                    fn clone(&self) -> Mode {
                        let _: ::core::clone::AssertParamIsClone<
                            <Mode as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Mode {}
                #[automatically_derived]
                impl ::core::hash::Hash for Mode {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        ::core::hash::Hash::hash(&self.0, state)
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Mode {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Mode {
                    #[inline]
                    fn eq(&self, other: &Mode) -> bool {
                        self.0 == other.0
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Eq for Mode {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {
                        let _: ::core::cmp::AssertParamIsEq<
                            <Mode as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Mode {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Mode,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Mode {
                    #[inline]
                    fn cmp(&self, other: &Mode) -> ::core::cmp::Ordering {
                        ::core::cmp::Ord::cmp(&self.0, &other.0)
                    }
                }
                impl Mode {
                    ///indicates this is the current mode
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const CURRENT: Self = Self::from_bits_retain(1);
                    ///indicates this is the preferred mode
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const PREFERRED: Self = Self::from_bits_retain(2);
                }
                impl ::bitflags::Flags for Mode {
                    const FLAGS: &'static [::bitflags::Flag<Mode>] = &[
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("CURRENT", Mode::CURRENT)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("PREFERRED", Mode::PREFERRED)
                        },
                    ];
                    type Bits = u32;
                    fn bits(&self) -> u32 {
                        Mode::bits(self)
                    }
                    fn from_bits_retain(bits: u32) -> Mode {
                        Mode::from_bits_retain(bits)
                    }
                }
                #[allow(
                    dead_code,
                    deprecated,
                    unused_doc_comments,
                    unused_attributes,
                    unused_mut,
                    unused_imports,
                    non_upper_case_globals,
                    clippy::assign_op_pattern,
                    clippy::indexing_slicing,
                    clippy::same_name_method,
                    clippy::iter_without_into_iter,
                )]
                const _: () = {
                    #[repr(transparent)]
                    pub struct InternalBitFlags(u32);
                    #[automatically_derived]
                    impl ::core::clone::Clone for InternalBitFlags {
                        #[inline]
                        fn clone(&self) -> InternalBitFlags {
                            let _: ::core::clone::AssertParamIsClone<u32>;
                            *self
                        }
                    }
                    #[automatically_derived]
                    impl ::core::marker::Copy for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::marker::StructuralPartialEq for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::cmp::PartialEq for InternalBitFlags {
                        #[inline]
                        fn eq(&self, other: &InternalBitFlags) -> bool {
                            self.0 == other.0
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Eq for InternalBitFlags {
                        #[inline]
                        #[doc(hidden)]
                        #[coverage(off)]
                        fn assert_receiver_is_total_eq(&self) -> () {
                            let _: ::core::cmp::AssertParamIsEq<u32>;
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::PartialOrd for InternalBitFlags {
                        #[inline]
                        fn partial_cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::option::Option<::core::cmp::Ordering> {
                            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Ord for InternalBitFlags {
                        #[inline]
                        fn cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::cmp::Ordering {
                            ::core::cmp::Ord::cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::hash::Hash for InternalBitFlags {
                        #[inline]
                        fn hash<__H: ::core::hash::Hasher>(
                            &self,
                            state: &mut __H,
                        ) -> () {
                            ::core::hash::Hash::hash(&self.0, state)
                        }
                    }
                    impl ::bitflags::__private::PublicFlags for Mode {
                        type Primitive = u32;
                        type Internal = InternalBitFlags;
                    }
                    impl ::bitflags::__private::core::default::Default
                    for InternalBitFlags {
                        #[inline]
                        fn default() -> Self {
                            InternalBitFlags::empty()
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Debug for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            if self.is_empty() {
                                f.write_fmt(
                                    format_args!("{0:#x}", <u32 as ::bitflags::Bits>::EMPTY),
                                )
                            } else {
                                ::bitflags::__private::core::fmt::Display::fmt(self, f)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Display for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            ::bitflags::parser::to_writer(&Mode(*self), f)
                        }
                    }
                    impl ::bitflags::__private::core::str::FromStr for InternalBitFlags {
                        type Err = ::bitflags::parser::ParseError;
                        fn from_str(
                            s: &str,
                        ) -> ::bitflags::__private::core::result::Result<
                            Self,
                            Self::Err,
                        > {
                            ::bitflags::parser::from_str::<Mode>(s).map(|flags| flags.0)
                        }
                    }
                    impl ::bitflags::__private::core::convert::AsRef<u32>
                    for InternalBitFlags {
                        fn as_ref(&self) -> &u32 {
                            &self.0
                        }
                    }
                    impl ::bitflags::__private::core::convert::From<u32>
                    for InternalBitFlags {
                        fn from(bits: u32) -> Self {
                            Self::from_bits_retain(bits)
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl InternalBitFlags {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(<u32 as ::bitflags::Bits>::EMPTY) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            {
                                let mut truncated = <u32 as ::bitflags::Bits>::EMPTY;
                                let mut i = 0;
                                {
                                    {
                                        let flag = <Mode as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Mode as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                let _ = i;
                                Self::from_bits_retain(truncated)
                            }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0 }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                let truncated = Self::from_bits_truncate(bits).0;
                                if truncated == bits {
                                    ::bitflags::__private::core::option::Option::Some(
                                        Self(bits),
                                    )
                                } else {
                                    ::bitflags::__private::core::option::Option::None
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits & Self::all().bits()) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                {
                                    if name == "CURRENT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Mode::CURRENT.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "PREFERRED" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Mode::PREFERRED.bits()),
                                        );
                                    }
                                };
                                let _ = name;
                                ::bitflags::__private::core::option::Option::None
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.bits() == <u32 as ::bitflags::Bits>::EMPTY }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { Self::all().bits() | f.bits() == f.bits() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            {
                                f.bits() & other.bits() != <u32 as ::bitflags::Bits>::EMPTY
                            }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.bits() & other.bits() == other.bits() }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).union(other);
                            }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).difference(other);
                            }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits())
                                    .symmetric_difference(other);
                            }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            {
                                if value {
                                    f.insert(other);
                                } else {
                                    f.remove(other);
                                }
                            }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & other.bits()) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() | other.bits()) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & !other.bits()) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() ^ other.bits()) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self::from_bits_truncate(!f.bits()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: InternalBitFlags) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for InternalBitFlags {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for InternalBitFlags {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for InternalBitFlags {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for InternalBitFlags {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<InternalBitFlags>
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        InternalBitFlags,
                    > for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl InternalBitFlags {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Mode> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Mode as ::bitflags::Flags>::FLAGS,
                                Mode::from_bits_retain(self.bits()),
                                Mode::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Mode> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Mode as ::bitflags::Flags>::FLAGS,
                                Mode::from_bits_retain(self.bits()),
                                Mode::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for InternalBitFlags {
                        type Item = Mode;
                        type IntoIter = ::bitflags::iter::Iter<Mode>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                    impl InternalBitFlags {
                        /// Returns a mutable reference to the raw value of the flags currently stored.
                        #[inline]
                        pub fn bits_mut(&mut self) -> &mut u32 {
                            &mut self.0
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl Mode {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(InternalBitFlags::empty()) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            { Self(InternalBitFlags::all()) }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0.bits() }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                match InternalBitFlags::from_bits(bits) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_truncate(bits)) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_retain(bits)) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                match InternalBitFlags::from_name(name) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.0.is_empty() }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { f.0.is_all() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.intersects(other.0) }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.contains(other.0) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.insert(other.0) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.remove(other.0) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.toggle(other.0) }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            { f.0.set(other.0, value) }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.intersection(other.0)) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.union(other.0)) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.difference(other.0)) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.symmetric_difference(other.0)) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self(f.0.complement()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for Mode {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for Mode {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex for Mode {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex for Mode {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for Mode {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: Mode) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign for Mode {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for Mode {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign for Mode {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for Mode {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign for Mode {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for Mode {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign for Mode {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for Mode {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<Mode> for Mode {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<Mode> for Mode {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl Mode {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Mode> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Mode as ::bitflags::Flags>::FLAGS,
                                Mode::from_bits_retain(self.bits()),
                                Mode::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Mode> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Mode as ::bitflags::Flags>::FLAGS,
                                Mode::from_bits_retain(self.bits()),
                                Mode::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator for Mode {
                        type Item = Mode;
                        type IntoIter = ::bitflags::iter::Iter<Mode>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                };
            }
        }
        pub mod region {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Region;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct Add {
                    x: i32,
                    y: i32,
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for Add {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Region;
                }
                impl<'s> crate::interface::Request<'s> for Add {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct Subtract {
                    x: i32,
                    y: i32,
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for Subtract {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Region;
                }
                impl<'s> crate::interface::Request<'s> for Subtract {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {}
        }
        pub mod subcompositor {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subcompositor;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct GetSubsurface {
                    surface: crate::object::WlObjectId,
                    parent: crate::object::WlObjectId,
                }
                impl crate::interface::ObjectParent for GetSubsurface {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subsurface;
                }
                impl crate::object::HasObjectType for GetSubsurface {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subcompositor;
                }
                impl<'s> crate::interface::Request<'s> for GetSubsurface {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::Subsurface,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.surface).unwrap(),
                                ),
                            )
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.parent).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///the to-be sub-surface is invalid
                    BadSurface = 0u32,
                    ///the to-be sub-surface parent is invalid
                    BadParent = 1u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::BadSurface => "BadSurface",
                                Error::BadParent => "BadParent",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::BadSurface,
                                1u32 => Self::BadParent,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod subsurface {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subsurface;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetPosition {
                    x: i32,
                    y: i32,
                }
                impl crate::object::HasObjectType for SetPosition {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subsurface;
                }
                impl<'s> crate::interface::Request<'s> for SetPosition {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .build()
                    }
                }
                pub struct PlaceAbove {
                    sibling: crate::object::WlObjectId,
                }
                impl crate::object::HasObjectType for PlaceAbove {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subsurface;
                }
                impl<'s> crate::interface::Request<'s> for PlaceAbove {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.sibling).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
                pub struct PlaceBelow {
                    sibling: crate::object::WlObjectId,
                }
                impl crate::object::HasObjectType for PlaceBelow {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subsurface;
                }
                impl<'s> crate::interface::Request<'s> for PlaceBelow {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.sibling).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
                pub struct SetSync;
                impl crate::object::HasObjectType for SetSync {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subsurface;
                }
                impl<'s> crate::interface::Request<'s> for SetSync {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetDesync;
                impl crate::object::HasObjectType for SetDesync {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::Subsurface;
                }
                impl<'s> crate::interface::Request<'s> for SetDesync {
                    const CODE: crate::sys::wire::OpCode = 5;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///wl_surface is not a sibling or the parent
                    BadSurface = 0u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(f, "BadSurface")
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        true
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        ::core::cmp::Ordering::Equal
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::BadSurface,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
    }
    pub mod xdg_shell {
        pub mod xdg_wm_base {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgWmBase;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct CreatePositioner;
                impl crate::interface::ObjectParent for CreatePositioner {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl crate::object::HasObjectType for CreatePositioner {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgWmBase;
                }
                impl<'s> crate::interface::Request<'s> for CreatePositioner {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::XdgPositioner,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct GetXdgSurface {
                    surface: crate::object::WlObjectId,
                }
                impl crate::interface::ObjectParent for GetXdgSurface {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgSurface;
                }
                impl crate::object::HasObjectType for GetXdgSurface {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgWmBase;
                }
                impl<'s> crate::interface::Request<'s> for GetXdgSurface {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::XdgSurface,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.surface).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
                pub struct Pong {
                    serial: u32,
                }
                impl crate::object::HasObjectType for Pong {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgWmBase;
                }
                impl<'s> crate::interface::Request<'s> for Pong {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.serial)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///given wl_surface has another role
                    Role = 0u32,
                    ///xdg_wm_base was destroyed before children
                    DefunctSurfaces = 1u32,
                    ///the client tried to map or destroy a non-topmost popup
                    NotTheTopmostPopup = 2u32,
                    ///the client specified an invalid popup parent surface
                    InvalidPopupParent = 3u32,
                    ///the client provided an invalid surface state
                    InvalidSurfaceState = 4u32,
                    ///the client provided an invalid positioner
                    InvalidPositioner = 5u32,
                    ///the client didn’t respond to a ping event in time
                    Unresponsive = 6u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::Role => "Role",
                                Error::DefunctSurfaces => "DefunctSurfaces",
                                Error::NotTheTopmostPopup => "NotTheTopmostPopup",
                                Error::InvalidPopupParent => "InvalidPopupParent",
                                Error::InvalidSurfaceState => "InvalidSurfaceState",
                                Error::InvalidPositioner => "InvalidPositioner",
                                Error::Unresponsive => "Unresponsive",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Role,
                                1u32 => Self::DefunctSurfaces,
                                2u32 => Self::NotTheTopmostPopup,
                                3u32 => Self::InvalidPopupParent,
                                4u32 => Self::InvalidSurfaceState,
                                5u32 => Self::InvalidPositioner,
                                6u32 => Self::Unresponsive,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod xdg_positioner {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetSize {
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for SetSize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetSize {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct SetAnchorRect {
                    x: i32,
                    y: i32,
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for SetAnchorRect {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetAnchorRect {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct SetAnchor {
                    anchor: u32,
                }
                impl crate::object::HasObjectType for SetAnchor {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetAnchor {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.anchor)
                            .build()
                    }
                }
                pub struct SetGravity {
                    gravity: u32,
                }
                impl crate::object::HasObjectType for SetGravity {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetGravity {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.gravity)
                            .build()
                    }
                }
                pub struct SetConstraintAdjustment {
                    constraint_adjustment: u32,
                }
                impl crate::object::HasObjectType for SetConstraintAdjustment {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetConstraintAdjustment {
                    const CODE: crate::sys::wire::OpCode = 5;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.constraint_adjustment)
                            .build()
                    }
                }
                pub struct SetOffset {
                    x: i32,
                    y: i32,
                }
                impl crate::object::HasObjectType for SetOffset {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetOffset {
                    const CODE: crate::sys::wire::OpCode = 6;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .build()
                    }
                }
                pub struct SetReactive;
                impl crate::object::HasObjectType for SetReactive {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetReactive {
                    const CODE: crate::sys::wire::OpCode = 7;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetParentSize {
                    parent_width: i32,
                    parent_height: i32,
                }
                impl crate::object::HasObjectType for SetParentSize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetParentSize {
                    const CODE: crate::sys::wire::OpCode = 8;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.parent_width)
                            .int(self.parent_height)
                            .build()
                    }
                }
                pub struct SetParentConfigure {
                    serial: u32,
                }
                impl crate::object::HasObjectType for SetParentConfigure {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPositioner;
                }
                impl<'s> crate::interface::Request<'s> for SetParentConfigure {
                    const CODE: crate::sys::wire::OpCode = 9;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.serial)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///invalid input provided
                    InvalidInput = 0u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(f, "InvalidInput")
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        true
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        ::core::cmp::Ordering::Equal
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidInput,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
                #[repr(u32)]
                pub enum Anchor {
                    None = 0u32,
                    Top = 1u32,
                    Bottom = 2u32,
                    Left = 3u32,
                    Right = 4u32,
                    TopLeft = 5u32,
                    BottomLeft = 6u32,
                    TopRight = 7u32,
                    BottomRight = 8u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Anchor {
                    #[inline]
                    fn clone(&self) -> Anchor {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Anchor {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Anchor::None => "None",
                                Anchor::Top => "Top",
                                Anchor::Bottom => "Bottom",
                                Anchor::Left => "Left",
                                Anchor::Right => "Right",
                                Anchor::TopLeft => "TopLeft",
                                Anchor::BottomLeft => "BottomLeft",
                                Anchor::TopRight => "TopRight",
                                Anchor::BottomRight => "BottomRight",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Anchor {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Anchor {
                    #[inline]
                    fn eq(&self, other: &Anchor) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Anchor {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Anchor {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Anchor {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Anchor,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Anchor {
                    #[inline]
                    fn cmp(&self, other: &Anchor) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Anchor {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Anchor {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Anchor> for u32 {
                    fn from(value: Anchor) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Anchor")]
                pub struct AnchorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for AnchorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "AnchorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for AnchorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for AnchorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Anchor", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Anchor {
                    type Error = AnchorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::None,
                                1u32 => Self::Top,
                                2u32 => Self::Bottom,
                                3u32 => Self::Left,
                                4u32 => Self::Right,
                                5u32 => Self::TopLeft,
                                6u32 => Self::BottomLeft,
                                7u32 => Self::TopRight,
                                8u32 => Self::BottomRight,
                                _ => return Err(AnchorFromU32Error(value)),
                            },
                        )
                    }
                }
                #[repr(u32)]
                pub enum Gravity {
                    None = 0u32,
                    Top = 1u32,
                    Bottom = 2u32,
                    Left = 3u32,
                    Right = 4u32,
                    TopLeft = 5u32,
                    BottomLeft = 6u32,
                    TopRight = 7u32,
                    BottomRight = 8u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Gravity {
                    #[inline]
                    fn clone(&self) -> Gravity {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Gravity {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Gravity::None => "None",
                                Gravity::Top => "Top",
                                Gravity::Bottom => "Bottom",
                                Gravity::Left => "Left",
                                Gravity::Right => "Right",
                                Gravity::TopLeft => "TopLeft",
                                Gravity::BottomLeft => "BottomLeft",
                                Gravity::TopRight => "TopRight",
                                Gravity::BottomRight => "BottomRight",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Gravity {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Gravity {
                    #[inline]
                    fn eq(&self, other: &Gravity) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Gravity {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Gravity {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Gravity {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Gravity,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Gravity {
                    #[inline]
                    fn cmp(&self, other: &Gravity) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Gravity {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Gravity {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Gravity> for u32 {
                    fn from(value: Gravity) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Gravity")]
                pub struct GravityFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for GravityFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "GravityFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for GravityFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for GravityFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Gravity", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Gravity {
                    type Error = GravityFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::None,
                                1u32 => Self::Top,
                                2u32 => Self::Bottom,
                                3u32 => Self::Left,
                                4u32 => Self::Right,
                                5u32 => Self::TopLeft,
                                6u32 => Self::BottomLeft,
                                7u32 => Self::TopRight,
                                8u32 => Self::BottomRight,
                                _ => return Err(GravityFromU32Error(value)),
                            },
                        )
                    }
                }
                /**The constraint adjustment value define ways the compositor will adjust
	the position of the surface, if the unadjusted position would result
	in the surface being partly constrained.

	Whether a surface is considered 'constrained' is left to the compositor
	to determine. For example, the surface may be partly outside the
	compositor's defined 'work area', thus necessitating the child surface's
	position be adjusted until it is entirely inside the work area.

	The adjustments can be combined, according to a defined precedence: 1)
	Flip, 2) Slide, 3) Resize.*/
                pub struct ConstraintAdjustment(
                    <ConstraintAdjustment as ::bitflags::__private::PublicFlags>::Internal,
                );
                #[automatically_derived]
                impl ::core::fmt::Debug for ConstraintAdjustment {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ConstraintAdjustment",
                            &&self.0,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::clone::Clone for ConstraintAdjustment {
                    #[inline]
                    fn clone(&self) -> ConstraintAdjustment {
                        let _: ::core::clone::AssertParamIsClone<
                            <ConstraintAdjustment as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for ConstraintAdjustment {}
                #[automatically_derived]
                impl ::core::hash::Hash for ConstraintAdjustment {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        ::core::hash::Hash::hash(&self.0, state)
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for ConstraintAdjustment {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for ConstraintAdjustment {
                    #[inline]
                    fn eq(&self, other: &ConstraintAdjustment) -> bool {
                        self.0 == other.0
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Eq for ConstraintAdjustment {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {
                        let _: ::core::cmp::AssertParamIsEq<
                            <ConstraintAdjustment as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for ConstraintAdjustment {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &ConstraintAdjustment,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for ConstraintAdjustment {
                    #[inline]
                    fn cmp(
                        &self,
                        other: &ConstraintAdjustment,
                    ) -> ::core::cmp::Ordering {
                        ::core::cmp::Ord::cmp(&self.0, &other.0)
                    }
                }
                impl ConstraintAdjustment {
                    ///don't move the child surface when constrained
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const NONE: Self = Self::from_bits_retain(0);
                    ///move along the x axis until unconstrained
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const SLIDE_X: Self = Self::from_bits_retain(1);
                    ///move along the y axis until unconstrained
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const SLIDE_Y: Self = Self::from_bits_retain(2);
                    ///invert the anchor and gravity on the x axis
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const FLIP_X: Self = Self::from_bits_retain(4);
                    ///invert the anchor and gravity on the y axis
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const FLIP_Y: Self = Self::from_bits_retain(8);
                    ///horizontally resize the surface
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const RESIZE_X: Self = Self::from_bits_retain(16);
                    ///vertically resize the surface
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const RESIZE_Y: Self = Self::from_bits_retain(32);
                }
                impl ::bitflags::Flags for ConstraintAdjustment {
                    const FLAGS: &'static [::bitflags::Flag<ConstraintAdjustment>] = &[
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("NONE", ConstraintAdjustment::NONE)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new(
                                "SLIDE_X",
                                ConstraintAdjustment::SLIDE_X,
                            )
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new(
                                "SLIDE_Y",
                                ConstraintAdjustment::SLIDE_Y,
                            )
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("FLIP_X", ConstraintAdjustment::FLIP_X)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("FLIP_Y", ConstraintAdjustment::FLIP_Y)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new(
                                "RESIZE_X",
                                ConstraintAdjustment::RESIZE_X,
                            )
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new(
                                "RESIZE_Y",
                                ConstraintAdjustment::RESIZE_Y,
                            )
                        },
                    ];
                    type Bits = u32;
                    fn bits(&self) -> u32 {
                        ConstraintAdjustment::bits(self)
                    }
                    fn from_bits_retain(bits: u32) -> ConstraintAdjustment {
                        ConstraintAdjustment::from_bits_retain(bits)
                    }
                }
                #[allow(
                    dead_code,
                    deprecated,
                    unused_doc_comments,
                    unused_attributes,
                    unused_mut,
                    unused_imports,
                    non_upper_case_globals,
                    clippy::assign_op_pattern,
                    clippy::indexing_slicing,
                    clippy::same_name_method,
                    clippy::iter_without_into_iter,
                )]
                const _: () = {
                    #[repr(transparent)]
                    pub struct InternalBitFlags(u32);
                    #[automatically_derived]
                    impl ::core::clone::Clone for InternalBitFlags {
                        #[inline]
                        fn clone(&self) -> InternalBitFlags {
                            let _: ::core::clone::AssertParamIsClone<u32>;
                            *self
                        }
                    }
                    #[automatically_derived]
                    impl ::core::marker::Copy for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::marker::StructuralPartialEq for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::cmp::PartialEq for InternalBitFlags {
                        #[inline]
                        fn eq(&self, other: &InternalBitFlags) -> bool {
                            self.0 == other.0
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Eq for InternalBitFlags {
                        #[inline]
                        #[doc(hidden)]
                        #[coverage(off)]
                        fn assert_receiver_is_total_eq(&self) -> () {
                            let _: ::core::cmp::AssertParamIsEq<u32>;
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::PartialOrd for InternalBitFlags {
                        #[inline]
                        fn partial_cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::option::Option<::core::cmp::Ordering> {
                            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Ord for InternalBitFlags {
                        #[inline]
                        fn cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::cmp::Ordering {
                            ::core::cmp::Ord::cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::hash::Hash for InternalBitFlags {
                        #[inline]
                        fn hash<__H: ::core::hash::Hasher>(
                            &self,
                            state: &mut __H,
                        ) -> () {
                            ::core::hash::Hash::hash(&self.0, state)
                        }
                    }
                    impl ::bitflags::__private::PublicFlags for ConstraintAdjustment {
                        type Primitive = u32;
                        type Internal = InternalBitFlags;
                    }
                    impl ::bitflags::__private::core::default::Default
                    for InternalBitFlags {
                        #[inline]
                        fn default() -> Self {
                            InternalBitFlags::empty()
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Debug for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            if self.is_empty() {
                                f.write_fmt(
                                    format_args!("{0:#x}", <u32 as ::bitflags::Bits>::EMPTY),
                                )
                            } else {
                                ::bitflags::__private::core::fmt::Display::fmt(self, f)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Display for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            ::bitflags::parser::to_writer(
                                &ConstraintAdjustment(*self),
                                f,
                            )
                        }
                    }
                    impl ::bitflags::__private::core::str::FromStr for InternalBitFlags {
                        type Err = ::bitflags::parser::ParseError;
                        fn from_str(
                            s: &str,
                        ) -> ::bitflags::__private::core::result::Result<
                            Self,
                            Self::Err,
                        > {
                            ::bitflags::parser::from_str::<ConstraintAdjustment>(s)
                                .map(|flags| flags.0)
                        }
                    }
                    impl ::bitflags::__private::core::convert::AsRef<u32>
                    for InternalBitFlags {
                        fn as_ref(&self) -> &u32 {
                            &self.0
                        }
                    }
                    impl ::bitflags::__private::core::convert::From<u32>
                    for InternalBitFlags {
                        fn from(bits: u32) -> Self {
                            Self::from_bits_retain(bits)
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl InternalBitFlags {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(<u32 as ::bitflags::Bits>::EMPTY) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            {
                                let mut truncated = <u32 as ::bitflags::Bits>::EMPTY;
                                let mut i = 0;
                                {
                                    {
                                        let flag = <ConstraintAdjustment as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <ConstraintAdjustment as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <ConstraintAdjustment as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <ConstraintAdjustment as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <ConstraintAdjustment as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <ConstraintAdjustment as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <ConstraintAdjustment as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                let _ = i;
                                Self::from_bits_retain(truncated)
                            }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0 }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                let truncated = Self::from_bits_truncate(bits).0;
                                if truncated == bits {
                                    ::bitflags::__private::core::option::Option::Some(
                                        Self(bits),
                                    )
                                } else {
                                    ::bitflags::__private::core::option::Option::None
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits & Self::all().bits()) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                {
                                    if name == "NONE" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(ConstraintAdjustment::NONE.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "SLIDE_X" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(ConstraintAdjustment::SLIDE_X.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "SLIDE_Y" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(ConstraintAdjustment::SLIDE_Y.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "FLIP_X" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(ConstraintAdjustment::FLIP_X.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "FLIP_Y" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(ConstraintAdjustment::FLIP_Y.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "RESIZE_X" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(ConstraintAdjustment::RESIZE_X.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "RESIZE_Y" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(ConstraintAdjustment::RESIZE_Y.bits()),
                                        );
                                    }
                                };
                                let _ = name;
                                ::bitflags::__private::core::option::Option::None
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.bits() == <u32 as ::bitflags::Bits>::EMPTY }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { Self::all().bits() | f.bits() == f.bits() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            {
                                f.bits() & other.bits() != <u32 as ::bitflags::Bits>::EMPTY
                            }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.bits() & other.bits() == other.bits() }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).union(other);
                            }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).difference(other);
                            }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits())
                                    .symmetric_difference(other);
                            }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            {
                                if value {
                                    f.insert(other);
                                } else {
                                    f.remove(other);
                                }
                            }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & other.bits()) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() | other.bits()) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & !other.bits()) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() ^ other.bits()) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self::from_bits_truncate(!f.bits()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: InternalBitFlags) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for InternalBitFlags {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for InternalBitFlags {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for InternalBitFlags {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for InternalBitFlags {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<InternalBitFlags>
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        InternalBitFlags,
                    > for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl InternalBitFlags {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(
                            &self,
                        ) -> ::bitflags::iter::Iter<ConstraintAdjustment> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <ConstraintAdjustment as ::bitflags::Flags>::FLAGS,
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<ConstraintAdjustment> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <ConstraintAdjustment as ::bitflags::Flags>::FLAGS,
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for InternalBitFlags {
                        type Item = ConstraintAdjustment;
                        type IntoIter = ::bitflags::iter::Iter<ConstraintAdjustment>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                    impl InternalBitFlags {
                        /// Returns a mutable reference to the raw value of the flags currently stored.
                        #[inline]
                        pub fn bits_mut(&mut self) -> &mut u32 {
                            &mut self.0
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl ConstraintAdjustment {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(InternalBitFlags::empty()) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            { Self(InternalBitFlags::all()) }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0.bits() }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                match InternalBitFlags::from_bits(bits) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_truncate(bits)) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_retain(bits)) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                match InternalBitFlags::from_name(name) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.0.is_empty() }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { f.0.is_all() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.intersects(other.0) }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.contains(other.0) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.insert(other.0) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.remove(other.0) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.toggle(other.0) }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            { f.0.set(other.0, value) }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.intersection(other.0)) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.union(other.0)) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.difference(other.0)) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.symmetric_difference(other.0)) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self(f.0.complement()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary
                    for ConstraintAdjustment {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal
                    for ConstraintAdjustment {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for ConstraintAdjustment {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for ConstraintAdjustment {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr
                    for ConstraintAdjustment {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: ConstraintAdjustment) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for ConstraintAdjustment {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor
                    for ConstraintAdjustment {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for ConstraintAdjustment {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd
                    for ConstraintAdjustment {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for ConstraintAdjustment {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for ConstraintAdjustment {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for ConstraintAdjustment {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for ConstraintAdjustment {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<ConstraintAdjustment>
                    for ConstraintAdjustment {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        ConstraintAdjustment,
                    > for ConstraintAdjustment {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl ConstraintAdjustment {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(
                            &self,
                        ) -> ::bitflags::iter::Iter<ConstraintAdjustment> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <ConstraintAdjustment as ::bitflags::Flags>::FLAGS,
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<ConstraintAdjustment> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <ConstraintAdjustment as ::bitflags::Flags>::FLAGS,
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                                ConstraintAdjustment::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for ConstraintAdjustment {
                        type Item = ConstraintAdjustment;
                        type IntoIter = ::bitflags::iter::Iter<ConstraintAdjustment>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                };
            }
        }
        pub mod xdg_surface {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgSurface;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct GetToplevel;
                impl crate::interface::ObjectParent for GetToplevel {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl crate::object::HasObjectType for GetToplevel {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgSurface;
                }
                impl<'s> crate::interface::Request<'s> for GetToplevel {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::XdgToplevel,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .build()
                    }
                }
                pub struct GetPopup {
                    parent: ::std::option::Option<crate::object::WlObjectId>,
                    positioner: crate::object::WlObjectId,
                }
                impl crate::interface::ObjectParent for GetPopup {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPopup;
                }
                impl crate::object::HasObjectType for GetPopup {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgSurface;
                }
                impl<'s> crate::interface::Request<'s> for GetPopup {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::XdgPopup,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .maybe_object(
                                self.parent.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.positioner).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
                pub struct SetWindowGeometry {
                    x: i32,
                    y: i32,
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for SetWindowGeometry {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetWindowGeometry {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.x)
                            .int(self.y)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct AckConfigure {
                    serial: u32,
                }
                impl crate::object::HasObjectType for AckConfigure {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgSurface;
                }
                impl<'s> crate::interface::Request<'s> for AckConfigure {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.serial)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///Surface was not fully constructed
                    NotConstructed = 1u32,
                    ///Surface was already constructed
                    AlreadyConstructed = 2u32,
                    ///Attaching a buffer to an unconfigured surface
                    UnconfiguredBuffer = 3u32,
                    ///Invalid serial number when acking a configure event
                    InvalidSerial = 4u32,
                    ///Width or height was zero or negative
                    InvalidSize = 5u32,
                    ///Surface was destroyed before its role object
                    DefunctRoleObject = 6u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::NotConstructed => "NotConstructed",
                                Error::AlreadyConstructed => "AlreadyConstructed",
                                Error::UnconfiguredBuffer => "UnconfiguredBuffer",
                                Error::InvalidSerial => "InvalidSerial",
                                Error::InvalidSize => "InvalidSize",
                                Error::DefunctRoleObject => "DefunctRoleObject",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                1u32 => Self::NotConstructed,
                                2u32 => Self::AlreadyConstructed,
                                3u32 => Self::UnconfiguredBuffer,
                                4u32 => Self::InvalidSerial,
                                5u32 => Self::InvalidSize,
                                6u32 => Self::DefunctRoleObject,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod xdg_toplevel {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetParent {
                    parent: ::std::option::Option<crate::object::WlObjectId>,
                }
                impl crate::object::HasObjectType for SetParent {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetParent {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.parent.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .build()
                    }
                }
                pub struct SetTitle<'s> {
                    title: &'s ::std::ffi::CStr,
                }
                impl crate::object::HasObjectType for SetTitle<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetTitle<'s> {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .str(self.title)
                            .build()
                    }
                }
                pub struct SetAppId<'s> {
                    app_id: &'s ::std::ffi::CStr,
                }
                impl crate::object::HasObjectType for SetAppId<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetAppId<'s> {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .str(self.app_id)
                            .build()
                    }
                }
                pub struct ShowWindowMenu {
                    seat: crate::object::WlObjectId,
                    serial: u32,
                    x: i32,
                    y: i32,
                }
                impl crate::object::HasObjectType for ShowWindowMenu {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for ShowWindowMenu {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .uint(self.serial)
                            .int(self.x)
                            .int(self.y)
                            .build()
                    }
                }
                pub struct Move {
                    seat: crate::object::WlObjectId,
                    serial: u32,
                }
                impl crate::object::HasObjectType for Move {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for Move {
                    const CODE: crate::sys::wire::OpCode = 5;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .uint(self.serial)
                            .build()
                    }
                }
                pub struct Resize {
                    seat: crate::object::WlObjectId,
                    serial: u32,
                    edges: u32,
                }
                impl crate::object::HasObjectType for Resize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for Resize {
                    const CODE: crate::sys::wire::OpCode = 6;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .uint(self.serial)
                            .uint(self.edges)
                            .build()
                    }
                }
                pub struct SetMaxSize {
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for SetMaxSize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetMaxSize {
                    const CODE: crate::sys::wire::OpCode = 7;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct SetMinSize {
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for SetMinSize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetMinSize {
                    const CODE: crate::sys::wire::OpCode = 8;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
                pub struct SetMaximized;
                impl crate::object::HasObjectType for SetMaximized {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetMaximized {
                    const CODE: crate::sys::wire::OpCode = 9;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct UnsetMaximized;
                impl crate::object::HasObjectType for UnsetMaximized {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for UnsetMaximized {
                    const CODE: crate::sys::wire::OpCode = 10;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetFullscreen {
                    output: ::std::option::Option<crate::object::WlObjectId>,
                }
                impl crate::object::HasObjectType for SetFullscreen {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetFullscreen {
                    const CODE: crate::sys::wire::OpCode = 11;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                self.output.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .build()
                    }
                }
                pub struct UnsetFullscreen;
                impl crate::object::HasObjectType for UnsetFullscreen {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for UnsetFullscreen {
                    const CODE: crate::sys::wire::OpCode = 12;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetMinimized;
                impl crate::object::HasObjectType for SetMinimized {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgToplevel;
                }
                impl<'s> crate::interface::Request<'s> for SetMinimized {
                    const CODE: crate::sys::wire::OpCode = 13;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    /**provided value is
        not a valid variant of the resize_edge enum*/
                    InvalidResizeEdge = 0u32,
                    ///invalid parent toplevel
                    InvalidParent = 1u32,
                    ///client provided an invalid min or max size
                    InvalidSize = 2u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::InvalidResizeEdge => "InvalidResizeEdge",
                                Error::InvalidParent => "InvalidParent",
                                Error::InvalidSize => "InvalidSize",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidResizeEdge,
                                1u32 => Self::InvalidParent,
                                2u32 => Self::InvalidSize,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
                /**These values are used to indicate which edge of a surface
	is being dragged in a resize operation.*/
                #[repr(u32)]
                pub enum ResizeEdge {
                    None = 0u32,
                    Top = 1u32,
                    Bottom = 2u32,
                    Left = 4u32,
                    TopLeft = 5u32,
                    BottomLeft = 6u32,
                    Right = 8u32,
                    TopRight = 9u32,
                    BottomRight = 10u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for ResizeEdge {
                    #[inline]
                    fn clone(&self) -> ResizeEdge {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for ResizeEdge {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                ResizeEdge::None => "None",
                                ResizeEdge::Top => "Top",
                                ResizeEdge::Bottom => "Bottom",
                                ResizeEdge::Left => "Left",
                                ResizeEdge::TopLeft => "TopLeft",
                                ResizeEdge::BottomLeft => "BottomLeft",
                                ResizeEdge::Right => "Right",
                                ResizeEdge::TopRight => "TopRight",
                                ResizeEdge::BottomRight => "BottomRight",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for ResizeEdge {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for ResizeEdge {
                    #[inline]
                    fn eq(&self, other: &ResizeEdge) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for ResizeEdge {}
                #[automatically_derived]
                impl ::core::cmp::Eq for ResizeEdge {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for ResizeEdge {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &ResizeEdge,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for ResizeEdge {
                    #[inline]
                    fn cmp(&self, other: &ResizeEdge) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for ResizeEdge {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl ResizeEdge {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<ResizeEdge> for u32 {
                    fn from(value: ResizeEdge) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to ResizeEdge")]
                pub struct ResizeEdgeFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ResizeEdgeFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ResizeEdgeFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ResizeEdgeFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ResizeEdgeFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to ResizeEdge",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for ResizeEdge {
                    type Error = ResizeEdgeFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::None,
                                1u32 => Self::Top,
                                2u32 => Self::Bottom,
                                4u32 => Self::Left,
                                5u32 => Self::TopLeft,
                                6u32 => Self::BottomLeft,
                                8u32 => Self::Right,
                                9u32 => Self::TopRight,
                                10u32 => Self::BottomRight,
                                _ => return Err(ResizeEdgeFromU32Error(value)),
                            },
                        )
                    }
                }
                /**The different state values used on the surface. This is designed for
	state values like maximized, fullscreen. It is paired with the
	configure event to ensure that both the client and the compositor
	setting the state can be synchronized.

	States set in this way are double-buffered, see wl_surface.commit.*/
                #[repr(u32)]
                pub enum State {
                    ///the surface is maximized
                    Maximized = 1u32,
                    ///the surface is fullscreen
                    Fullscreen = 2u32,
                    ///the surface is being resized
                    Resizing = 3u32,
                    ///the surface is now activated
                    Activated = 4u32,
                    ///the surface’s left edge is tiled
                    TiledLeft = 5u32,
                    ///the surface’s right edge is tiled
                    TiledRight = 6u32,
                    ///the surface’s top edge is tiled
                    TiledTop = 7u32,
                    ///the surface’s bottom edge is tiled
                    TiledBottom = 8u32,
                    ///surface repaint is suspended
                    Suspended = 9u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for State {
                    #[inline]
                    fn clone(&self) -> State {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for State {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                State::Maximized => "Maximized",
                                State::Fullscreen => "Fullscreen",
                                State::Resizing => "Resizing",
                                State::Activated => "Activated",
                                State::TiledLeft => "TiledLeft",
                                State::TiledRight => "TiledRight",
                                State::TiledTop => "TiledTop",
                                State::TiledBottom => "TiledBottom",
                                State::Suspended => "Suspended",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for State {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for State {
                    #[inline]
                    fn eq(&self, other: &State) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for State {}
                #[automatically_derived]
                impl ::core::cmp::Eq for State {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for State {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &State,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for State {
                    #[inline]
                    fn cmp(&self, other: &State) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for State {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl State {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<State> for u32 {
                    fn from(value: State) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to State")]
                pub struct StateFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for StateFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "StateFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for StateFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for StateFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to State", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for State {
                    type Error = StateFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                1u32 => Self::Maximized,
                                2u32 => Self::Fullscreen,
                                3u32 => Self::Resizing,
                                4u32 => Self::Activated,
                                5u32 => Self::TiledLeft,
                                6u32 => Self::TiledRight,
                                7u32 => Self::TiledTop,
                                8u32 => Self::TiledBottom,
                                9u32 => Self::Suspended,
                                _ => return Err(StateFromU32Error(value)),
                            },
                        )
                    }
                }
                #[repr(u32)]
                pub enum WmCapabilities {
                    ///show_window_menu is available
                    WindowMenu = 1u32,
                    ///set_maximized and unset_maximized are available
                    Maximize = 2u32,
                    ///set_fullscreen and unset_fullscreen are available
                    Fullscreen = 3u32,
                    ///set_minimized is available
                    Minimize = 4u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for WmCapabilities {
                    #[inline]
                    fn clone(&self) -> WmCapabilities {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for WmCapabilities {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                WmCapabilities::WindowMenu => "WindowMenu",
                                WmCapabilities::Maximize => "Maximize",
                                WmCapabilities::Fullscreen => "Fullscreen",
                                WmCapabilities::Minimize => "Minimize",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for WmCapabilities {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for WmCapabilities {
                    #[inline]
                    fn eq(&self, other: &WmCapabilities) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for WmCapabilities {}
                #[automatically_derived]
                impl ::core::cmp::Eq for WmCapabilities {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for WmCapabilities {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &WmCapabilities,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for WmCapabilities {
                    #[inline]
                    fn cmp(&self, other: &WmCapabilities) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for WmCapabilities {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl WmCapabilities {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<WmCapabilities> for u32 {
                    fn from(value: WmCapabilities) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to WmCapabilities")]
                pub struct WmCapabilitiesFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for WmCapabilitiesFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "WmCapabilitiesFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for WmCapabilitiesFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for WmCapabilitiesFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to WmCapabilities",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for WmCapabilities {
                    type Error = WmCapabilitiesFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                1u32 => Self::WindowMenu,
                                2u32 => Self::Maximize,
                                3u32 => Self::Fullscreen,
                                4u32 => Self::Minimize,
                                _ => return Err(WmCapabilitiesFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod xdg_popup {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPopup;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct Grab {
                    seat: crate::object::WlObjectId,
                    serial: u32,
                }
                impl crate::object::HasObjectType for Grab {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPopup;
                }
                impl<'s> crate::interface::Request<'s> for Grab {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.seat).unwrap(),
                                ),
                            )
                            .uint(self.serial)
                            .build()
                    }
                }
                pub struct Reposition {
                    positioner: crate::object::WlObjectId,
                    token: u32,
                }
                impl crate::object::HasObjectType for Reposition {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::XdgPopup;
                }
                impl<'s> crate::interface::Request<'s> for Reposition {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.positioner).unwrap(),
                                ),
                            )
                            .uint(self.token)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///tried to grab after being mapped
                    InvalidGrab = 0u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(f, "InvalidGrab")
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        true
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        ::core::cmp::Ordering::Equal
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidGrab,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
    }
    pub mod viewporter {
        pub mod wp_viewporter {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::WpViewporter;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct GetViewport {
                    surface: crate::object::WlObjectId,
                }
                impl crate::interface::ObjectParent for GetViewport {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::WpViewport;
                }
                impl crate::object::HasObjectType for GetViewport {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::WpViewporter;
                }
                impl<'s> crate::interface::Request<'s> for GetViewport {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::WpViewport,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.surface).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///the surface already has a viewport object associated
                    ViewportExists = 0u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(f, "ViewportExists")
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        true
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        ::core::cmp::Ordering::Equal
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::ViewportExists,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod wp_viewport {
            pub mod request {
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::WpViewport;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetSource {
                    x: wayland_sys::WlFixed,
                    y: wayland_sys::WlFixed,
                    width: wayland_sys::WlFixed,
                    height: wayland_sys::WlFixed,
                }
                impl crate::object::HasObjectType for SetSource {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::WpViewport;
                }
                impl<'s> crate::interface::Request<'s> for SetSource {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .fixed(self.x)
                            .fixed(self.y)
                            .fixed(self.width)
                            .fixed(self.height)
                            .build()
                    }
                }
                pub struct SetDestination {
                    width: i32,
                    height: i32,
                }
                impl crate::object::HasObjectType for SetDestination {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::WpViewport;
                }
                impl<'s> crate::interface::Request<'s> for SetDestination {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.width)
                            .int(self.height)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///negative or zero values in width or height
                    BadValue = 0u32,
                    ///destination size is not integer
                    BadSize = 1u32,
                    ///source rectangle extends outside of the content area
                    OutOfBuffer = 2u32,
                    ///the wl_surface was destroyed
                    NoSurface = 3u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::BadValue => "BadValue",
                                Error::BadSize => "BadSize",
                                Error::OutOfBuffer => "OutOfBuffer",
                                Error::NoSurface => "NoSurface",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::BadValue,
                                1u32 => Self::BadSize,
                                2u32 => Self::OutOfBuffer,
                                3u32 => Self::NoSurface,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
    }
    pub mod wlr_layer_shell_unstable_v1 {
        pub mod _layer_shell {
            pub mod request {
                pub struct GetLayerSurface<'s> {
                    surface: crate::object::WlObjectId,
                    output: ::std::option::Option<crate::object::WlObjectId>,
                    layer: u32,
                    namespace: &'s ::std::ffi::CStr,
                }
                impl crate::interface::ObjectParent for GetLayerSurface<'_> {
                    const CHILD_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl crate::object::HasObjectType for GetLayerSurface<'_> {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerShell;
                }
                impl<'s> crate::interface::Request<'s> for GetLayerSurface<'s> {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::Some(
                        super::super::super::WlObjectType::LayerSurface,
                    );
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .new_id()
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.surface).unwrap(),
                                ),
                            )
                            .maybe_object(
                                self.output.map(|id| storage.get_proxy(id).unwrap()),
                            )
                            .uint(self.layer)
                            .str(self.namespace)
                            .build()
                    }
                }
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerShell;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                #[repr(u32)]
                pub enum Error {
                    ///wl_surface has another role
                    Role = 0u32,
                    ///layer value is invalid
                    InvalidLayer = 1u32,
                    ///wl_surface has a buffer attached or committed
                    AlreadyConstructed = 2u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::Role => "Role",
                                Error::InvalidLayer => "InvalidLayer",
                                Error::AlreadyConstructed => "AlreadyConstructed",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Role,
                                1u32 => Self::InvalidLayer,
                                2u32 => Self::AlreadyConstructed,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
                /**These values indicate which layers a surface can be rendered in. They
        are ordered by z depth, bottom-most first. Traditional shell surfaces
        will typically be rendered between the bottom and top layers.
        Fullscreen shell surfaces are typically rendered at the top layer.
        Multiple surfaces can share a single layer, and ordering within a
        single layer is undefined.*/
                #[repr(u32)]
                pub enum Layer {
                    Background = 0u32,
                    Bottom = 1u32,
                    Top = 2u32,
                    Overlay = 3u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Layer {
                    #[inline]
                    fn clone(&self) -> Layer {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Layer {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Layer::Background => "Background",
                                Layer::Bottom => "Bottom",
                                Layer::Top => "Top",
                                Layer::Overlay => "Overlay",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Layer {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Layer {
                    #[inline]
                    fn eq(&self, other: &Layer) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Layer {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Layer {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Layer {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Layer,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Layer {
                    #[inline]
                    fn cmp(&self, other: &Layer) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Layer {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Layer {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Layer> for u32 {
                    fn from(value: Layer) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Layer")]
                pub struct LayerFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for LayerFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "LayerFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for LayerFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for LayerFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Layer", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Layer {
                    type Error = LayerFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::Background,
                                1u32 => Self::Bottom,
                                2u32 => Self::Top,
                                3u32 => Self::Overlay,
                                _ => return Err(LayerFromU32Error(value)),
                            },
                        )
                    }
                }
            }
        }
        pub mod _layer_surface {
            pub mod request {
                pub struct SetSize {
                    width: u32,
                    height: u32,
                }
                impl crate::object::HasObjectType for SetSize {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetSize {
                    const CODE: crate::sys::wire::OpCode = 0;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.width)
                            .uint(self.height)
                            .build()
                    }
                }
                pub struct SetAnchor {
                    anchor: u32,
                }
                impl crate::object::HasObjectType for SetAnchor {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetAnchor {
                    const CODE: crate::sys::wire::OpCode = 1;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.anchor)
                            .build()
                    }
                }
                pub struct SetExclusiveZone {
                    zone: i32,
                }
                impl crate::object::HasObjectType for SetExclusiveZone {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetExclusiveZone {
                    const CODE: crate::sys::wire::OpCode = 2;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.zone)
                            .build()
                    }
                }
                pub struct SetMargin {
                    top: i32,
                    right: i32,
                    bottom: i32,
                    left: i32,
                }
                impl crate::object::HasObjectType for SetMargin {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetMargin {
                    const CODE: crate::sys::wire::OpCode = 3;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .int(self.top)
                            .int(self.right)
                            .int(self.bottom)
                            .int(self.left)
                            .build()
                    }
                }
                pub struct SetKeyboardInteractivity {
                    keyboard_interactivity: u32,
                }
                impl crate::object::HasObjectType for SetKeyboardInteractivity {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetKeyboardInteractivity {
                    const CODE: crate::sys::wire::OpCode = 4;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.keyboard_interactivity)
                            .build()
                    }
                }
                pub struct GetPopup {
                    popup: crate::object::WlObjectId,
                }
                impl crate::object::HasObjectType for GetPopup {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for GetPopup {
                    const CODE: crate::sys::wire::OpCode = 5;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .maybe_object(
                                ::std::option::Option::Some(
                                    storage.get_proxy(self.popup).unwrap(),
                                ),
                            )
                            .build()
                    }
                }
                pub struct AckConfigure {
                    serial: u32,
                }
                impl crate::object::HasObjectType for AckConfigure {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for AckConfigure {
                    const CODE: crate::sys::wire::OpCode = 6;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.serial)
                            .build()
                    }
                }
                pub struct Destroy;
                impl crate::object::HasObjectType for Destroy {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for Destroy {
                    const CODE: crate::sys::wire::OpCode = 7;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .build()
                    }
                }
                pub struct SetLayer {
                    layer: u32,
                }
                impl crate::object::HasObjectType for SetLayer {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetLayer {
                    const CODE: crate::sys::wire::OpCode = 8;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.layer)
                            .build()
                    }
                }
                pub struct SetExclusiveEdge {
                    edge: u32,
                }
                impl crate::object::HasObjectType for SetExclusiveEdge {
                    const OBJECT_TYPE: super::super::super::WlObjectType = super::super::super::WlObjectType::LayerSurface;
                }
                impl<'s> crate::interface::Request<'s> for SetExclusiveEdge {
                    const CODE: crate::sys::wire::OpCode = 9;
                    const OUTGOING_INTERFACE: ::std::option::Option<
                        super::super::super::WlObjectType,
                    > = ::std::option::Option::None;
                    fn build_message<'m, S: crate::sys::object::dispatch::State>(
                        self,
                        buf: &'m mut impl crate::sys::wire::MessageBuffer,
                        #[allow(dead_code)]
                        storage: &'m crate::sys::object_storage::WlObjectStorage<'_, S>,
                    ) -> crate::sys::wire::WlMessage<'m>
                    where
                        's: 'm,
                    {
                        crate::sys::wire::WlMessage::builder(buf)
                            .opcode(Self::CODE)
                            .uint(self.edge)
                            .build()
                    }
                }
            }
            pub mod event {}
            pub mod wl_enum {
                /**Types of keyboard interaction possible for layer shell surfaces. The
        rationale for this is twofold: (1) some applications are not interested
        in keyboard events and not allowing them to be focused can improve the
        desktop experience; (2) some applications will want to take exclusive
        keyboard focus.*/
                #[repr(u32)]
                pub enum KeyboardInteractivity {
                    ///no keyboard focus is possible
                    None = 0u32,
                    ///request exclusive keyboard focus
                    Exclusive = 1u32,
                    ///request regular keyboard focus semantics
                    OnDemand = 2u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for KeyboardInteractivity {
                    #[inline]
                    fn clone(&self) -> KeyboardInteractivity {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for KeyboardInteractivity {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                KeyboardInteractivity::None => "None",
                                KeyboardInteractivity::Exclusive => "Exclusive",
                                KeyboardInteractivity::OnDemand => "OnDemand",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for KeyboardInteractivity {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for KeyboardInteractivity {
                    #[inline]
                    fn eq(&self, other: &KeyboardInteractivity) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for KeyboardInteractivity {}
                #[automatically_derived]
                impl ::core::cmp::Eq for KeyboardInteractivity {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for KeyboardInteractivity {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &KeyboardInteractivity,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for KeyboardInteractivity {
                    #[inline]
                    fn cmp(
                        &self,
                        other: &KeyboardInteractivity,
                    ) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for KeyboardInteractivity {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl KeyboardInteractivity {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<KeyboardInteractivity> for u32 {
                    fn from(value: KeyboardInteractivity) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to KeyboardInteractivity")]
                pub struct KeyboardInteractivityFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for KeyboardInteractivityFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "KeyboardInteractivityFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error
                for KeyboardInteractivityFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for KeyboardInteractivityFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!(
                                            "failed to convert {0} to KeyboardInteractivity",
                                            __display0,
                                        ),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for KeyboardInteractivity {
                    type Error = KeyboardInteractivityFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::None,
                                1u32 => Self::Exclusive,
                                2u32 => Self::OnDemand,
                                _ => return Err(KeyboardInteractivityFromU32Error(value)),
                            },
                        )
                    }
                }
                #[repr(u32)]
                pub enum Error {
                    ///provided surface state is invalid
                    InvalidSurfaceState = 0u32,
                    ///size is invalid
                    InvalidSize = 1u32,
                    ///anchor bitfield is invalid
                    InvalidAnchor = 2u32,
                    ///keyboard interactivity is invalid
                    InvalidKeyboardInteractivity = 3u32,
                    ///exclusive edge is invalid given the surface anchors
                    InvalidExclusiveEdge = 4u32,
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Error {
                    #[inline]
                    fn clone(&self) -> Error {
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::fmt::Debug for Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::write_str(
                            f,
                            match self {
                                Error::InvalidSurfaceState => "InvalidSurfaceState",
                                Error::InvalidSize => "InvalidSize",
                                Error::InvalidAnchor => "InvalidAnchor",
                                Error::InvalidKeyboardInteractivity => {
                                    "InvalidKeyboardInteractivity"
                                }
                                Error::InvalidExclusiveEdge => "InvalidExclusiveEdge",
                            },
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Error {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Error {
                    #[inline]
                    fn eq(&self, other: &Error) -> bool {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        __self_discr == __arg1_discr
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Error {}
                #[automatically_derived]
                impl ::core::cmp::Eq for Error {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {}
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Error {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Error,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Error {
                    #[inline]
                    fn cmp(&self, other: &Error) -> ::core::cmp::Ordering {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                        ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
                    }
                }
                #[automatically_derived]
                impl ::core::hash::Hash for Error {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        let __self_discr = ::core::intrinsics::discriminant_value(self);
                        ::core::hash::Hash::hash(&__self_discr, state)
                    }
                }
                impl Error {
                    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                        unsafe { ::std::mem::transmute::<u32, Self>(value) }
                    }
                }
                impl ::std::convert::From<Error> for u32 {
                    fn from(value: Error) -> Self {
                        value as u32
                    }
                }
                #[error("failed to convert {0} to Error")]
                pub struct ErrorFromU32Error(pub u32);
                #[automatically_derived]
                impl ::core::fmt::Debug for ErrorFromU32Error {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ErrorFromU32Error",
                            &&self.0,
                        )
                    }
                }
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::thiserror::__private::Error for ErrorFromU32Error {}
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::fmt::Display for ErrorFromU32Error {
                    #[allow(clippy::used_underscore_binding)]
                    fn fmt(
                        &self,
                        __formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        use ::thiserror::__private::AsDisplay as _;
                        #[allow(unused_variables, deprecated)]
                        let Self(_0) = self;
                        match (_0.as_display(),) {
                            (__display0,) => {
                                __formatter
                                    .write_fmt(
                                        format_args!("failed to convert {0} to Error", __display0),
                                    )
                            }
                        }
                    }
                }
                impl ::std::convert::TryFrom<u32> for Error {
                    type Error = ErrorFromU32Error;
                    fn try_from(value: u32) -> ::std::result::Result<Self, Self::Error> {
                        Ok(
                            match value {
                                0u32 => Self::InvalidSurfaceState,
                                1u32 => Self::InvalidSize,
                                2u32 => Self::InvalidAnchor,
                                3u32 => Self::InvalidKeyboardInteractivity,
                                4u32 => Self::InvalidExclusiveEdge,
                                _ => return Err(ErrorFromU32Error(value)),
                            },
                        )
                    }
                }
                pub struct Anchor(
                    <Anchor as ::bitflags::__private::PublicFlags>::Internal,
                );
                #[automatically_derived]
                impl ::core::fmt::Debug for Anchor {
                    #[inline]
                    fn fmt(
                        &self,
                        f: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Anchor",
                            &&self.0,
                        )
                    }
                }
                #[automatically_derived]
                impl ::core::clone::Clone for Anchor {
                    #[inline]
                    fn clone(&self) -> Anchor {
                        let _: ::core::clone::AssertParamIsClone<
                            <Anchor as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                        *self
                    }
                }
                #[automatically_derived]
                impl ::core::marker::Copy for Anchor {}
                #[automatically_derived]
                impl ::core::hash::Hash for Anchor {
                    #[inline]
                    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                        ::core::hash::Hash::hash(&self.0, state)
                    }
                }
                #[automatically_derived]
                impl ::core::marker::StructuralPartialEq for Anchor {}
                #[automatically_derived]
                impl ::core::cmp::PartialEq for Anchor {
                    #[inline]
                    fn eq(&self, other: &Anchor) -> bool {
                        self.0 == other.0
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Eq for Anchor {
                    #[inline]
                    #[doc(hidden)]
                    #[coverage(off)]
                    fn assert_receiver_is_total_eq(&self) -> () {
                        let _: ::core::cmp::AssertParamIsEq<
                            <Anchor as ::bitflags::__private::PublicFlags>::Internal,
                        >;
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::PartialOrd for Anchor {
                    #[inline]
                    fn partial_cmp(
                        &self,
                        other: &Anchor,
                    ) -> ::core::option::Option<::core::cmp::Ordering> {
                        ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                    }
                }
                #[automatically_derived]
                impl ::core::cmp::Ord for Anchor {
                    #[inline]
                    fn cmp(&self, other: &Anchor) -> ::core::cmp::Ordering {
                        ::core::cmp::Ord::cmp(&self.0, &other.0)
                    }
                }
                impl Anchor {
                    ///the top edge of the anchor rectangle
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const TOP: Self = Self::from_bits_retain(1);
                    ///the bottom edge of the anchor rectangle
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const BOTTOM: Self = Self::from_bits_retain(2);
                    ///the left edge of the anchor rectangle
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const LEFT: Self = Self::from_bits_retain(4);
                    ///the right edge of the anchor rectangle
                    #[allow(deprecated, non_upper_case_globals)]
                    pub const RIGHT: Self = Self::from_bits_retain(8);
                }
                impl ::bitflags::Flags for Anchor {
                    const FLAGS: &'static [::bitflags::Flag<Anchor>] = &[
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("TOP", Anchor::TOP)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("BOTTOM", Anchor::BOTTOM)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("LEFT", Anchor::LEFT)
                        },
                        {
                            #[allow(deprecated, non_upper_case_globals)]
                            ::bitflags::Flag::new("RIGHT", Anchor::RIGHT)
                        },
                    ];
                    type Bits = u32;
                    fn bits(&self) -> u32 {
                        Anchor::bits(self)
                    }
                    fn from_bits_retain(bits: u32) -> Anchor {
                        Anchor::from_bits_retain(bits)
                    }
                }
                #[allow(
                    dead_code,
                    deprecated,
                    unused_doc_comments,
                    unused_attributes,
                    unused_mut,
                    unused_imports,
                    non_upper_case_globals,
                    clippy::assign_op_pattern,
                    clippy::indexing_slicing,
                    clippy::same_name_method,
                    clippy::iter_without_into_iter,
                )]
                const _: () = {
                    #[repr(transparent)]
                    pub struct InternalBitFlags(u32);
                    #[automatically_derived]
                    impl ::core::clone::Clone for InternalBitFlags {
                        #[inline]
                        fn clone(&self) -> InternalBitFlags {
                            let _: ::core::clone::AssertParamIsClone<u32>;
                            *self
                        }
                    }
                    #[automatically_derived]
                    impl ::core::marker::Copy for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::marker::StructuralPartialEq for InternalBitFlags {}
                    #[automatically_derived]
                    impl ::core::cmp::PartialEq for InternalBitFlags {
                        #[inline]
                        fn eq(&self, other: &InternalBitFlags) -> bool {
                            self.0 == other.0
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Eq for InternalBitFlags {
                        #[inline]
                        #[doc(hidden)]
                        #[coverage(off)]
                        fn assert_receiver_is_total_eq(&self) -> () {
                            let _: ::core::cmp::AssertParamIsEq<u32>;
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::PartialOrd for InternalBitFlags {
                        #[inline]
                        fn partial_cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::option::Option<::core::cmp::Ordering> {
                            ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::cmp::Ord for InternalBitFlags {
                        #[inline]
                        fn cmp(
                            &self,
                            other: &InternalBitFlags,
                        ) -> ::core::cmp::Ordering {
                            ::core::cmp::Ord::cmp(&self.0, &other.0)
                        }
                    }
                    #[automatically_derived]
                    impl ::core::hash::Hash for InternalBitFlags {
                        #[inline]
                        fn hash<__H: ::core::hash::Hasher>(
                            &self,
                            state: &mut __H,
                        ) -> () {
                            ::core::hash::Hash::hash(&self.0, state)
                        }
                    }
                    impl ::bitflags::__private::PublicFlags for Anchor {
                        type Primitive = u32;
                        type Internal = InternalBitFlags;
                    }
                    impl ::bitflags::__private::core::default::Default
                    for InternalBitFlags {
                        #[inline]
                        fn default() -> Self {
                            InternalBitFlags::empty()
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Debug for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            if self.is_empty() {
                                f.write_fmt(
                                    format_args!("{0:#x}", <u32 as ::bitflags::Bits>::EMPTY),
                                )
                            } else {
                                ::bitflags::__private::core::fmt::Display::fmt(self, f)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Display for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter<'_>,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            ::bitflags::parser::to_writer(&Anchor(*self), f)
                        }
                    }
                    impl ::bitflags::__private::core::str::FromStr for InternalBitFlags {
                        type Err = ::bitflags::parser::ParseError;
                        fn from_str(
                            s: &str,
                        ) -> ::bitflags::__private::core::result::Result<
                            Self,
                            Self::Err,
                        > {
                            ::bitflags::parser::from_str::<Anchor>(s)
                                .map(|flags| flags.0)
                        }
                    }
                    impl ::bitflags::__private::core::convert::AsRef<u32>
                    for InternalBitFlags {
                        fn as_ref(&self) -> &u32 {
                            &self.0
                        }
                    }
                    impl ::bitflags::__private::core::convert::From<u32>
                    for InternalBitFlags {
                        fn from(bits: u32) -> Self {
                            Self::from_bits_retain(bits)
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl InternalBitFlags {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(<u32 as ::bitflags::Bits>::EMPTY) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            {
                                let mut truncated = <u32 as ::bitflags::Bits>::EMPTY;
                                let mut i = 0;
                                {
                                    {
                                        let flag = <Anchor as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Anchor as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Anchor as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                {
                                    {
                                        let flag = <Anchor as ::bitflags::Flags>::FLAGS[i]
                                            .value()
                                            .bits();
                                        truncated = truncated | flag;
                                        i += 1;
                                    }
                                };
                                let _ = i;
                                Self::from_bits_retain(truncated)
                            }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0 }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                let truncated = Self::from_bits_truncate(bits).0;
                                if truncated == bits {
                                    ::bitflags::__private::core::option::Option::Some(
                                        Self(bits),
                                    )
                                } else {
                                    ::bitflags::__private::core::option::Option::None
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits & Self::all().bits()) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(bits) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                {
                                    if name == "TOP" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Anchor::TOP.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "BOTTOM" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Anchor::BOTTOM.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "LEFT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Anchor::LEFT.bits()),
                                        );
                                    }
                                };
                                {
                                    if name == "RIGHT" {
                                        return ::bitflags::__private::core::option::Option::Some(
                                            Self(Anchor::RIGHT.bits()),
                                        );
                                    }
                                };
                                let _ = name;
                                ::bitflags::__private::core::option::Option::None
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.bits() == <u32 as ::bitflags::Bits>::EMPTY }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { Self::all().bits() | f.bits() == f.bits() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            {
                                f.bits() & other.bits() != <u32 as ::bitflags::Bits>::EMPTY
                            }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.bits() & other.bits() == other.bits() }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).union(other);
                            }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits()).difference(other);
                            }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            {
                                *f = Self::from_bits_retain(f.bits())
                                    .symmetric_difference(other);
                            }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            {
                                if value {
                                    f.insert(other);
                                } else {
                                    f.remove(other);
                                }
                            }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & other.bits()) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() | other.bits()) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() & !other.bits()) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self::from_bits_retain(f.bits() ^ other.bits()) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self::from_bits_truncate(!f.bits()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex
                    for InternalBitFlags {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: InternalBitFlags) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign
                    for InternalBitFlags {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign
                    for InternalBitFlags {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for InternalBitFlags {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign
                    for InternalBitFlags {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for InternalBitFlags {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<InternalBitFlags>
                    for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<
                        InternalBitFlags,
                    > for InternalBitFlags {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl InternalBitFlags {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Anchor> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Anchor as ::bitflags::Flags>::FLAGS,
                                Anchor::from_bits_retain(self.bits()),
                                Anchor::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Anchor> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Anchor as ::bitflags::Flags>::FLAGS,
                                Anchor::from_bits_retain(self.bits()),
                                Anchor::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator
                    for InternalBitFlags {
                        type Item = Anchor;
                        type IntoIter = ::bitflags::iter::Iter<Anchor>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                    impl InternalBitFlags {
                        /// Returns a mutable reference to the raw value of the flags currently stored.
                        #[inline]
                        pub fn bits_mut(&mut self) -> &mut u32 {
                            &mut self.0
                        }
                    }
                    #[allow(dead_code, deprecated, unused_attributes)]
                    impl Anchor {
                        /// Get a flags value with all bits unset.
                        #[inline]
                        pub const fn empty() -> Self {
                            { Self(InternalBitFlags::empty()) }
                        }
                        /// Get a flags value with all known bits set.
                        #[inline]
                        pub const fn all() -> Self {
                            { Self(InternalBitFlags::all()) }
                        }
                        /// Get the underlying bits value.
                        ///
                        /// The returned value is exactly the bits set in this flags value.
                        #[inline]
                        pub const fn bits(&self) -> u32 {
                            let f = self;
                            { f.0.bits() }
                        }
                        /// Convert from a bits value.
                        ///
                        /// This method will return `None` if any unknown bits are set.
                        #[inline]
                        pub const fn from_bits(
                            bits: u32,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let bits = bits;
                            {
                                match InternalBitFlags::from_bits(bits) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Convert from a bits value, unsetting any unknown bits.
                        #[inline]
                        pub const fn from_bits_truncate(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_truncate(bits)) }
                        }
                        /// Convert from a bits value exactly.
                        #[inline]
                        pub const fn from_bits_retain(bits: u32) -> Self {
                            let bits = bits;
                            { Self(InternalBitFlags::from_bits_retain(bits)) }
                        }
                        /// Get a flags value with the bits of a flag with the given name set.
                        ///
                        /// This method will return `None` if `name` is empty or doesn't
                        /// correspond to any named flag.
                        #[inline]
                        pub fn from_name(
                            name: &str,
                        ) -> ::bitflags::__private::core::option::Option<Self> {
                            let name = name;
                            {
                                match InternalBitFlags::from_name(name) {
                                    ::bitflags::__private::core::option::Option::Some(bits) => {
                                        ::bitflags::__private::core::option::Option::Some(
                                            Self(bits),
                                        )
                                    }
                                    ::bitflags::__private::core::option::Option::None => {
                                        ::bitflags::__private::core::option::Option::None
                                    }
                                }
                            }
                        }
                        /// Whether all bits in this flags value are unset.
                        #[inline]
                        pub const fn is_empty(&self) -> bool {
                            let f = self;
                            { f.0.is_empty() }
                        }
                        /// Whether all known bits in this flags value are set.
                        #[inline]
                        pub const fn is_all(&self) -> bool {
                            let f = self;
                            { f.0.is_all() }
                        }
                        /// Whether any set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn intersects(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.intersects(other.0) }
                        }
                        /// Whether all set bits in a source flags value are also set in a target flags value.
                        #[inline]
                        pub const fn contains(&self, other: Self) -> bool {
                            let f = self;
                            let other = other;
                            { f.0.contains(other.0) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        pub fn insert(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.insert(other.0) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `remove` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        pub fn remove(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.remove(other.0) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        pub fn toggle(&mut self, other: Self) {
                            let f = self;
                            let other = other;
                            { f.0.toggle(other.0) }
                        }
                        /// Call `insert` when `value` is `true` or `remove` when `value` is `false`.
                        #[inline]
                        pub fn set(&mut self, other: Self, value: bool) {
                            let f = self;
                            let other = other;
                            let value = value;
                            { f.0.set(other.0, value) }
                        }
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn intersection(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.intersection(other.0)) }
                        }
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn union(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.union(other.0)) }
                        }
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        #[must_use]
                        pub const fn difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.difference(other.0)) }
                        }
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        #[must_use]
                        pub const fn symmetric_difference(self, other: Self) -> Self {
                            let f = self;
                            let other = other;
                            { Self(f.0.symmetric_difference(other.0)) }
                        }
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        #[must_use]
                        pub const fn complement(self) -> Self {
                            let f = self;
                            { Self(f.0.complement()) }
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Binary for Anchor {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Binary::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::Octal for Anchor {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::Octal::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::LowerHex for Anchor {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::LowerHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::fmt::UpperHex for Anchor {
                        fn fmt(
                            &self,
                            f: &mut ::bitflags::__private::core::fmt::Formatter,
                        ) -> ::bitflags::__private::core::fmt::Result {
                            let inner = self.0;
                            ::bitflags::__private::core::fmt::UpperHex::fmt(&inner, f)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOr for Anchor {
                        type Output = Self;
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor(self, other: Anchor) -> Self {
                            self.union(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitOrAssign for Anchor {
                        /// The bitwise or (`|`) of the bits in two flags values.
                        #[inline]
                        fn bitor_assign(&mut self, other: Self) {
                            self.insert(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXor for Anchor {
                        type Output = Self;
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor(self, other: Self) -> Self {
                            self.symmetric_difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitXorAssign for Anchor {
                        /// The bitwise exclusive-or (`^`) of the bits in two flags values.
                        #[inline]
                        fn bitxor_assign(&mut self, other: Self) {
                            self.toggle(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAnd for Anchor {
                        type Output = Self;
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand(self, other: Self) -> Self {
                            self.intersection(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::BitAndAssign for Anchor {
                        /// The bitwise and (`&`) of the bits in two flags values.
                        #[inline]
                        fn bitand_assign(&mut self, other: Self) {
                            *self = Self::from_bits_retain(self.bits())
                                .intersection(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Sub for Anchor {
                        type Output = Self;
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub(self, other: Self) -> Self {
                            self.difference(other)
                        }
                    }
                    impl ::bitflags::__private::core::ops::SubAssign for Anchor {
                        /// The intersection of a source flags value with the complement of a target flags value (`&!`).
                        ///
                        /// This method is not equivalent to `self & !other` when `other` has unknown bits set.
                        /// `difference` won't truncate `other`, but the `!` operator will.
                        #[inline]
                        fn sub_assign(&mut self, other: Self) {
                            self.remove(other);
                        }
                    }
                    impl ::bitflags::__private::core::ops::Not for Anchor {
                        type Output = Self;
                        /// The bitwise negation (`!`) of the bits in a flags value, truncating the result.
                        #[inline]
                        fn not(self) -> Self {
                            self.complement()
                        }
                    }
                    impl ::bitflags::__private::core::iter::Extend<Anchor> for Anchor {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn extend<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(&mut self, iterator: T) {
                            for item in iterator {
                                self.insert(item)
                            }
                        }
                    }
                    impl ::bitflags::__private::core::iter::FromIterator<Anchor>
                    for Anchor {
                        /// The bitwise or (`|`) of the bits in each flags value.
                        fn from_iter<
                            T: ::bitflags::__private::core::iter::IntoIterator<
                                    Item = Self,
                                >,
                        >(iterator: T) -> Self {
                            use ::bitflags::__private::core::iter::Extend;
                            let mut result = Self::empty();
                            result.extend(iterator);
                            result
                        }
                    }
                    impl Anchor {
                        /// Yield a set of contained flags values.
                        ///
                        /// Each yielded flags value will correspond to a defined named flag. Any unknown bits
                        /// will be yielded together as a final flags value.
                        #[inline]
                        pub const fn iter(&self) -> ::bitflags::iter::Iter<Anchor> {
                            ::bitflags::iter::Iter::__private_const_new(
                                <Anchor as ::bitflags::Flags>::FLAGS,
                                Anchor::from_bits_retain(self.bits()),
                                Anchor::from_bits_retain(self.bits()),
                            )
                        }
                        /// Yield a set of contained named flags values.
                        ///
                        /// This method is like [`iter`](#method.iter), except only yields bits in contained named flags.
                        /// Any unknown bits, or bits not corresponding to a contained flag will not be yielded.
                        #[inline]
                        pub const fn iter_names(
                            &self,
                        ) -> ::bitflags::iter::IterNames<Anchor> {
                            ::bitflags::iter::IterNames::__private_const_new(
                                <Anchor as ::bitflags::Flags>::FLAGS,
                                Anchor::from_bits_retain(self.bits()),
                                Anchor::from_bits_retain(self.bits()),
                            )
                        }
                    }
                    impl ::bitflags::__private::core::iter::IntoIterator for Anchor {
                        type Item = Anchor;
                        type IntoIter = ::bitflags::iter::Iter<Anchor>;
                        fn into_iter(self) -> Self::IntoIter {
                            self.iter()
                        }
                    }
                };
            }
        }
    }
}
