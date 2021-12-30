use crate::enums::ReadoutError;
use crate::extra;
use crate::shared;
use crate::traits::NetworkReadout;
use std::path::PathBuf;

pub struct LinuxNetworkReadout;

impl NetworkReadout for LinuxNetworkReadout {
    fn new() -> Self {
        LinuxNetworkReadout
    }

    fn tx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/tx_bytes");
            let content = std::fs::read_to_string(rx_file)?;
            let bytes = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(bytes)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn tx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/tx_packets");
            let content = std::fs::read_to_string(rx_file)?;
            let packets = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(packets)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn rx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/rx_bytes");
            let content = std::fs::read_to_string(rx_file)?;
            let bytes = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(bytes)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn rx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/rx_packets");
            let content = std::fs::read_to_string(rx_file)?;
            let packets = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(packets)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn physical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net").join(ifname).join("address");
            let content = std::fs::read_to_string(rx_file)?;
            Ok(content)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn logical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        shared::logical_address(interface)
    }
}
