mod usb;

use usb::USBInterface;

fn main() {
    let x = USBInterface::new().unwrap();

    println!("$ Demo running {}\n", USBInterface::libusb_version_string());

    let show2 = x.open_device(60000, 4292).unwrap();
    show2.debug_verbose();
}
