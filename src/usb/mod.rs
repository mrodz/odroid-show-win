mod connect;
mod init;

extern crate libc;
extern crate libusb1_sys as ffi;

use std::{
    alloc::{dealloc, Layout},
    fmt::{Debug, Display},
};

use ffi::*;

fn ffi_ptr_mut<T>() -> *mut T {
    std::ptr::null_mut() as *mut T
}

fn ffi_ptr_const<T>() -> *const T {
    std::ptr::null_mut() as *const T
}

#[derive(Clone, Debug)]
pub struct Device {
    libusb_device: *mut libusb_device, // do not access after `libusb_free_device_list`
    descriptor: *mut libusb_device_descriptor, // must be freed via `dealloc`
    handle: *mut libusb_device_handle, // must be closed via `libusb_close`
    closed: bool,
}

impl Device {
    pub fn new(
        descriptor: *mut libusb_device_descriptor,
        handle: *mut libusb_device_handle,
        device_ptr: *mut libusb_device,
    ) -> Self {
        Self {
            descriptor: descriptor,
            handle: handle,
            libusb_device: device_ptr,
            closed: false,
        }
    }

    pub fn can_use(&self) -> bool {
        return !self.closed;
    }

    pub fn close(&mut self) {
        self.closed = true;
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use init::descriptor_to_string_check;

		if !self.can_use() {
			return write!(f, "Inaccessible USB device (was it dropped?)")
		}

        let product =
            descriptor_to_string_check(self.handle, unsafe { (*self.descriptor).iProduct })
                .unwrap_or("no product name".into());
        let manufacturer =
            descriptor_to_string_check(self.handle, unsafe { (*self.descriptor).iManufacturer })
                .unwrap_or("no manufacturer name".into());
        let serial =
            descriptor_to_string_check(self.handle, unsafe { (*self.descriptor).iSerialNumber })
                .unwrap_or("none".into());

        write!(
            f,
            "libusb-compatible device\n\
		        - product: {product}\n\
                - manufacturer: {manufacturer}\n\
				- serial number: {serial}\n\
			"
        )
    }
}

pub struct USBInterface {
    initialized: bool,
    devices: Vec<Device>,
    devices_ptr: *const *mut libusb_device, // must be freed via `libusb_free_device_list`
}

impl USBInterface {
    pub fn new() -> Result<Self, i32> {
        init::init()?;

        Ok(Self {
            initialized: true,
            devices: vec![],
            devices_ptr: std::ptr::null(),
        })
    }

	pub fn libusb_version_string() -> String {
		let (major, minor, micro, nano) = unsafe {
			let v = libusb_get_version();
			((*v).major, (*v).minor, (*v).micro, (*v).nano)
		};
	
		format!("libusb v{major}.{minor}.{micro}.{nano}")
	}

    pub fn devices(&mut self) -> Result<Vec<Device>, i32> {
        if !self.initialized {
            panic!("not initialized")
        }

        let (devices, ptr) = init::get_devices()?;

        self.devices_ptr = ptr;
        self.devices = devices.clone();

        Ok(devices)
    }
}

impl Drop for USBInterface {
    fn drop(&mut self) {
        unsafe { libusb_free_device_list(self.devices_ptr, 1) }

        for device in &mut self.devices {
            unsafe {
                dealloc(
                    device.descriptor as *mut u8,
                    Layout::new::<libusb_device_descriptor>(),
                )
            }
            unsafe { libusb_close(device.handle) }

            device.close();
        }
    }
}
