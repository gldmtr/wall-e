use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use shared_bus::{BusMutex, NullMutex};

pub struct DelayManager<D: DelayUs<u32>> {
    mutex: NullMutex<D>,
}

pub struct SharedDelay<'a, D: DelayUs<u32>> {
    mutex: &'a NullMutex<D>,
}

impl<'a, D: DelayUs<u32>> DelayManager<D> {
    pub fn new(delay: D) -> Self {
        let mutex = NullMutex::create(delay);

        return Self { mutex };
    }

    pub fn get(&'a self) -> SharedDelay<'a, D> {
        return SharedDelay { mutex: &self.mutex };
    }
}

impl<'a, W, D> DelayUs<W> for SharedDelay<'a, D>
where
    W: Into<u32>,
    D: DelayUs<u32>,
{
    fn delay_us(&mut self, us: W) {
        self.mutex.lock(|delay| delay.delay_us(us.into()))
    }
}

impl<'a, W, D> DelayMs<W> for SharedDelay<'a, D>
where
    W: Into<u32>,
    D: DelayUs<u32>,
{
    fn delay_ms(&mut self, us: W) {
        self.mutex.lock(|delay| delay.delay_us(us.into() * 1000))
    }
}
