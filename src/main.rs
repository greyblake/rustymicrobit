#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
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
use heapless::Vec;
use panic_halt as _;

pub mod music;

use music::{
    get_pitch_freq, note_duration_to_ms, Bpm, Melody, Note, NoteDuration, Octave, Pitch, PitchClass,
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
    // 587 = D
    pwm.set_period(Hertz(587));

    // 50% duty cycle
    let max_duty = pwm.max_duty();
    pwm.enable();

    pwm.set_duty_on(Channel::C0, max_duty / 9);

    // -----------------------------------------
    // 3) Set up the LED matrix display & a timer for delays
    // -----------------------------------------
    let mut display = Display::new(board.display_pins);
    let mut timer = Timer::new(board.TIMER0);

    let button_a = board.buttons.button_a;
    let button_b = board.buttons.button_b;

    let mut play_note = |pwm: &mut Pwm<_>,
                         timer: &mut Timer<TIMER0>,
                         display: &mut Display,
                         note: Note,
                         current_tempo: Bpm,
                         pattern: Pattern| {
        if let Some(pitch) = note.pitch {
            let freq = get_pitch_freq(pitch);
            pwm.set_period(freq);
            pwm.set_duty_on(Channel::C0, max_duty / 2);
        } else {
            pwm.set_duty_on(Channel::C0, 0);
        }
        let ms = note_duration_to_ms(note.duration, current_tempo);
        //timer.delay_ms(ms);
        display.show(timer, pattern, ms);
    };

    let mut play_melody = |pwm: &mut Pwm<_>,
                           timer: &mut Timer<TIMER0>,
                           display: &mut Display,
                           melody: &Melody,
                           pattern: Pattern| {
        for note in melody.notes {
            play_note(pwm, timer, display, *note, melody.tempo, pattern);
        }
    };

    let mut game = Game::new();

    loop {
        let pattern = render_game(&game);
        display.show(&mut timer, pattern, 120);
        pwm.set_duty_on(Channel::C0, 0);

        let mut sound = game.tick();

        let action = read_action(&button_a, &button_b);
        if let Some(action) = action {
            let maybe_another_sound = game.apply_player_action(action);
            if maybe_another_sound.is_some() {
                sound = maybe_another_sound;
            }
        }

        match sound {
            Some(Effect::SoundShoot) => {
                pwm.set_period(Hertz(659));
                pwm.set_duty_on(Channel::C0, max_duty / 2);
            }
            Some(Effect::SoundHit) => {
                pwm.set_period(Hertz(440));
                pwm.set_duty_on(Channel::C0, max_duty / 2);
            }
            Some(Effect::GameOver) => {
                play_melody(
                    &mut pwm,
                    &mut timer,
                    &mut display,
                    &MELODY_GAME_OVER,
                    PATTERN_SAD_FACE,
                );
                game = Game::new();
            }
            None => {}
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Left,
    Right,
    Shoot,
}

fn read_action(
    button_a: &P0_14<Input<Floating>>,
    button_b: &P0_23<Input<Floating>>,
) -> Option<Action> {
    let mut is_a = false;
    let mut is_b = false;

    if button_a.is_low().unwrap() {
        is_a = true;
    }
    if button_b.is_low().unwrap() {
        is_b = true;
    }

    match (is_a, is_b) {
        (false, false) => None,
        (true, true) => Some(Action::Shoot),
        (false, true) => Some(Action::Right),
        (true, false) => Some(Action::Left),
    }
}

fn render_game(game: &Game) -> Pattern {
    let mut pattern = [[0; 5]; 5];

    pattern[game.player.y][game.player.x] = 1;

    for enemy in game.enemies.iter() {
        pattern[enemy.y][enemy.x] = 1;
    }

    if let Some(bullet) = game.bullet {
        pattern[bullet.y][bullet.x] = 1;
    }

    pattern
}

#[derive(Clone, Copy, Debug)]
struct Pos {
    x: usize,
    y: usize,
}

struct Game {
    player: Pos,
    enemies: Vec<Pos, 25>,
    bullet: Option<Pos>,
    counter: usize,
    noise: usize,
}

impl Game {
    fn new() -> Self {
        Game {
            player: Pos { x: 2, y: 4 },
            enemies: Vec::new(),
            bullet: None,
            counter: 0,
            noise: 123,
        }
    }

    fn apply_player_action(&mut self, action: Action) -> Option<Effect> {
        let mut sound = None;
        match action {
            Action::Left => {
                if self.player.x > 0 {
                    self.player.x = self.player.x - 1;
                }
                self.noise = self.noise + 1;
            }
            Action::Right => {
                if self.player.x < 4 {
                    self.player.x = self.player.x + 1;
                }
                self.noise = self.noise + 5;
            }
            Action::Shoot => {
                if self.bullet.is_none() {
                    self.bullet = Some(Pos {
                        x: self.player.x,
                        y: self.player.y,
                    });
                }
                self.noise = self.noise + 13;
                sound = Some(Effect::SoundShoot);
            }
        }
        sound
    }

    fn tick(&mut self) -> Option<Effect> {
        self.counter = self.counter + 1;
        let mut sound = None;

        if let Some(bullet) = self.bullet {
            if bullet.y == 0 {
                self.bullet = None;
            } else {
                let y = bullet.y - 1;
                let x = bullet.x;
                self.bullet = Some(Pos { x, y });

                for enemy in self.enemies.iter() {
                    if enemy.x == x && enemy.y == y {
                        self.enemies.retain(|e| e.x != x || e.y != y);
                        self.bullet = None;
                        sound = Some(Effect::SoundHit);
                        break;
                    }
                }
            }
        }

        // Generate a new enemy every 49 ticks
        if self.counter % 19 == 0 {
            let x = (self.counter + self.noise) % 5;
            let _old = self.enemies.push(Pos { x: x, y: 0 });
        }

        // Move enemies down
        if self.counter % 49 == 0 {
            for enemy in self.enemies.iter_mut() {
                enemy.y = enemy.y + 1;
                if enemy.y >= 4 {
                    return Some(Effect::GameOver);
                }
            }
        }

        sound
    }
}

enum Effect {
    SoundShoot,
    SoundHit,
    GameOver,
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

const PATTERN_BLANK: [[u8; 5]; 5] = [[0; 5]; 5];
const PATTERN_FULL: [[u8; 5]; 5] = [[1; 5]; 5];

const PATTERN_SAD_FACE: Pattern = [
    [0, 0, 0, 0, 0],
    [0, 1, 0, 1, 0],
    [0, 0, 0, 0, 0],
    [0, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
];

const MELODY_GAME_OVER: Melody = Melody {
    tempo: Bpm(120),
    notes: &[
        Note {
            duration: NoteDuration::Quarter,
            pitch: Some(Pitch {
                note: PitchClass::D,
                octave: Octave::Two,
            }),
        },
        Note {
            duration: NoteDuration::Quarter,
            pitch: Some(Pitch {
                note: PitchClass::CSharp,
                octave: Octave::Two,
            }),
        },
        Note {
            duration: NoteDuration::Quarter,
            pitch: Some(Pitch {
                note: PitchClass::C,
                octave: Octave::Two,
            }),
        },
        Note {
            duration: NoteDuration::Half,
            pitch: Some(Pitch {
                note: PitchClass::B,
                octave: Octave::One,
            }),
        },
        // Note {
        //     duration: NoteDuration::Whole,
        //     pitch: Some(Pitch {
        //         note: PitchClass::ASharp,
        //         octave: Octave::One,
        //     }),
        // },
    ],
};
