#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::InputPin;
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::gpio::p0::{P0_14, P0_23};
use microbit::hal::gpio::{Floating, Input, Level};
use microbit::hal::prelude::*;
use microbit::hal::pwm::{Channel, Pwm};
use microbit::hal::time::Hertz;
use microbit::hal::Timer;
use microbit::pac::TIMER0;
//use microbit::hal::PWM0;
//
use panic_halt as _;

pub mod music;

use music::{
    get_pitch_freq, note_duration_to_ms, Bpm, Note, NoteDuration, Octave, Pitch, PitchClass,
};

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
    pwm.set_period(Hertz(600));

    // 50% duty cycle
    let max_duty = pwm.max_duty();
    //pwm.enable();

    //pwm.set_duty_on(Channel::C0, max_duty / 2);

    // -----------------------------------------
    // 3) Set up the LED matrix display & a timer for delays
    // -----------------------------------------
    let mut display = Display::new(board.display_pins);
    let mut timer = Timer::new(board.TIMER0);

    // Patterns to use on the 5x5 LED matrix
    let on_pattern = [[1; 5]; 5];
    let off_pattern = [[0; 5]; 5];

    let button_a = board.buttons.button_a;
    let button_b = board.buttons.button_b;


    let mut current_pos = Pos {
        x: 2,
        y: 4,
    };

    loop {
        let pattern = render_pattern(current_pos);
        display.show(&mut timer, pattern, 100);

        let action = read_action(&button_a, &button_b);
        if let Some(action) = action {
            current_pos = change_position(current_pos, action);
        }
    }
}


fn change_position(
    pos: Pos,
    action: Action,
) -> Pos {

    match action {
        Action::Left => {
            if pos.x > 0 {
                Pos { x: pos.x - 1, y: pos.y }
            } else {
                pos
            }
        }
        Action::Right => {
            if pos.x < 4 {
                Pos { x: pos.x + 1, y: pos.y }
            } else {
                pos
            }
        }
        Action::Up => {
            if pos.y > 0 {
                Pos { x: pos.x, y: pos.y - 1 }
            } else {
                pos
            }
        }
    }
}

enum Action {
    Left,
    Right,
    Up,
}

fn read_action(
    button_a: &P0_14<Input<Floating>>,
    button_b: &P0_23<Input<Floating>>,
) -> Option<Action> {
    let mut is_a = false;
    let mut is_b = false;

    if button_a.is_low().unwrap() {
        is_a = true;
    } else if button_b.is_low().unwrap() {
        is_b = true;
    }

    match (is_a, is_b) {
        (false, false) => None,
        (true, true) => Some(Action::Up),
        (false, true) => Some(Action::Right),
        (true, false) => Some(Action::Left),
    }
}


#[derive(Clone, Copy)]
struct Pos {
    x: usize,
    y: usize,
}

fn render_pattern(pos: Pos) -> Pattern {
    let mut pattern = [[0; 5]; 5];

    pattern[pos.y][pos.x] = 1;

    pattern
}


type Pattern = [[u8; 5]; 5];

// 0
const PATTERN_0: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
];

// 1
const PATTERN_1: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
];

// 2
const PATTERN_2: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 0, 0, 1, 0],
    [0, 0, 1, 0, 0],
    [0, 1, 1, 1, 1],
];

// 3
const PATTERN_3: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 0, 0, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
];

// 4
const PATTERN_4: [[u8; 5]; 5] = [
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
    [0, 0, 0, 0, 1],
    [0, 0, 0, 0, 1],
];

// 5
const PATTERN_5: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 0],
    [0, 1, 1, 1, 0],
    [0, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
];

// 6
const PATTERN_6: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 0],
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
];

// 7
const PATTERN_7: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 1],
    [0, 0, 0, 0, 1],
    [0, 0, 0, 1, 0],
    [0, 0, 1, 0, 0],
    [0, 1, 0, 0, 0],
];

// 8
const PATTERN_8: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
];

// 9
const PATTERN_9: [[u8; 5]; 5] = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
    [0, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
];

fn index_to_pattern(index: usize) -> Pattern {
    match index {
        0 => PATTERN_0,
        1 => PATTERN_1,
        2 => PATTERN_2,
        3 => PATTERN_3,
        4 => PATTERN_4,
        5 => PATTERN_5,
        6 => PATTERN_6,
        7 => PATTERN_7,
        8 => PATTERN_8,
        9 => PATTERN_9,
        _ => PATTERN_0,
    }
}
