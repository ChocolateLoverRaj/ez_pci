[![Crates.io](https://img.shields.io/crates/v/ez_pci.svg)](https://crates.io/crates/ez_pci)
[![Docs.rs](https://img.shields.io/docsrs/ez_pci)](https://docs.rs/ez_pci)

The goal of this library is to make it very easy to implement PCI in your own OS.

## Features
- PCI (I/O based) and PCIe (MMIO based)
- Scan buses, devices, and functions
- Read BAR address and size
- Iterate through capabilities
- Configure MSI
- Configure MSI-X

## Planned
- Better concurrent access

## Tested on
- QEMU

This crate is still very new. If you notice any limitations with this crate have feature requests, or want help using this crate, feel free to open an issue on GitHub.
 