#![no_std]
#![no_main]

use cortex_m_rt::entry;
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::gpio::Level;
use microbit::hal::prelude::*;
use microbit::hal::pwm::{Pwm, Channel};
use microbit::hal::time::Hertz;
use microbit::hal::Timer;
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Take ownership of the Board (Peripherals + pins, etc.)
    let mut board = Board::take().unwrap();

    // -----------------------------------------
    // 1) Enable the speaker by driving P0.01 high.
    //
    // The Board definition shows:
    //   speaker_pin: p0::P0_00<Disconnected>
    //   p0_01: p0::P0_01<Disconnected>  // among 'Pins'
    //
    // So we turn p0_01 into a push-pull output, level High:
    // -----------------------------------------
    let _speaker_enable_pin = board.pins.p0_01.into_push_pull_output(Level::High);

    // -----------------------------------------
    // 2) Set up PWM on the speaker pin (P0.00).
    // -----------------------------------------
    let speaker_pin = board.speaker_pin.into_push_pull_output(Level::Low);
    let mut pwm = Pwm::new(board.PWM0);

    // Route PWM channel 0 to our speaker pin:
    pwm.set_output_pin(Channel::C0, speaker_pin.degrade());

    // Example beep frequency: 440 Hz
    pwm.set_period(Hertz(440));

    // 50% duty cycle
    let max_duty = pwm.max_duty();
    pwm.set_duty_on(Channel::C0, max_duty / 2);

    // -----------------------------------------
    // 3) Set up the LED matrix display & a timer for delays
    // -----------------------------------------
    let mut display = Display::new(board.display_pins);
    let mut timer = Timer::new(board.TIMER0);

    // Patterns to use on the 5x5 LED matrix
    let on_pattern = [[1; 5]; 5];
    let off_pattern = [[0; 5]; 5];

    // -----------------------------------------
    // 4) Main loop
    // -----------------------------------------
    loop {
        // -- Turn on PWM to start beeping --
        pwm.enable();

        // Show some simple animation on the display
        display.show(&mut timer, on_pattern, 500);
        display.show(&mut timer, off_pattern, 500);

        // Pause beep for 1 second
        timer.delay(1000u32);

        // -- Stop the beep --
        pwm.disable();

        // Another quick pattern
        display.show(&mut timer, on_pattern, 300);
        display.show(&mut timer, off_pattern, 300);

        // Delay 2 seconds before repeating
        timer.delay(2000u32);
    }
}
