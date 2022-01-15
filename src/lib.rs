//!
//! Rust wrappers for the [PCI ID Repository](https://pci-ids.ucw.cz/v2.2/pci.ids).
//!
//! The PCI ID Repository is the canonical source of PCI device information for most
//! Linux userspaces; this crate vendors the PCI ID database to allow non-Linux hosts to
//! access the same canonical information.
//!
//! # Usage
//!
//! Iterating over all known vendors:
//!
//! ```rust
//! use pci_ids::Vendors;
//!
//! for vendor in Vendors::iter() {
//!     for device in vendor.devices() {
//!         println!("vendor: {}, device: {}", vendor.name(), device.name());
//!     }
//! }
//! ```
//!
//! See the individual documentation for each structure for more details.
//!

#![no_std]
#![warn(missing_docs)]

// Codegen: introduces PCI_IDS, a phf::Map<u16, Vendor>.
include!(concat!(env!("OUT_DIR"), "/pci_ids.cg.rs"));

/// An abstraction for iterating over all vendors in the PCI database.
pub struct Vendors;
impl Vendors {
    /// Returns an iterator over all vendors in the PCI database.
    pub fn iter() -> impl Iterator<Item = &'static Vendor> {
        PCI_IDS.values()
    }
}

/// Represents a PCI device vendor in the PCI database.
///
/// Every device vendor has a vendor ID, a pretty name, and a
/// list of associated [`Device`]s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vendor {
    id: u16,
    name: &'static str,
    devices: &'static [Device],
}

impl Vendor {
    /// Returns the vendor's ID.
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Returns the vendor's name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns an iterator over the vendor's devices.
    pub fn devices(&self) -> impl Iterator<Item = &'static Device> {
        self.devices.iter()
    }
}

/// Represents a single device in the PCI database.
///
/// Every device has a corresponding vendor, a device ID, a pretty name,
/// and a list of associated [`SubSystem`]s.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Device {
    vendor_id: u16,
    id: u16,
    name: &'static str,
    subsystem: &'static [SubSystem],
}

impl Device {
    /// Returns the [`Device`] corresponding to the given vendor and product IDs,
    /// or `None` if no such device exists in the DB.
    pub fn from_vid_pid(vid: u16, pid: u16) -> Option<&'static Device> {
        let vendor = Vendor::from_id(vid);

        vendor.and_then(|v| v.devices().find(|d| d.id == pid))
    }

    /// Returns the [`Vendor`] that this device belongs to.
    ///
    /// Looking up a vendor by device is cheap (`O(1)`).
    pub fn vendor(&self) -> &'static Vendor {
        PCI_IDS.get(&self.vendor_id).unwrap()
    }

    /// Returns a tuple of (vendor id, device/"product" id) for this device.
    ///
    /// This is convenient for interactions with other PCI libraries.
    pub fn as_vid_pid(&self) -> (u16, u16) {
        (self.vendor_id, self.id)
    }

    /// Returns the device's ID.
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Returns the device's name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns an iterator over the device's subsystem.
    ///
    /// **NOTE**: The PCI database does not include subsystem information for
    /// most devices. This list is not authoritative.
    pub fn subsystem(&self) -> impl Iterator<Item = &'static SubSystem> {
        self.subsystem.iter()
    }
}

/// Represents an subsystem to a PCI device in the PCI database.
///
/// Every subsystem has subvendor and subdevice ID
/// and a pretty name.
///
/// **NOTE**: The PCI database is not a canonical or authoritative source
/// of subsystem information for devices. Users who wish to discover subsystem
/// on their PCI devices should query those devices directly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubSystem {
    subvendor: u16,
    subdevice: u16,
    name: &'static str,
}

impl SubSystem {
    /// Returns the subsystem's ID.
    pub fn subvendor(&self) -> u16 {
        self.subvendor
    }

    /// Returns the subsystem's subdevice.
    pub fn subdevice(&self) -> u16 {
        self.subdevice
    }

    /// Returns the subsystem's name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// A convenience trait for retrieving a top-level entity (like a [`Vendor`]) from the PCI
/// database by its unique ID.
// NOTE(ww): This trait will be generally useful once we support other top-level
// entities in `PCI.ids` (like language, country code, HID codes, etc).
pub trait FromId<T> {
    /// Returns the entity corresponding to `id`, or `None` if none exists.
    fn from_id(id: T) -> Option<&'static Self>;
}

impl FromId<u16> for Vendor {
    fn from_id(id: u16) -> Option<&'static Self> {
        PCI_IDS.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_id() {
        let vendor = Vendor::from_id(0x14c3).unwrap();

        assert_eq!(vendor.name(), "MEDIATEK Corp.");
        assert_eq!(vendor.id(), 0x14c3);
    }

    #[test]
    fn test_vendor_devices() {
        let vendor = Vendor::from_id(0x17cb).unwrap();

        for device in vendor.devices() {
            assert_eq!(device.vendor(), vendor);
            assert!(!device.name().is_empty());
        }
    }

    #[test]
    fn test_from_vid_pid() {
        let device = Device::from_vid_pid(0x16ae, 0x000a).unwrap();

        assert_eq!(device.name(), "SafeXcel 1841");

        let (vid, pid) = device.as_vid_pid();

        assert_eq!(vid, device.vendor().id());
        assert_eq!(pid, device.id());

        let device2 = Device::from_vid_pid(vid, pid).unwrap();

        assert_eq!(device, device2);
    }
}
