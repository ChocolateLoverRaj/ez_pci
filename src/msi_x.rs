use core::{
    fmt::Debug,
    num::NonZero,
    ptr::{NonNull, slice_from_raw_parts_mut},
};

use bitfield::bitfield;
use volatile::{
    VolatileFieldAccess, VolatileRef,
    access::{ReadOnly, ReadWrite},
};

use super::*;

pub struct MsiX<'a> {
    pci: &'a mut PciAccess,
    bus_number: u8,
    device_number: u8,
    function_number: u8,
    ptr: u8,
}

impl<'a> MsiX<'a> {
    pub(super) fn find(function: &'a mut PciFunction) -> Option<Option<Self>> {
        if let Some(capability) = function
            .capabilities()?
            .find(|capability| capability.id == 0x11)
        {
            Some(Some(Self {
                pci: function.pci,
                bus_number: function.bus_number,
                device_number: function.device_number,
                function_number: function.function_number,
                ptr: capability.ptr_to_self,
            }))
        } else {
            Some(None)
        }
    }
}

impl MsiX<'_> {
    pub fn message_control(&mut self) -> MsiXMessageControl {
        MsiXMessageControl(self.pci.read_u16(
            self.bus_number,
            self.device_number,
            self.function_number,
            self.ptr + 0x2,
        ))
    }

    pub fn set_message_control(&mut self, message_control: MsiXMessageControl) {
        self.pci.write_u16(
            self.bus_number,
            self.device_number,
            self.function_number,
            self.ptr + 0x2,
            message_control.0,
        );
    }

    pub fn table_location(&mut self) -> MsiXLocation {
        MsiXLocation(self.pci.read_u32(
            self.bus_number,
            self.device_number,
            self.function_number,
            self.ptr + 0x4,
        ))
    }

    /// The location of the Pending Bit Array
    pub fn pba_location(&mut self) -> MsiXLocation {
        MsiXLocation(self.pci.read_u32(
            self.bus_number,
            self.device_number,
            self.function_number,
            self.ptr + 0x8,
        ))
    }

    /// To use this function, you must:
    /// - Find out which BAR the table is located in using [`Self::table_location`].
    /// - Map the BAR (it will always be MMIO) using the correct memory type
    /// - Input the virtual address that points to the **start** of the BAR
    ///
    /// # Safety
    /// The virtual address must be mapped to the **start** of the BAR.
    pub unsafe fn table<'a>(&mut self, bar_virt_addr: NonZero<usize>) -> MsiXTable<'a> {
        let table_addr = bar_virt_addr
            .checked_add(self.table_location().offset_in_bar() as usize)
            .expect("Doesn't overflow");
        let table_size = self.message_control().table_size();
        unsafe { MsiXTable::new(table_addr, table_size) }
    }

    /// To use this function, you must:
    /// - Find out which BAR the table is located in using [`Self::pba_location`].
    /// - Map the BAR (it will always be MMIO) using the correct memory type
    /// - Input the virtual address that points to the **start** of the BAR
    ///
    /// # Safety
    /// The virtual address must be mapped to the **start** of the BAR.
    pub unsafe fn pending_bit_array<'a>(
        &mut self,
        bar_virt_addr: NonZero<usize>,
    ) -> MsiXPendingBitArray<'a> {
        let table_addr = bar_virt_addr
            .checked_add(self.pba_location().offset_in_bar() as usize)
            .expect("Doesn't overflow");
        let table_size = self.message_control().table_size();
        unsafe { MsiXPendingBitArray::new(table_addr, table_size) }
    }
}

bitfield! {
    /// PCI Local Bus Specification Rev. 3.0 -> 6.8.2.3. Message Control for MSI-X
    #[derive(Clone, Copy)]
    pub struct MsiXMessageControl(u16);
    impl Debug;

    u16;
    /// The table size is encoded as N-1. So if 3 is stored, that means the table size is actually 4.
    _table_size, _: 10, 0;
    pub function_mask, _: 14;
    pub enable, set_enable: 15;
}

impl MsiXMessageControl {
    pub fn table_size(&self) -> u16 {
        self._table_size() + 1
    }
}

bitfield! {
    /// The table and pending bit array are stored inside a BAR. The BAR index and offset inside the BAR are encoded in a `u32``.
    /// PCI Local Bus Specification Rev. 3.0 -> 6.8.2.4. Table Offset/Table BIR for MSI-X
    #[derive(Clone, Copy)]
    pub struct MsiXLocation(u32);
    impl Debug;

    u32;
    /// Actual start bit is 0, but bits 2:0 are used for something else
    _offset_in_bar, _: 31, 3;

    u8;
    /// The BAR index that contains the table
    pub bar_index, _: 2, 0;
}

impl MsiXLocation {
    pub fn offset_in_bar(&self) -> u32 {
        self._offset_in_bar() << 3
    }
}

#[derive(Debug, Clone, Copy, VolatileFieldAccess)]
#[repr(C)]
pub struct MsiXTableEntry {
    /// In reality this is documented as a high and low u32, but on little-endian systems we can just treat it as a u64.
    /// It is aligned for u64 access and u64 access is allowed.
    #[access(ReadWrite)]
    pub message_address: u64,
    #[access(ReadWrite)]
    pub message_data: u32,
    #[access(ReadWrite)]
    pub vector_control: MsiXVectorControl,
}

bitfield! {
    /// PCI Local Bus Specification Rev. 3.0 -> 6.8.2.9. Vector Control for MSI-X Table Entries
    #[derive(Clone, Copy)]
    pub struct MsiXVectorControl(u32);
    impl Debug;

    pub mask, set_mask: 0;
}

pub struct MsiXTable<'a> {
    ptr: VolatileRef<'a, [MsiXTableEntry]>,
}

impl MsiXTable<'_> {
    unsafe fn new(table_addr: NonZero<usize>, table_size: u16) -> Self {
        Self {
            ptr: {
                let ptr = NonNull::new(slice_from_raw_parts_mut(
                    table_addr.get() as *mut MsiXTableEntry,
                    table_size as usize,
                ))
                .expect("ptr is not null");
                unsafe { VolatileRef::new(ptr) }
            },
        }
    }

    pub fn entry_mut(&mut self, index: u16) -> VolatilePtr<MsiXTableEntry> {
        self.ptr.as_mut_ptr().index(index as usize)
    }
}

pub use volatile::VolatilePtr;

impl Debug for MsiXTable<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries((0..self.ptr.as_ptr().len()).map(|i| self.ptr.as_ptr().index(i).read()))
            .finish()
    }
}

/// This array tells you which interrupts are **pending delivery**. This is read-only to the kernel.
#[derive(Debug)]
pub struct MsiXPendingBitArray<'a> {
    array: VolatileRef<'a, [u64], ReadOnly>,
}

impl<'a> MsiXPendingBitArray<'a> {
    /// # Safety
    /// The pointer must be a pointer to the pending bit array. Remember to map it with the correct memory type.
    unsafe fn new(pba_addr: NonZero<usize>, table_size: u16) -> Self {
        Self {
            array: {
                let ptr = NonNull::new(slice_from_raw_parts_mut(
                    pba_addr.get() as *mut u64,
                    table_size.div_ceil(size_of::<u64>() as u16) as usize,
                ))
                .expect("ptr is not null");
                unsafe { VolatileRef::new_read_only(ptr) }
            },
        }
    }

    pub fn is_pending(&self, entry: u16) -> bool {
        let u64_index = entry / u64::BITS as u16;
        let bit_index = entry % u64::BITS as u16;
        (self.array.as_ptr().index(u64_index as usize).read() >> bit_index) & 1 != 0
    }
}
