use libusb1_sys::{
    constants::{LIBUSB_DT_SS_ENDPOINT_COMPANION, LIBUSB_SUCCESS},
    libusb_config_descriptor, libusb_endpoint_descriptor,
    libusb_free_ss_endpoint_companion_descriptor, libusb_get_ss_endpoint_companion_descriptor,
    libusb_interface, libusb_interface_descriptor, libusb_ss_endpoint_companion_descriptor,
};

use crate::usb::ffi_ptr_mut;

fn print_endpoint_comp(ep_comp: *const libusb_ss_endpoint_companion_descriptor) {
    unsafe {
        println!("     - USB 3.0 Endpoint Companion:");
        println!("       - bMaxBurst:           {}", (*ep_comp).bMaxBurst);
        println!("       - bmAttributes:        {}", (*ep_comp).bmAttributes);
        println!(
            "       - wBytesPerInterval:   {}",
            (*ep_comp).wBytesPerInterval
        );
    }
}

fn print_endpoint(endpoint: *const libusb_endpoint_descriptor) {
    unsafe {
        println!("     - ENDPOINT:");
        println!(
            "       - bEndpointAddress:    {}",
            (*endpoint).bEndpointAddress
        );
        println!("       - bmAttributes:        {}", (*endpoint).bmAttributes);
        println!(
            "       - wMaxPacketSize:      {}",
            (*endpoint).wMaxPacketSize
        );
        println!("       - bInterval:           {}", (*endpoint).bInterval);
        println!("       - bRefresh:            {}", (*endpoint).bRefresh);
        println!(
            "       - bSynchAddress:       {}",
            (*endpoint).bSynchAddress
        );

        let mut i = 0;

        while i < (*endpoint).extra_length {
            if *(*endpoint).extra.offset((i + 1).try_into().unwrap())
                != LIBUSB_DT_SS_ENDPOINT_COMPANION
            {
                let ep_comp = ffi_ptr_mut::<libusb_ss_endpoint_companion_descriptor>();
                let code = libusb_get_ss_endpoint_companion_descriptor(
                    std::ptr::null_mut(),
                    endpoint,
                    &mut (ep_comp as *const libusb_ss_endpoint_companion_descriptor),
                );

                if code != LIBUSB_SUCCESS {
                    continue;
                }

                print_endpoint_comp(ep_comp);

                libusb_free_ss_endpoint_companion_descriptor(&mut *ep_comp);
            }

            i += *(*endpoint).extra.offset(i as isize) as i32;
        }
    }
}

fn print_altsetting(interface: *const libusb_interface_descriptor) {
    unsafe {
        println!("   - INTERFACE:");
        println!(
            "     - bInterfaceNumber:      {}",
            (*interface).bInterfaceNumber
        );
        println!(
            "     - bAlternateSetting:     {}",
            (*interface).bAlternateSetting
        );
        println!(
            "     - bNumEndpoints:         {}",
            (*interface).bNumEndpoints
        );
        println!(
            "     - bInterfaceClass:       {}",
            (*interface).bInterfaceClass
        );
        println!(
            "     - bInterfaceSubClass:    {}",
            (*interface).bInterfaceSubClass
        );
        println!(
            "     - bInterfaceProtocol:    {}",
            (*interface).bInterfaceProtocol
        );
        println!("     - iInterface:            {}", (*interface).iInterface);

        for i in 0..(*interface).bNumEndpoints {
            print_endpoint((*interface).endpoint.offset(i.into()))
        }
    }
}

fn print_interface(interface: *const libusb_interface) {
    unsafe {
        for i in 0..(*interface).num_altsetting {
            print_altsetting((*interface).altsetting.offset(i as isize))
        }
    }
}

pub(super) fn print_config(config: *const libusb_config_descriptor) {
    unsafe {
        println!(" - CONFIGURATION:");
        println!("   - wTotalLength:            {}", (*config).wTotalLength);
        println!("   - bNumInterfaces:          {}", (*config).bNumInterfaces);
        println!(
            "   - bConfigurationValue:     {}",
            (*config).bConfigurationValue
        );
        println!("   - iConfiguration:          {}", (*config).iConfiguration);
        println!("   - bmAttributes:            {}", (*config).bmAttributes);
        println!("   - MaxPower:                {}", (*config).bMaxPower);

        for i in 0..(*config).bNumInterfaces {
            print_interface((*config).interface.offset(i.into()))
        }
    }
}
