use core::fmt;

use embedded_hal::blocking::spi::Transfer;

pub struct PS2Controller<SPI>
where
    SPI: Transfer<u8>
{
    device: SPI,
    state: [u8; 10],
}

const POLL_CMD: [u8; 10] = [0x01, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

#[derive(Debug, defmt::Format)]
pub struct StickPosition(pub u8, pub u8);

pub struct StickPositions(pub StickPosition, pub StickPosition);

impl<SPI> PS2Controller<SPI>
where 
    SPI: Transfer<u8, Error: fmt::Debug>,
{
    pub fn new(device: SPI) -> Self {
        Self { device, state: [0; 10] }
    }

    pub fn read_state(&mut self) {
        let mut cmd = POLL_CMD;

        let result = self.device.transfer(&mut cmd).unwrap();
        
        for (i, byte) in self.state.iter_mut().enumerate() {
            *byte = result[i]
        }
        // defmt::debug!("sent: {=[u8]:#X}, received: {=[u8]:#X}", POLL_CMD, result);
    }

    pub fn read_sticks(&self) -> StickPositions {
        let right_x = self.state[5];
        let right_y = self.state[6];

        let left_x = self.state[7];
        let left_y = self.state[8];

        StickPositions(
            StickPosition(left_x, left_y),
            StickPosition(right_x, right_y),
        )
    }
}