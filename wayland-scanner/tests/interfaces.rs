use std::{ffi::CStr, slice};
use tracing::debug;
use wayland_sys::{count_arguments_from_message_signature, wl_interface, wl_message};

mod protocols {
    wayland_scanner::include_wl_interfaces!("wayland-protocols/wayland.xml");
}

unsafe fn check_wl_messages_arrays_are_the_same(
    lhs_ptr: *const wl_message,
    lhs_len: i32,
    rhs_ptr: *const wl_message,
    rhs_len: i32,
) {
    assert!(lhs_len >= 0 && rhs_len == lhs_len);

    if !lhs_ptr.is_null() && !rhs_ptr.is_null() {
        let left_methods = unsafe { slice::from_raw_parts(lhs_ptr, lhs_len as usize) };
        let right_methods = unsafe { slice::from_raw_parts(rhs_ptr, rhs_len as usize) };

        for (left, right) in left_methods.iter().zip(right_methods) {
            unsafe { check_wl_messages_are_the_same(left, right) };
        }
    }
}

unsafe fn check_wl_messages_are_the_same(lhs: &wl_message, rhs: &wl_message) {
    assert!(!lhs.name.is_null() && !rhs.name.is_null());

    let left_name = unsafe { CStr::from_ptr(lhs.name) };
    let right_name = unsafe { CStr::from_ptr(rhs.name) };

    assert_eq!(left_name, right_name, "same names");

    if lhs.signature.is_null() && rhs.signature.is_null() {
        return;
    }

    assert!(
        !lhs.signature.is_null() && !rhs.signature.is_null(),
        "both signatures are present in the {left_name:?} message",
    );

    let left_signature = unsafe { CStr::from_ptr(lhs.signature) };
    let right_signature = unsafe { CStr::from_ptr(rhs.signature) };

    assert_eq!(
        left_signature, right_signature,
        "same signatures for the {left_name:?} message",
    );

    let signature = left_signature;

    let n_args = count_arguments_from_message_signature(signature);

    if lhs.types.is_null() && rhs.types.is_null() {
        return;
    }

    assert!(
        !lhs.types.is_null() && !rhs.types.is_null(),
        "both type arrays are present for the {left_name:?} message, left is {:?}, right is {:?}",
        lhs.types,
        rhs.types,
    );

    let left_interfaces = unsafe { slice::from_raw_parts(lhs.types, n_args) };
    let right_interfaces = unsafe { slice::from_raw_parts(rhs.types, n_args) };

    for (&left, &right) in left_interfaces.iter().zip(right_interfaces) {
        if left.is_null() && right.is_null() {
            continue;
        }

        let left = unsafe { left.as_ref().expect("left interface is null") };
        let right = unsafe { right.as_ref().expect("right interface is null") };

        unsafe { check_wl_interfaces_are_the_same(left, right, CheckDepth::Shallow) };
    }
}

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
enum CheckDepth {
    Shallow,
    #[default]
    Deep,
}

unsafe fn check_wl_interfaces_are_the_same(
    lhs: &wl_interface,
    rhs: &wl_interface,
    depth: CheckDepth,
) {
    assert!(
        !lhs.name.is_null() && !rhs.name.is_null(),
        "both names exist"
    );

    let left_name = unsafe { CStr::from_ptr(lhs.name) };
    let right_name = unsafe { CStr::from_ptr(rhs.name) };

    assert_eq!(left_name, right_name, "same names");
    assert_eq!(lhs.version, rhs.version, "same versions");

    if let CheckDepth::Deep = depth {
        debug!(?left_name, "checking requests are the same");

        unsafe {
            check_wl_messages_arrays_are_the_same(
                lhs.methods,
                lhs.method_count,
                rhs.methods,
                rhs.method_count,
            )
        };

        debug!(?left_name, "checking events are the same");

        unsafe {
            check_wl_messages_arrays_are_the_same(
                lhs.events,
                lhs.event_count,
                rhs.events,
                rhs.event_count,
            )
        };
    }
}

macro_rules! define_interface_tests {
    ( $( $interface:ident ),* $(,)? ) => {
        $(
            ::paste::paste! {
                #[test]
                fn [< $interface _the_same >] () {
                    _ = tracing_subscriber::fmt::try_init();

                    let external = unsafe { &::wayland_sys:: [< $interface _interface >] };
                    let internal = & $crate ::protocols:: $interface ::WL_INTERFACE;

                    unsafe { check_wl_interfaces_are_the_same(external, internal, CheckDepth::Deep) };
                }
            }
        )*
    };
}

define_interface_tests!(
    wl_display,
    wl_registry,
    wl_callback,
    wl_compositor,
    wl_shm_pool,
    wl_shm,
    wl_buffer,
    wl_data_offer,
    wl_data_source,
    wl_data_device,
    wl_data_device_manager,
    wl_shell,
    wl_shell_surface,
    wl_surface,
    wl_seat,
    wl_pointer,
    wl_keyboard,
    wl_touch,
    wl_output,
    wl_region,
    wl_subcompositor,
    wl_subsurface,
);
