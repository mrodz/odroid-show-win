use libusb1_sys::{constants::*, *};

use super::Device;

pub(super) fn read(device: &Device, interface_number: i32) {
    detach_kernel_if_active(device.handle, interface_number);

    let mut data = [0_u8; 4];
    let mut actual_length = 0;

    loop {
        let r = unsafe {
            libusb_bulk_transfer(
                device.handle,
                LIBUSB_ENDPOINT_IN,
                data.as_mut_ptr(),
                std::mem::size_of_val(&data).try_into().unwrap(),
                &mut actual_length,
                0,
            )
        };

        if r == LIBUSB_SUCCESS {
            println!("Data: {:?}", data);
        } else {
            // print!("{r},")
        }
    }
}

fn detach_kernel_if_active(handle: *mut libusb_device_handle, interface_number: i32) {
    let code = unsafe { libusb_kernel_driver_active(handle, interface_number) };

    let code_2 = unsafe { libusb_detach_kernel_driver(handle, interface_number) };
    println!("{code},{code_2}");

    match code {
        0 => (),
        1 if code_2 != 0 => {
            eprintln!("Unable to detach driver: this device is unusable.")
        }
        LIBUSB_ERROR_NO_DEVICE => eprintln!("Device disconnected."),
        LIBUSB_ERROR_NOT_SUPPORTED => eprintln!("Platform does not support detaching drivers."),
        x => eprintln!("Unable to detach driver (code {x})."),
    }

    let code = unsafe { libusb_claim_interface(handle, interface_number) };

    if code != 0 {
        eprintln!("Unable to claim the device interface (code {code}).");
    }
}
