use std::{
    fs::{read_dir, read_to_string},
    io,
    path::PathBuf,
};

use pciid_parser::{schema::SubDeviceId, Database};

use crate::extra::pop_newline;

fn parse_device_hex(hex_str: &str) -> String {
    pop_newline(hex_str).chars().skip(2).collect::<String>()
}

pub enum PciDeviceReadableValues {
    Class,
    Vendor,
    Device,
    SubVendor,
    SubDevice,
}

impl PciDeviceReadableValues {
    fn as_str(&self) -> &'static str {
        match self {
            PciDeviceReadableValues::Class => "class",
            PciDeviceReadableValues::Vendor => "vendor",
            PciDeviceReadableValues::Device => "device",
            PciDeviceReadableValues::SubVendor => "subsystem_vendor",
            PciDeviceReadableValues::SubDevice => "subsystem_device",
        }
    }
}

#[derive(Debug)]
pub struct PciDevice {
    base_path: PathBuf,
}

impl PciDevice {
    fn new(base_path: PathBuf) -> PciDevice {
        PciDevice { base_path }
    }

    fn _read_value(&self, readable_value: PciDeviceReadableValues) -> String {
        let value_path = self.base_path.join(readable_value.as_str());

        match read_to_string(&value_path) {
            Ok(hex_string) => parse_device_hex(&hex_string),
            _ => panic!("Could not find value: {:?}", value_path),
        }
    }

    pub fn is_gpu(&self, db: &Database) -> bool {
        let class_value = self._read_value(PciDeviceReadableValues::Class);
        let first_pair = class_value.chars().take(2).collect::<String>();

        match db.classes.get(&first_pair) {
            Some(class) => class.name == "Display controller",
            _ => false,
        }
    }

    pub fn get_sub_device_name(&self, db: &Database) -> String {
        let vendor_value = self._read_value(PciDeviceReadableValues::Vendor);
        let sub_vendor_value = self._read_value(PciDeviceReadableValues::SubVendor);
        let device_value = self._read_value(PciDeviceReadableValues::Device);
        let sub_device_value = self._read_value(PciDeviceReadableValues::SubDevice);

        let vendor = match db.vendors.get(&vendor_value) {
            Some(vendor) => vendor,
            _ => panic!("Could not find vendor for value: {}", vendor_value),
        };

        let device = match vendor.devices.get(&device_value) {
            Some(device) => device,
            _ => panic!("Could not find device for value: {}", device_value),
        };

        let sub_device_id = SubDeviceId {
            subvendor: sub_vendor_value,
            subdevice: sub_device_value,
        };

        match device.subdevices.get(&sub_device_id) {
            Some(sub_device) => {
                let start = match sub_device.find('[') {
                    Some(i) => i + 1,
                    _ => panic!(
                        "Could not find opening square bracket for sub device: {}",
                        sub_device
                    ),
                };
                let end = sub_device.len() - 1;

                sub_device.chars().take(end).skip(start).collect::<String>()
            }
            _ => panic!("Could not find sub device for id: {:?}", sub_device_id),
        }
    }
}

pub fn get_pci_devices() -> Result<Vec<PciDevice>, io::Error> {
    let devices_dir = read_dir("/sys/bus/pci/devices/")?;

    let mut devices = vec![];
    for device_entry in devices_dir.flatten() {
        devices.push(PciDevice::new(device_entry.path()));
    }

    Ok(devices)
}
