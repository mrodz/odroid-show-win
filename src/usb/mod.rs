mod comms;
mod debug;
mod init;

extern crate libc;
extern crate libusb1_sys as ffi;

pub use std::result::Result::Ok as StdOk;
use anyhow::{Result, bail, Ok};
use ffi::constants::{LIBUSB_SUCCESS, LIBUSB_ERROR_NOT_FOUND};
use ffi::*;
use std::alloc::{dealloc, Layout};
use std::fmt::{Debug, Display};

use self::init::get_handle;

/// Pushes a pointer of type `T` to the stack.
/// ### Does not allocate the space of T.
///
/// Returns a pointer to a mutable T that points to `null`
#[inline]
fn ffi_ptr_mut<T>() -> *mut T {
    std::ptr::null_mut() as *mut T
}

/// Pushes a pointer of type `T` to the stack.
/// ### Does not allocate the space of T.
///
/// Returns a pointer to a const T that points to `null`
#[inline]
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
    /// Create a new device wrapper
    fn new(
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

    pub fn debug_verbose(&self) {
        print!("{self}");
        for i in 0..unsafe { (*self.descriptor).bNumConfigurations } {
            let mut config_descriptor = ffi_ptr_const::<libusb_config_descriptor>();
            let code = unsafe {
                libusb_get_config_descriptor(self.libusb_device, i, &mut config_descriptor)
            };

            if code != LIBUSB_SUCCESS {
                println!("  Couldn't retrieve descriptors");
                continue;
            }

            debug::print_config(config_descriptor);

            unsafe { libusb_free_config_descriptor(config_descriptor) }
        }
    }

    pub fn read(&self) -> Result<()> {
        comms::read(&self, 0)?;

        Ok(())
    }

    /// You can only trust this struct's fields if this call returns true.
    ///
    /// Otherwise, pointers might have been dropped.
    pub fn can_use(&self) -> bool {
        return !self.closed;
    }

    /// Indicate this device can no longer be trusted. [`Self::can_use`]
    pub fn close(&mut self) {
        self.closed = true;
    }
}

/// Custom display code.
impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use init::descriptor_to_string_check;

        if !self.can_use() {
            return write!(f, "Inaccessible USB device (was it dropped?)");
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

        let product_id = unsafe { (*self.descriptor).idProduct };
        let vendor_id = unsafe { (*self.descriptor).idVendor };

        write!(
            f,
            "libusb-compatible device\n\
		        - product: {product} (p.ID {product_id})\n\
                - manufacturer: {manufacturer} (v.ID {vendor_id})\n\
				- serial number: {serial}\n\
			",
        )
    }
}

/// Interface to FFI bindings for [libusb](https://libusb.info/).
/// # Example
/// ```
/// use usb::USBInterface;
///
/// let mut x = USBInterface::new().unwrap();
/// println!("$ Demo running {}\n", USBInterface::libusb_version_string());
/// let devices = x.open_all_devices().unwrap();
/// for device in devices {
///		println!("{}", device);
/// }
/// ```
///
pub struct USBInterface {
    initialized: bool,
    devices: Vec<Device>,
    devices_ptr: *const *mut libusb_device, // must be freed via `libusb_free_device_list`
    devices_len: isize,
}

impl USBInterface {
    /// Create a new `USBInterface`.
    /// Will construct the interface or return an error code if initialization fails.
    /// # Caution: constructing multiple instances is undefined behavior.
    pub fn new() -> Result<Self> {
        init::init()?;
        let (devices, devices_len) = init::get_libusb_devices();

        Ok(Self {
            initialized: true,
            devices: vec![],
            devices_ptr: devices,
            devices_len: devices_len,
        })
    }

    /// Get the version of [libusb](https://libusb.info/) running under the hood.
    #[inline]
    pub fn libusb_version_string() -> String {
        let (major, minor, micro, nano) = unsafe {
            let v = libusb_get_version();
            ((*v).major, (*v).minor, (*v).micro, (*v).nano)
        };

        format!("libusb v{major}.{minor}.{micro}.{nano}")
    }

    /// Return a vector of accessible devices, or an error code if this fails.
    /// Will also store a clone of the devices pointer to be freed (using [`libusb_free_device_list`])
    /// on drop.
    pub fn open_all_devices(&mut self) -> Result<Vec<Device>> {
        assert!(self.initialized);

        let (devices, ptr) = init::get_devices(self.devices_ptr, self.devices_len)?;

        self.devices_ptr = ptr;
        self.devices = devices.clone();

        Ok(devices)
    }

    pub fn open_device(&self, product_id: u16, vendor_id: u16) -> Result<Device> {
        assert!(self.initialized);

        for i in 0..self.devices_len {
            let device = unsafe { *self.devices_ptr.offset(i) };

            let descriptor = init::get_descriptor(device)?; // possible break

            unsafe {
                if (*descriptor).idProduct == product_id && (*descriptor).idVendor == vendor_id {
                    let handle = get_handle(device, descriptor)?;

                    libusb_set_auto_detach_kernel_driver(handle, 1);

                    dbg!(handle);

                    return Ok(Device::new(descriptor, handle, device));
                }
            }
        }

        bail!(LIBUSB_ERROR_NOT_FOUND)
    }
}

/// Cleanup logic for the `USBInterface`
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
