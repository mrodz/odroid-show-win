use std::alloc::{Layout, alloc};

use libusb1_sys::{
    libusb_device, libusb_device_descriptor, libusb_device_handle, libusb_get_device_descriptor,
    libusb_get_device_list, libusb_get_string_descriptor_ascii, libusb_init, libusb_open,
};

use super::{ffi_ptr_const, ffi_ptr_mut, Device};

type LibusbDevice = *mut libusb_device;
const STRING_BUF_LEN: usize = 255;

pub(super) fn init() -> Result<(), i32> {
    let code = unsafe { libusb_init(std::ptr::null_mut()) };
    if code < 0 {
        Err(code)
    } else {
        Ok(())
    }
}

fn get_libusb_devices() -> (*const LibusbDevice, isize) {
    let mut devices = ffi_ptr_const::<*mut libusb_device>();
    let devices_size: isize = unsafe { libusb_get_device_list(std::ptr::null_mut(), &mut devices) };

    (devices, devices_size)
}

fn get_descriptor(device: LibusbDevice) -> Result<*mut libusb_device_descriptor, i32> {
    let descriptor_layout = Layout::new::<libusb_device_descriptor>();
    let descriptor = unsafe { alloc(descriptor_layout) } as *mut libusb_device_descriptor;

    let code = unsafe { libusb_get_device_descriptor(device, descriptor) };

    if code < 0 {
        Err(code)
    } else {
        let result = Ok(descriptor);
        result
    }
}

fn get_handle(
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

fn open_usb(
    device: LibusbDevice,
) -> Result<(*mut libusb_device_descriptor, *mut libusb_device_handle), i32> {
    let descriptor = get_descriptor(device)?;
    let handle = get_handle(device, descriptor)?;

    Ok((descriptor, handle))
}

pub fn descriptor_to_string_check(handle: *mut libusb_device_handle, descriptor_field: u8) -> Option<String> {
	if descriptor_field == 0 {
		return None
	}

	if let Ok(result) = descriptor_to_string(handle, descriptor_field) {
		Some(result)
	} else {
		None
	}
}

pub fn descriptor_to_string(
    handle: *mut libusb_device_handle,
    descriptor_field: u8,
) -> Result<String, i32> {
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

pub fn get_devices() -> Result<(Vec<Device>, *const *mut libusb_device), i32> {
    let (devices, devices_size) = get_libusb_devices();

    let mut result: Vec<Device> = Vec::with_capacity(devices_size.try_into().unwrap());

    for i in 0..devices_size {
        let device: LibusbDevice = unsafe { *devices.offset(i) };

        let Ok((descriptor, handle)) = open_usb(device) else {
			continue;
		};

        result.push(Device::new(descriptor, handle, device));
    }

    Ok((result, devices))
}
