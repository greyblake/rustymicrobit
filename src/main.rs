#![no_std]
#![no_main]

use cortex_m_rt::entry;
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::Timer;
use microbit::hal::prelude::*;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    let mut display = Display::new(board.display_pins);
    let mut timer = Timer::new(board.TIMER0);

    // A simple "all on" pattern
    let on_pattern = [[1; 5]; 5];

    // All off
    let off_pattern = [[0; 5]; 5];

    loop {
        // Intro
        display.show(&mut timer, PATTERN_1, 300);
        display.show(&mut timer, PATTERN_2, 300);
        display.show(&mut timer, on_pattern, 300);
        display.show(&mut timer, PATTERN_2, 300);
        display.show(&mut timer, on_pattern, 500);
        display.show(&mut timer, off_pattern, 500);

        // HEART
        for _ in 0..5 {
            display.show(&mut timer, PATTERN_HEART, 500);
            display.show(&mut timer, off_pattern, 500);
        }

        display.show(&mut timer, LETTER_A, 800);
        display.show(&mut timer, off_pattern, 500);

        display.show(&mut timer, LETTER_H, 800);
        display.show(&mut timer, off_pattern, 500);

        display.show(&mut timer, LETTER_T, 800);
        display.show(&mut timer, off_pattern, 500);

        display.show(&mut timer, LETTER_O, 800);
        display.show(&mut timer, off_pattern, 500);

        display.show(&mut timer, LETTER_H, 800);
        display.show(&mut timer, off_pattern, 500);
    }
}

type Pattern = [[u8; 5]; 5];

/// A 5x5 representation of the letter 'A'
const LETTER_A: Pattern = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [1, 1, 1, 1, 1],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
];

const LETTER_H: Pattern = [
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [1, 1, 1, 1, 1],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
];

const LETTER_T: Pattern = [
    [1, 1, 1, 1, 1],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
];

const LETTER_O: Pattern = [
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0],
];

const PATTERN_1: Pattern = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

const PATTERN_2: Pattern = [
    [0, 0, 0, 0, 0],
    [0, 1, 1, 1, 0],
    [0, 1, 1, 1, 0],
    [0, 1, 1, 1, 0],
    [0, 0, 0, 0, 0],
];

const PATTERN_HEART: Pattern = [
    [0, 1, 0, 1, 0],
    [1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
    [0, 1, 1, 1, 0],
    [0, 0, 1, 0, 0],
];
