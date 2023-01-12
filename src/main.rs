#![no_std]
#![no_main]

mod bsp;
mod log_det;

use bsp::*;
use defmt::*;
use defmt_rtt as _;
use hal::pac;
use panic_probe as _;
use rp2040_hal as hal;
use rp2040_monotonic::{ExtU64, Rp2040Monotonic};

use hal::adc::{Adc, TempSense};
use hal::gpio::{bank0::Gpio21, Output, Pin, PushPull};

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
        rf1_status_led: Rf1StatusLed,
        rf2_status_led: Rf2StatusLed,
        lna_1: Rf1LnaEn,
        lna_2: Rf2LnaEn,
        rf1_if_pow: Rf1IfPow,
        rf2_if_pow: Rf2IfPow,
        temp_sense: TempSense,
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

        // Enable the ADC peripheral and internal temperature sensor
        let mut adc = Adc::new(cx.device.ADC, &mut resets);
        let temp_sense = adc.enable_temp_sensor();
        let rf1_if_pow = pins.rf1_if_pow.into_floating_input();
        let rf2_if_pow = pins.rf2_if_pow.into_floating_input();

        // Set the LNA outputs to ON by default
        let mut lna_1 = pins.rf1_lna_en.into_push_pull_output();
        let mut lna_2 = pins.rf2_lna_en.into_push_pull_output();
        lna_1.set_high().unwrap();
        lna_2.set_high().unwrap();

        // Set the RF status LEDs to off
        let mut rf1_status_led = pins.rf1_status_led.into_push_pull_output();
        let mut rf2_status_led = pins.rf2_status_led.into_push_pull_output();
        rf1_status_led.set_low().unwrap();
        rf2_status_led.set_low().unwrap();

        (
            Shared {},
            Local {
                lna_1,
                lna_2,
                rf1_status_led,
                rf2_status_led,
                rf1_if_pow,
                rf2_if_pow,
                temp_sense,
            },
            init::Monotonics(mono),
        )
    }
}
