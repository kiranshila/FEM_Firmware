use rp2040_hal as hal;

// Crystal freq
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

// Don't forget the second stage bootloader
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// And add all of our pins!
hal::bsp_pins! {
    Gpio25 {
        name: led,
    },
}
