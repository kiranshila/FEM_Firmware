use rp2040_hal as hal;

// Crystal freq
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

// Don't forget the second stage bootloader
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_IS25LP080;

// And add all of our pins!
hal::bsp_pins! {
    Gpio20 {
        name: rf1_status_led,
        aliases: { PushPullOutput: Rf1StatusLed }
    },
    Gpio21 {
        name: rf2_status_led,
        aliases: {PushPullOutput: Rf2StatusLed }
    },
    Gpio22 {
        name: rf1_lna_en,
        aliases: {PushPullOutput: Rf1LnaEn }
    },
    Gpio23 {
        name: rf2_lna_en,
        aliases: {PushPullOutput: Rf2LnaEn }
    },
    Gpio26 {
        name: rf1_if_pow,
        aliases: {FloatingInput: Rf1IfPow}
    }
    Gpio27 {
        name: rf2_if_pow,
        aliases: {FloatingInput: Rf2IfPow}
    }
}
