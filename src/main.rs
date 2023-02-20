use usb::USBInterface;

mod usb;


// extern crate libusb1_sys as ffi;
// use ffi::libusb_device;

fn main() {
    let mut x = USBInterface::new().unwrap();

    println!("$ Demo running {}\n", USBInterface::libusb_version_string());

    let devices = x.devices().unwrap();

    for device in devices {
        println!("{}", device);
    }

}
