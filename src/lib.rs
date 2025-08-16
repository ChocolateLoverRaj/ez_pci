//! To use PCI, use [`PciAccess::new_pci`].
//! To use PCIe, use [`PciAccess::new_pcie`].
//!
//! Then you can scan buses.
//! For each bus, you can scan devices.
//! For each device, you can scan functions.
//! For each function, you can scan BARs, capabilities, and general info.
//!
//! You can also find and configure MSI (Message Signaled Interrupts)
#![no_std]
mod bar;
mod bus;
mod capabilities;
mod command;
mod device;
mod function;
mod get_phys_range_to_map;
mod header_type;
mod msi;
mod msi_x;
mod pci_access;
mod pci_config;

pub use bar::*;
pub use bus::*;
pub use capabilities::*;
pub use command::*;
pub use device::*;
pub use function::*;
pub use get_phys_range_to_map::*;
pub use header_type::*;
pub use msi::*;
pub use msi_x::*;
pub use pci_access::*;
use pci_config::*;
