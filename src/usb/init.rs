use super::{ffi_ptr_const, ffi_ptr_mut, Device};
use libusb1_sys::*;
use std::alloc::{alloc, Layout};

/// Alias for convenience.
type LibusbDevice = *mut libusb_device;

/// The maximum length alloted for string description reads.
pub const STRING_BUF_LEN: usize = 255;

/// Wrapper for [`libusb_init`].
/// Returns the error code if this step fails.
pub(super) fn init() -> Result<(), i32> {
    let code = unsafe { libusb_init(std::ptr::null_mut()) };
    if code < 0 {
        Err(code)
    } else {
        Ok(())
    }
}

/// Get devices as a C-style array of type [`LibusbDevice`].
/// Returns said array and its length.
pub(super) fn get_libusb_devices() -> (*const LibusbDevice, isize) {
    let mut devices = ffi_ptr_const::<*mut libusb_device>();
    let devices_size: isize = unsafe { libusb_get_device_list(std::ptr::null_mut(), &mut devices) };

    (devices, devices_size)
}

/// Get the descriptor of a device, or return the error code if it fails.
pub(super) fn get_descriptor(device: LibusbDevice) -> Result<*mut libusb_device_descriptor, i32> {
    // Use allocator to reserve space for the device descriptor.
    let descriptor_layout = Layout::new::<libusb_device_descriptor>();
    // Must be cleaned up in super::USBInterface when dropped.
    let descriptor = unsafe { alloc(descriptor_layout) } as *mut libusb_device_descriptor;

    let code = unsafe { libusb_get_device_descriptor(device, descriptor) };

    if code < 0 {
        Err(code)
    } else {
        let result = Ok(descriptor);
        result
    }
}

pub(super) fn get_handle(
    device: LibusbDevice,
    descriptor: *mut libusb_device_descriptor,
) -> Result<*mut libusb_device_handle, i32> {
    assert!(!descriptor.is_null());

    let mut handle = ffi_ptr_mut::<libusb_device_handle>();
    let code = unsafe { libusb_open(device, &mut handle) };

    if code < 0 {
        Err(code)
    } else {
        Ok(handle)
    }
}

/// Open a USB device.
///
/// Will yield its descriptor and a handle to the device,
/// or an error code if this step fails.
fn open_usb(
    device: LibusbDevice,
) -> Result<(*mut libusb_device_descriptor, *mut libusb_device_handle), i32> {
    let descriptor = get_descriptor(device)?;
    let handle = get_handle(device, descriptor)?;

    Ok((descriptor, handle))
}

/// Stringify a descriptor field.
/// If the field does not exist or the descriptor cannot find the string, returns `None`.
///
/// Simplifies [`descriptor_to_string`]
pub fn descriptor_to_string_check(
    handle: *mut libusb_device_handle,
    descriptor_field: u8,
) -> Option<String> {
    if descriptor_field == 0 {
        return None;
    }

    if let Ok(result) = descriptor_to_string(handle, descriptor_field) {
        Some(result)
    } else {
        None
    }
}

/// Stringify a descriptor field.
///
/// Returns the string, if successful, or an error code.
pub fn descriptor_to_string(
    handle: *mut libusb_device_handle,
    descriptor_field: u8,
) -> Result<String, i32> {
    assert!(descriptor_field != 0);

    let mut raw_bytes = [0_u8; STRING_BUF_LEN];
    let size: i32 = std::mem::size_of_val(&raw_bytes).try_into().unwrap();

    let code = unsafe {
        libusb_get_string_descriptor_ascii(handle, descriptor_field, raw_bytes.as_mut_ptr(), size)
    };

    if code < 0 {
        Err(code)
    } else {
        Ok(unsafe { String::from_utf8_unchecked(raw_bytes.to_vec()) })
    }
}

/// Get opened devices as a vector of [`Device`]s.
///
/// Also returns a pointer to the underlying C-Style array [`LibusbDevice`]s,
/// in order to be cleaned up at a later date.
///
/// Will return an error code if any step fails.
pub fn get_devices(
    devices: *const *mut libusb_device,
    devices_size: isize,
) -> Result<(Vec<Device>, *const *mut libusb_device), i32> {
    let mut result: Vec<Device> = Vec::with_capacity(devices_size.try_into().unwrap());

    for i in 0..devices_size {
        let device: LibusbDevice = unsafe { *devices.offset(i) };

        let Ok((descriptor, handle)) = open_usb(device) else {
			continue // this "device" is useless to us. 
		};

        unsafe { libusb_set_auto_detach_kernel_driver(handle, 1) };

        result.push(Device::new(descriptor, handle, device));
    }

    Ok((result, devices))
}
