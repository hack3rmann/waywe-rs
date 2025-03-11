pub mod init;
pub mod interface;
pub mod object;
pub mod sys;

wayland_scanner::include_wl_interfaces!("wayland-protocols/wayland.xml");

pub mod proto {
    use wayland_sys::{Interface, InterfaceMessage, InterfaceSemiFfi, wl_interface, wl_message};

    pub const WL_DISPLAY: Interface<'static> = Interface {
        name: c"wl_display",
        version: 1,
        methods: &[
            InterfaceMessage {
                name: c"sync",
                signature: c"n",
                outgoing_interfaces: &[&WL_CALLBACK],
            },
            InterfaceMessage {
                name: c"get_registry",
                signature: c"n",
                outgoing_interfaces: &[&WL_REGISTRY],
            },
        ],
        events: &[
            InterfaceMessage {
                name: c"error",
                signature: c"ous",
                outgoing_interfaces: &[],
            },
            InterfaceMessage {
                name: c"delete_id",
                signature: c"u",
                outgoing_interfaces: &[],
            },
        ],
    };

    pub const WL_DISPLAY_MESSAGES: InterfaceSemiFfi<'static> = InterfaceSemiFfi {
        methods: &[wl_message {
            name: WL_DISPLAY.methods[0].name.as_ptr(),
            signature: WL_DISPLAY.methods[0].signature.as_ptr(),
            types: {
                const REF: &[&wl_interface] = &[&WL_CALLBACK_INTERFACE];
                unsafe {
                    std::mem::transmute::<
                        *const &wayland_sys::wl_interface,
                        *const *const wayland_sys::wl_interface,
                    >(REF.as_ptr())
                }
            },
        }],
        events: &[],
    };

    pub const WL_DISPLAY_INTERFACE: wl_interface = wl_interface {
        name: WL_DISPLAY.name.as_ptr(),
        version: WL_DISPLAY.version as i32,
        method_count: WL_DISPLAY_MESSAGES.methods.len() as i32,
        methods: WL_DISPLAY_MESSAGES.methods.as_ptr(),
        event_count: WL_DISPLAY_MESSAGES.events.len() as i32,
        events: WL_DISPLAY_MESSAGES.events.as_ptr(),
    };

    pub const WL_CALLBACK: Interface<'static> = Interface {
        name: c"wl_callback",
        version: 1,
        methods: &[],
        events: &[],
    };

    pub const WL_CALLBACK_MESSAGES: InterfaceSemiFfi<'static> = InterfaceSemiFfi {
        methods: &[],
        events: &[],
    };

    pub const WL_CALLBACK_INTERFACE: wl_interface = wl_interface {
        name: WL_CALLBACK.name.as_ptr(),
        version: WL_CALLBACK.version as i32,
        method_count: WL_CALLBACK_MESSAGES.methods.len() as i32,
        methods: WL_CALLBACK_MESSAGES.methods.as_ptr(),
        event_count: WL_CALLBACK_MESSAGES.events.len() as i32,
        events: WL_CALLBACK_MESSAGES.events.as_ptr(),
    };

    pub const WL_REGISTRY: Interface<'static> = Interface {
        name: c"wl_registry",
        version: 1,
        methods: &[],
        events: &[],
    };

    pub const WL_REGISTRY_MESSAGES: InterfaceSemiFfi<'static> = InterfaceSemiFfi {
        methods: &[],
        events: &[],
    };
}
