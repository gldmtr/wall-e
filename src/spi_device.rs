use core::fmt;

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

pub struct SpiDevice<SPI, CS, Delay> {
    cs: CS,
    spi: SPI,
    delay: Delay,

    read_delay: u8,
}

impl<SPI, CS, Delay> SpiDevice<SPI, CS, Delay>
where
    SPI: spi::Transfer<u8>,
    CS: OutputPin,
    Delay: DelayUs<u8>,
{
    pub fn new(spi: SPI, cs: CS, delay: Delay) -> Self {
        return SpiDevice {
            cs,
            spi,
            delay,
            read_delay: 0,
        };
    }

    pub fn set_read_delay_us(&mut self, delay: u8) {
        self.read_delay = delay;
    }
}

impl<SPI, CS, Delay> spi::Transfer<u8> for SpiDevice<SPI, CS, Delay>
where
    SPI: spi::Transfer<u8, Error: fmt::Debug>,
    CS: OutputPin<Error: fmt::Debug>,
    Delay: DelayUs<u8>,
{
    type Error = ();

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.cs.set_low().unwrap();

        if self.read_delay > 0 {
            self.delay.delay_us(self.read_delay);
        }

        for byte in words.iter_mut() {
            let mut packet = [*byte];
            let res = self.spi.transfer(&mut packet).unwrap();
            *byte = res[0];

            self.delay.delay_us(self.read_delay);
        }

        self.cs.set_high().unwrap();

        Ok(words)
    }
}
