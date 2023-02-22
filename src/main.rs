mod usb;

use anyhow::{Result, Ok, Context};
use usb::USBInterface;

fn main() -> Result<()> {
    

    let mut x = USBInterface::new().context("Failed to set up the USB Interface")?;

    println!("$ Demo running {}\n", USBInterface::libusb_version_string());

    let show2 = x.open_device(60000, 4292)?;
    // show2.debug_verbose();

    show2.read()?;

    // show2.read();

    // for device in x.open_all_devices().context("Failed to open all devices")? {
    //     println!("{device}");
    // }

    Ok(())
}
