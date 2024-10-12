use core::fmt;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;

pub enum Direction {
    Forward,
    Backward,
    Stop,
}

pub trait Motor {
    fn set_power(&mut self, power: f32);
    fn set_direction(&mut self, direction: Direction);
}

pub struct PWMMotor<Enable: PwmPin, Pin1: OutputPin, Pin2: OutputPin> {
    enable: Enable,
    pin1: Pin1,
    pin2: Pin2,
}

impl<Enable: PwmPin<Duty = u32>, Pin1: OutputPin, Pin2: OutputPin> PWMMotor<Enable, Pin1, Pin2> {
    pub fn new(mut enable: Enable, pin1: Pin1, pin2: Pin2) -> Self {
        enable.enable();
        enable.set_duty(0);

        Self { enable, pin1, pin2 }
    }
}

impl<Enable, Pin1, Pin2> Motor for PWMMotor<Enable, Pin1, Pin2>
where
    Enable: PwmPin<Duty = u32>,
    Pin1: OutputPin<Error: fmt::Debug>,
    Pin2: OutputPin<Error: fmt::Debug>,
{
    fn set_power(&mut self, power: f32) {
        let max_duty = self.enable.get_max_duty();
        let duty = libm::floorf(max_duty as f32 * power.min(100.0) / 100.0);

        defmt::debug!(
            "Set PWM duty cycle of {}/{}, power = {}",
            duty,
            max_duty,
            power
        );

        match power {
            0.0 => self.set_direction(Direction::Stop),
            x if x > 0.0 => self.set_direction(Direction::Forward),
            x if x < 0.0 => self.set_direction(Direction::Backward),
            _ => unreachable!(),
        }

        self.enable.set_duty(duty as u32);
    }

    fn set_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Forward => {
                self.pin1.set_high().unwrap();
                self.pin2.set_low().unwrap();
            }
            Direction::Backward => {
                self.pin1.set_low().unwrap();
                self.pin2.set_high().unwrap();
            }
            Direction::Stop => {
                self.pin1.set_low().unwrap();
                self.pin1.set_low().unwrap();
            }
        }
    }
}
