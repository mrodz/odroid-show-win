mod usb;

use usb::USBInterface;

fn main() {
    let mut x = USBInterface::new().unwrap();

    println!("$ Demo running {}\n", USBInterface::libusb_version_string());

    let devices = x.devices().unwrap();

    for device in devices {
        println!("{}", device);
    }
}
