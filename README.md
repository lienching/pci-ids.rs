pci-ids
=======

### This project is modified from wooduffw's usb-ids.rs (https://github.com/woodruffw/usb-ids.rs) 

![license](https://raster.shields.io/badge/license-MIT%20with%20restrictions-green.png)
[![Build Status](https://img.shields.io/github/workflow/status/lienching/pci-ids.rs/CI/main)](https://github.com/lienching/pci-ids.rs/actions?query=workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/pci-ids)](https://crates.io/crates/pci-ids)

Cross-platform Rust wrappers for the [PCI ID Repository](https://pci-ids.ucw.cz/).

This library bundles the PCI ID database, allowing platforms other than Linux to query it
as a source of canonical PCI metadata.

## Usage

Iterating over all known vendors:

```rust
use pci_ids::Vendors;

for vendor in Vendors::iter() {
    for device in vendor.devices() {
        println!("vendor: {}, device: {}", vendor.name(), device.name());
    }
}
```
