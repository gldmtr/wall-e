use embedded_hal::blocking::delay::{DelayUs, DelayMs};
use shared_bus::{BusMutex, NullMutex};

pub struct DelayManager<'a> {
    mutex: NullMutex<&'a mut dyn DelayUs<u32>>,
}

pub struct SharedDelay<'a> {
    mutex: &'a NullMutex<&'a mut dyn DelayUs<u32>>,
}

impl<'a> DelayManager<'a> {
    pub fn new(delay: &'a mut dyn DelayUs<u32>) -> Self {
        let mutex = NullMutex::create(delay);

        return Self{mutex};
    }

    pub fn get(&'a self) -> SharedDelay<'a> {
        return SharedDelay{mutex: &self.mutex}
    }
}

impl<'a, W> DelayUs<W> for SharedDelay<'a> where W: Into<u32> {
    fn delay_us(&mut self, us: W) {
        self.mutex.lock(|delay| delay.delay_us(us.into()))
    }
}

impl<'a, W> DelayMs<W> for SharedDelay<'a> where W: Into<u32> {
    fn delay_ms(&mut self, us: W) {
        self.mutex.lock(|delay| delay.delay_us(us.into() * 1000))
    }
}