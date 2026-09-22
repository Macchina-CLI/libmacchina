use std::{
    fs::{read_dir, read_to_string},
    io,
    path::PathBuf,
};

use pciid_parser::{schema::SubDeviceId, Database};

use crate::extra::pop_newline;

fn parse_device_hex(hex_str: &str) -> String {
    pop_newline(hex_str)
        .chars()
        .skip(2) // Remove the starting "0x"
        .take(4) // May be longer than 4 bytes, so just trim it down
        .collect::<String>()
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

    fn read_value(&self, readable_value: PciDeviceReadableValues) -> u16 {
        let value_path = self.base_path.join(readable_value.as_str());

        match read_to_string(&value_path) {
            // Safety: parse_device_hex should always make it 4 chars long, and
            // the kernel grantees that this is hex
            Ok(hex_string) => u16::from_str_radix(&parse_device_hex(&hex_string), 16).unwrap(),
            _ => panic!("Could not find value: {:?}", value_path),
        }
    }

    pub fn is_gpu(&self, db: &Database) -> bool {
        let class_value = self.read_value(PciDeviceReadableValues::Class);
        let classes = ["Display controller", "VGA compatible controller"];

        match db.classes.get(&((class_value >> 8) as u8)) {
            Some(class) => classes.contains(&class.name.as_str()),
            _ => false,
        }
    }

    pub fn get_device_name(&self, db: &Database) -> Option<String> {
        let vendor_value = self.read_value(PciDeviceReadableValues::Vendor);
        let sub_vendor_value = self.read_value(PciDeviceReadableValues::SubVendor);
        let device_value = self.read_value(PciDeviceReadableValues::Device);
        let sub_device_value = self.read_value(PciDeviceReadableValues::SubDevice);

        let vendor = db.vendors.get(&vendor_value)?;

        let device = vendor.devices.get(&device_value)?;
        // To return device name if no valid subdevice name is found
        let device_name = device.name.to_owned();

        let sub_device_id = SubDeviceId {
            subvendor: sub_vendor_value,
            subdevice: sub_device_value,
        };

        if let Some(sub_device) = device.subdevices.get(&sub_device_id) {
            let start = match sub_device.find('[') {
                Some(i) => i + 1,
                _ => return Some(device_name),
            };
            let end = sub_device.len() - 1;

            Some(sub_device.chars().take(end).skip(start).collect::<String>())
        } else {
            Some(device_name)
        }
    }
}
pub fn get_pci_devices() -> Result<Vec<PciDevice>, io::Error> {
    let devices_dir = read_dir("/sys/bus/pci/devices/")?;

    let mut devices = vec![];
    for device_entry in devices_dir.map_while(Result::ok) {
        devices.push(PciDevice::new(device_entry.path()));
    }

    Ok(devices)
}
