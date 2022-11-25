#![no_std]
#![no_main]

mod bsp;

use bsp::XOSC_CRYSTAL_FREQ;
use defmt::*;
use defmt_rtt as _;
use hal::pac;
use panic_probe as _;
use rp2040_hal as hal;
use rp2040_monotonic::{ExtU64, Rp2040Monotonic};

use hal::gpio::{bank0::Gpio25, Output, Pin, PushPull};

// GPIO traits
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};

// Bind software tasks to SIO_IRQ_PROC0, we're not using it
#[rtic::app(device = pac, peripherals = true, dispatchers = [SIO_IRQ_PROC0])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Pin<Gpio25, Output<PushPull>>,
    }

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Tonic = Rp2040Monotonic;

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Soft-reset does not release the hardware spinlocks
        // Release them now to avoid a deadlock after debug or watchdog reset
        // This is normally done in the custom #[entry]
        // Safety: As stated in the docs, this is the first thing that will
        // run in the entry point of the firmware
        unsafe {
            hal::sio::spinlock_reset();
        }

        // Create the RTIC timer
        let mono = Rp2040Monotonic::new(cx.device.TIMER);

        // Grab the global RESETS
        let mut resets = cx.device.RESETS;

        // Setup the clocks
        let mut watchdog = hal::Watchdog::new(cx.device.WATCHDOG);
        let _clocks = hal::clocks::init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            cx.device.XOSC,
            cx.device.CLOCKS,
            cx.device.PLL_SYS,
            cx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // Grab the pins and set them up
        let sio = hal::Sio::new(cx.device.SIO);
        let pins = bsp::Pins::new(
            cx.device.IO_BANK0,
            cx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        let mut led = pins.led.into_push_pull_output();
        led.set_low().unwrap();

        // Spawn the blinking task
        blink::spawn_after(1.secs()).unwrap();

        (Shared {}, Local { led }, init::Monotonics(mono))
    }

    #[task(local = [led])]
    fn blink(cx: blink::Context) {
        info!("Blink");
        let led = cx.local.led;
        led.toggle().unwrap();
        // Schedule self to blink
        blink::spawn_after(1.secs()).unwrap();
    }
}
