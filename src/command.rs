use bitfield::bitfield;

bitfield! {
    pub struct CommandRegister(u16);

    pub io_space, set_io_space: 0;
    pub memory_space, set_memory_space: 1;
    pub bus_master, set_bus_master: 2;
    pub special_cycles, _: 3;
    pub memory_write_and_invalidate_enable, _: 4;
    pub vga_palette_snoop, _: 5;
    pub parity_error_response, set_parity_error_response: 6;
    // bit 7 is reserved
    pub serr_enable, set_serr_enable: 8;
    pub fast_back_to_back_enable, _: 9;
    pub interrupt_disable, set_interrupt_disable: 10;
    // bits 11..=15 are reserved
}
