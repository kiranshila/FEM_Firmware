//! Log detector functionality using the AD8314 chip

use embedded_hal::adc::OneShot;

struct Ad8314<ADC> {
    adc: ADC,
}
