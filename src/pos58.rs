use std::io::{Error, ErrorKind, Result, Write};

pub struct POS58USB<'a> {
    handle: libusb::DeviceHandle<'a>,
    timeout: std::time::Duration,
    endpoint: u8,
}

const VENDOR_ID: u16 = 0x0416;
const PRODUCT_ID: u16 = 0x5011;

impl<'a> POS58USB<'a> {
    pub fn new(
        context: &'a mut libusb::Context,
        timeout: std::time::Duration,
    ) -> libusb::Result<Self> {
        let (device, device_desc, handle) =
            Self::get_device(context).ok_or(libusb::Error::NoDevice)?;
        let endpoint =
            Self::find_writeable_endpoint(&device, &device_desc).ok_or(libusb::Error::NotFound)?;
        Ok(POS58USB {
            endpoint,
            handle,
            timeout,
        })
    }

    fn get_device(
        context: &mut libusb::Context,
    ) -> Option<(
        libusb::Device,
        libusb::DeviceDescriptor,
        libusb::DeviceHandle,
    )> {
        let devices = context.devices().ok()?;

        for device in devices.iter() {
            if let Ok(device_desc) = device.device_descriptor() {
                if device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == PRODUCT_ID {
                    if let Ok(handle) = device.open() {
                        return Some((device, device_desc, handle));
                    }
                }
            }
        }
        None
    }

    fn find_writeable_endpoint(
        device: &libusb::Device,
        device_desc: &libusb::DeviceDescriptor,
    ) -> Option<u8> {
        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        if endpoint_desc.direction() == libusb::Direction::Out
                            && endpoint_desc.transfer_type() == libusb::TransferType::Bulk
                        {
                            return Some(endpoint_desc.address());
                        }
                    }
                }
            }
        }
        None
    }
}

impl<'a> Write for POS58USB<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.handle.write_bulk(self.endpoint, buf, self.timeout) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(Error::from(ErrorKind::Other)),
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.write(b"\n").map(|_| ())
    }
}
