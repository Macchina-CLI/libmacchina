use crate::extra;
use crate::traits::ProductReadout;
use crate::traits::ReadoutError;
use itertools::Itertools;
use std::fs;

pub struct LinuxProductReadout;

impl ProductReadout for LinuxProductReadout {
    fn new() -> Self {
        LinuxProductReadout
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/sys_vendor",
        )?))
    }

    fn family(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/product_family",
        )?))
    }

    fn product(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/product_name",
        )?))
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        let vendor = self.vendor()?;
        let family = self.family()?;
        let product = self.product()?;
        let version = extra::pop_newline(fs::read_to_string("/sys/class/dmi/id/product_version")?);

        // If one field is generic, the others are likely the same, so fail the readout.
        if vendor.eq_ignore_ascii_case("system manufacturer") {
            return Err(ReadoutError::Other(String::from(
                "Your manufacturer may have not specified your machine's product information.",
            )));
        }

        let new_product = format!("{} {} {} {}", vendor, family, product, version)
            .replace("To be filled by O.E.M.", "");

        if family == product && family == version {
            return Ok(family);
        } else if version.is_empty() || version.len() <= 22 {
            return Ok(new_product
                .split_whitespace()
                .into_iter()
                .unique()
                .join(" "));
        }

        Ok(version)
    }
}
