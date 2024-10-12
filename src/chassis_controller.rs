use crate::controller::StickPosition;
use crate::motor::Motor;

pub struct ChassisController<LeftMotor: Motor, RightMotor: Motor> {
    left_motor: LeftMotor,
    right_motor: RightMotor,
}

fn map(x: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

const FRAC_PI_4: f64 = 0.785398163397448309615660845819875721_f64;

impl<LeftMotor: Motor, RightMotor: Motor> ChassisController<LeftMotor, RightMotor> {
    pub fn new(left_motor: LeftMotor, right_motor: RightMotor) -> Self {
        Self {
            left_motor,
            right_motor,
        }
    }

    pub fn process_input(&mut self, stick_position: StickPosition) {
        let (left, right) = self.stick_to_motor_power(stick_position);

        self.left_motor.set_power(left);
        self.right_motor.set_power(right);
    }

    fn stick_to_motor_power(&self, stick_position: StickPosition) -> (f32, f32) {
        let x = i32::from(stick_position.0);
        let normalized_x = if x < 127 {
            map(x, 0, 127, -100, 0)
        } else if x == 127 {
            0
        } else {
            map(x, 127, 255, 0, 100)
        };

        let y = i32::from(stick_position.1);
        let normalized_y = if y < 128 {
            100 - map(y, 0, 128, 0, 100)
        } else if y == 128 {
            0
        } else {
            -map(y, 128, 255, 0, 100)
        };

        let r = libm::hypotf(normalized_x as f32, normalized_y as f32);
        let mut t = libm::atan2f(normalized_y as f32, normalized_x as f32);

        t -= FRAC_PI_4 as f32;

        let mut left = r * libm::cosf(t);
        let mut right = r * libm::sinf(t);

        left = left * libm::sqrtf(2 as f32);
        right = right * libm::sqrtf(2 as f32);

        left = left.min(100 as f32).max(-100 as f32);
        right = right.min(100 as f32).max(-100 as f32);

        defmt::debug!(
            "normalized stick positions: x={:?}, y={:?}",
            normalized_x,
            normalized_y
        );
        defmt::debug!("motor powers: left={:?}, RightMotor={:?}", left, right);

        (left, right)
    }
}
