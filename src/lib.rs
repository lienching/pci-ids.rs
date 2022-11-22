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
//! Iterating over all known subclasses:
//!
//! ```rust
//! use pci_ids::Classes;
//!
//! for class in Classes::iter() {
//!     for subclass in class.subclasses() {
//!         println!("class: {}, subclass: {}", class.name(), subclass.name());
//!     }
//! }
//! ```
//!
//! See the individual documentation for each structure for more details.
//!

#![no_std]
#![warn(missing_docs)]

// Codegen: introduces VENDORS, a phf::Map<u16, Vendor>.
include!(concat!(env!("OUT_DIR"), "/pci_ids.cg.rs"));

/// An abstraction for iterating over all vendors in the PCI database.
pub struct Vendors;
impl Vendors {
    /// Returns an iterator over all vendors in the PCI database.
    pub fn iter() -> impl Iterator<Item = &'static Vendor> {
        VENDORS.values()
    }
}

/// Represents a PCI device vendor in the PCI database.
///
/// Every device vendor has a vendor ID, a pretty name, and a
/// list of associated [`Device`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Device {
    vendor_id: u16,
    id: u16,
    name: &'static str,
    subsystems: &'static [SubSystem],
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
        VENDORS.get(&self.vendor_id).unwrap()
    }

    /// Returns a tuple of (vendor ID, device/"product" ID) for this device.
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

    /// Returns an iterator over the device's subsystems.
    ///
    /// **NOTE**: The PCI database does not include subsystem information for
    /// most devices. This list is not authoritative.
    pub fn subsystems(&self) -> impl Iterator<Item = &'static SubSystem> {
        self.subsystems.iter()
    }
}

/// Represents an subsystem to a PCI device in the PCI database.
///
/// Every subsystem has subvendor and subdevice ID
/// and a pretty name.
///
/// **NOTE**: The PCI database is not a canonical or authoritative source
/// of subsystems information for devices. Users who wish to discover subsystems
/// on their PCI devices should query those devices directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// An abstraction for iterating over all classes in the PCI database.
pub struct Classes;

impl Classes {
    /// Returns an iterator over all classes in the PCI database.
    pub fn iter() -> impl Iterator<Item = &'static Class> {
        CLASSES.values()
    }
}

/// Represents a PCI device class in the PCI database.
///
/// Every device class has a class ID, a pretty name, and a list of associated [`Subclass`]es.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Class {
    id: u8,
    name: &'static str,
    subclasses: &'static [Subclass],
}

impl Class {
    /// Returns the class' ID.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Returns the class' name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns an iterator over the class' devices.
    pub fn subclasses(&self) -> impl Iterator<Item = &'static Subclass> {
        self.subclasses.iter()
    }
}

/// Represents a PCI device subclass in the PCI database.
///
/// Every subclass has a corresponding class, a subclass id, a pretty name, and a list of associated [`ProgIf`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subclass {
    class_id: u8,
    id: u8,
    name: &'static str,
    prog_ifs: &'static [ProgIf],
}

impl Subclass {
    /// Returns the [`Subclass`] corresponding to the given class and subclass IDs, or `None` if no such device exists in the DB.
    pub fn from_cid_sid(cid: u8, sid: u8) -> Option<&'static Self> {
        let class = Class::from_id(cid);

        class.and_then(|c| c.subclasses().find(|s| s.id == sid))
    }

    /// Returns the [`Class`] that this subclass belongs to.
    ///
    /// Looking up a class by subclass is cheap (`O(1)`).
    pub fn class(&self) -> &'static Class {
        CLASSES.get(&self.class_id).unwrap()
    }

    /// Returns a tuple of (class ID, subclass ID) for this subclass.
    ///
    /// This is conveniont for interactions with other PCI libraries.
    pub fn as_cid_sid(&self) -> (u8, u8) {
        (self.class_id, self.id)
    }

    /// Returns the subclass' ID.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Returns the subclass' name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Returns an iterator over the subclass' programming interfaces.
    ///
    /// **NOTE**: The PCI database does not include programming interface information for most devices.
    /// This list is not authoritative.
    pub fn prog_ifs(&self) -> impl Iterator<Item = &'static ProgIf> {
        self.prog_ifs.iter()
    }
}

/// Represents a programming interface to a PCI subclass in the PCI database.
///
/// Every programming interface has an ID and a pretty name.
///
/// **NOTE**: The PCI database is not a canonical or authoritative source of programming interface information for subclasses.
/// Users who wish to discover programming interfaces on their PCI devices should query those devices directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgIf {
    id: u8,
    name: &'static str,
}

impl ProgIf {
    /// Returns the programming interface's ID.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Returns the programming interface's name.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// A convenience trait for retrieving a top-level entity (like a [`Vendor`]) from the PCI
/// database by its unique ID.
pub trait FromId<T> {
    /// Returns the entity corresponding to `id`, or `None` if none exists.
    fn from_id(id: T) -> Option<&'static Self>;
}

impl FromId<u16> for Vendor {
    fn from_id(id: u16) -> Option<&'static Self> {
        VENDORS.get(&id)
    }
}

impl FromId<u8> for Class {
    fn from_id(id: u8) -> Option<&'static Self> {
        CLASSES.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_from_id() {
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
    fn test_device_from_vid_pid() {
        let device = Device::from_vid_pid(0x16ae, 0x000a).unwrap();

        assert_eq!(device.name(), "SafeXcel 1841");

        let (vid, pid) = device.as_vid_pid();

        assert_eq!(vid, device.vendor().id());
        assert_eq!(pid, device.id());

        let device2 = Device::from_vid_pid(vid, pid).unwrap();

        assert_eq!(device, device2);
    }

    #[test]
    fn test_class_from_id() {
        let class = Class::from_id(0x08).unwrap();

        assert_eq!(class.name(), "Generic system peripheral");
        assert_eq!(class.id(), 0x08);
    }

    #[test]
    fn test_class_subclasses() {
        let class = Class::from_id(0x01).unwrap();

        for subclass in class.subclasses() {
            assert_eq!(subclass.class(), class);
            assert!(!subclass.name().is_empty());
        }
    }

    #[test]
    fn test_subclass_from_cid_sid() {
        let subclass = Subclass::from_cid_sid(0x07, 0x00).unwrap();

        assert_eq!(subclass.name(), "Serial controller");

        let (cid, sid) = subclass.as_cid_sid();

        assert_eq!(cid, subclass.class().id());
        assert_eq!(sid, subclass.id());

        let subclass2 = Subclass::from_cid_sid(cid, sid).unwrap();

        assert_eq!(subclass, subclass2);
    }
}
