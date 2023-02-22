use anyhow::{bail, Result};
use libusb1_sys::{constants::*, *};

use super::Device;
const BULK_EP_OUT: u8 = 0x82;
const BULK_EP_IN: u8 = 0x02;

pub(super) fn read(device: &Device, interface_number: i32) -> Result<()> {
    detach_kernel_if_active(device.handle, interface_number); // if on windows, does nothing.

    let code = unsafe { libusb_claim_interface(device.handle, interface_number) };

    if code != 0 {
        eprintln!("Unable to claim the device interface (code {code}).");
    }

    let mut data = 0x06; // ASCII 'AKG'
    let mut actual_length = 0;

    // loop {
    unsafe {
        let r = libusb_bulk_transfer(
            device.handle,
            BULK_EP_OUT,
            &mut data,
            std::mem::size_of_val(&data).try_into().unwrap(),
            &mut actual_length,
            1_000,
        );

        if r == LIBUSB_SUCCESS {
            println!("Data: {:?}", data);
        } else {
            bail!("{r} || handle = {:?}", device.handle)
        }
    }
    Ok(())
    // }
}

fn detach_kernel_if_active(handle: *mut libusb_device_handle, interface_number: i32) {
    let code = unsafe { libusb_kernel_driver_active(handle, interface_number) };

    if code == LIBUSB_SUCCESS || code == LIBUSB_ERROR_NOT_SUPPORTED {
        return;
    }

    let code_2 = unsafe { libusb_detach_kernel_driver(handle, interface_number) };

    match code {
        0 => (),
        1 if code_2 != 0 => {
            eprintln!("Unable to detach driver: this device is unusable.")
        }
        LIBUSB_ERROR_NO_DEVICE => eprintln!("Device disconnected."),
        LIBUSB_ERROR_NOT_SUPPORTED => eprintln!("Platform does not support detaching drivers."),
        x => eprintln!("Unable to detach driver (code {x})."),
    }
}
