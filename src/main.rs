#![no_std]
#![no_main]

mod chassis_controller;
mod controller;
mod motor;
mod shared_delay;
mod spi_device;

use chassis_controller::ChassisController;
use controller::PS2Controller;
use defmt_rtt as _;
use hal::hal::spi::MODE_3;

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use stm32f3xx_hal::{
    self as hal,
    hal::blocking::delay::DelayMs,
    pac,
    prelude::*,
    pwm::tim2,
    spi::{config, Spi},
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(8.MHz())
        .pclk1(8.MHz())
        .freeze(&mut flash.acr);
    defmt::info!("init");

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    // let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);

    // Configure I2C1
    // let mut scl =
    //     gpiob
    //         .pb6
    //         .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    // let mut sda =
    //     gpiob
    //         .pb7
    //         .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    // scl.internal_pull_up(&mut gpiob.pupdr, true);
    // sda.internal_pull_up(&mut gpiob.pupdr, true);

    // let mut i2c = hal::i2c::I2c::new(dp.I2C1, (scl, sda), 10_000.Hz(), clocks, &mut rcc.apb1);

    // defmt::info!("Start i2c connect...");

    // let address = Address::default();
    // let address = 0x40;
    // let mut pwm = Pca9685::new(i2c, address).unwrap();
    // pwm.set_prescale(100).unwrap();
    // pwm.enable().unwrap();
    // pwm.set_channel_on_off(pwm_pca9685::Channel::All, 0, 2048)
    //     .unwrap();
    // pwm.set_channel_off(pwm_pca9685::Channel::C0, 2048).unwrap();
    // pwm.set_channel_full_on(channel, value)
    // do something...

    // get the I2C device back
    // let dev = pwm.destroy();

    let sck = gpioa
        .pa5
        .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let miso = gpioa
        .pa6
        .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mosi = gpioa
        .pa7
        .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mut ps_cs = gpiob
        .pb0
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut sd_cs = gpiob
        .pb1
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    ps_cs.set_high().unwrap();
    sd_cs.set_high().unwrap();

    let mut spi = Spi::<_, _, u8>::new(
        dp.SPI1,
        (sck, miso, mosi),
        config::Config::default().frequency(250.kHz()).mode(MODE_3),
        clocks,
        &mut rcc.apb2,
    );
    unsafe {
        spi.peripheral().cr1.modify(|_, w| w.lsbfirst().set_bit());
    }

    let harware_delay = hal::delay::Delay::new(cp.SYST, clocks);
    let delay = shared_delay::DelayManager::new(harware_delay);

    let bus = shared_bus::BusManagerSimple::new(spi);
    let mut dev = spi_device::SpiDevice::new(bus.acquire_spi(), ps_cs, delay.get());
    dev.set_read_delay_us(50);
    let mut controller = PS2Controller::new(dev);

    let tim2 = tim2(dp.TIM2, 100, 50.Hz(), &clocks);
    let pa1 = gpioa
        .pa1
        .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let m11 = gpioa
        .pa2
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let m12 = gpioa
        .pa3
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let me1 = tim2.1.output_to_pa1(pa1);
    let left_motor = motor::PWMMotor::new(me1, m11, m12);

    let pa0 = gpioa
        .pa0
        .into_af_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let m21 = gpioa
        .pa9
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let m22 = gpioa
        .pa10
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    let me2 = tim2.0.output_to_pa0(pa0);
    let right_motor = motor::PWMMotor::new(me2, m21, m22);

    let mut chassis_controller = ChassisController::new(left_motor, right_motor);

    loop {
        controller.read_state();

        delay.get().delay_ms(50 as u8);

        let stick_positions = controller.read_sticks();
        defmt::info!(
            "stick positions: left: {:?} right {:?}",
            stick_positions.0,
            stick_positions.1
        );

        chassis_controller.process_input(stick_positions.0);
    }
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
