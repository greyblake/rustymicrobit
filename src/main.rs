#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::InputPin;
use microbit::board::Board;
use microbit::display::blocking::Display;
use microbit::hal::gpio::Level;
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
    pwm.enable();

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

    // let mut play_note = |pwm: &mut Pwm<_>, note: Note, current_tempo| {
    //     if let Some(pitch) = note.pitch {
    //         let freq = get_pitch_freq(pitch);
    //         pwm.set_period(freq);
    //         pwm.set_duty_on(Channel::C0, max_duty / 9);
    //     } else {
    //         pwm.set_duty_on(Channel::C0, 0);
    //     }
    //     let ms = note_duration_to_ms(note.duration, current_tempo);
    //     timer.delay_ms(ms);
    // };

    //let mut tempo = Bpm(120);
    let mut melody_index = 0;
    loop {
        let pattern = index_to_pattern(melody_index);
        display.show(&mut timer, pattern, 300);

        let melody = &MELODIES[melody_index];
        let tempo = melody.tempo;
        let notes = melody.notes;

        for note in notes {
            // play_note(&mut pwm, *note, tempo);
            // Play note
            {
                if let Some(pitch) = note.pitch {
                    let freq = get_pitch_freq(pitch);
                    pwm.set_period(freq);
                    pwm.set_duty_on(Channel::C0, max_duty / 2);
                } else {
                    pwm.set_duty_on(Channel::C0, 0);
                }
                let ms = note_duration_to_ms(note.duration, tempo);
                timer.delay_ms(ms);
            }

            if button_a.is_low().unwrap() {
                pwm.set_duty_on(Channel::C0, 0);
                melody_index = (melody_index + MELODIES_COUNT - 1) % MELODIES_COUNT;
                break;
            } else if button_b.is_low().unwrap() {
                pwm.set_duty_on(Channel::C0, 0);
                melody_index = (melody_index + 1) % MELODIES_COUNT;
                break;
            }
        }

        //tempo = Bpm(tempo.0 + tempo.0 / 10);

        // for tune in PitchClass::all() {
        //     let freq = calc_frequency(tune);

        //     pwm.set_period(freq);
        //     pwm.set_duty_on(Channel::C0, 0);
        //     display.show(&mut timer, on_pattern, 300);

        //     //timer.delay_ms(500u32);
        //     //pwm.set_period(Hertz(880));
        //     //timer.delay_ms(500u32);

        //     //pwm.disable();
        //     // pwm.set_duty_on(Channel::C0, 0);
        //     // display.show(&mut timer, off_pattern, 100);
        // }
        // pwm.set_duty_on(Channel::C0, 0);

        // display.show(&mut timer, off_pattern, 1000);
    }

    // // Show some simple animation on the display
    // display.show(&mut timer, on_pattern, 500);
    // display.show(&mut timer, off_pattern, 500);

    // Pause beep for 1 second
    // timer.delay(1000u32);

    // -- Stop the beep --
    // pwm.disable();

    // Another quick pattern
    // display.show(&mut timer, on_pattern, 300);
    // display.show(&mut timer, off_pattern, 300);

    // Delay 2 seconds before repeating
    //timer.delay(2000u32);
}

const MELODIES_COUNT: usize = 5;
const MELODIES: [Melody; MELODIES_COUNT] = [
    Melody {
        notes: KINO_PACHKA_SIGARET,
        tempo: Bpm(120),
    },
    Melody {
        notes: PAPA_ROACH_LAST_RESORT,
        tempo: Bpm(120),
    },
    Melody {
        notes: NIRVANA_COME_AS_YOU_ARE,
        tempo: Bpm(120),
    },
    Melody {
        notes: NIRVANA_COME_AS_YOU_ARE_OCTAVE_UP,
        tempo: Bpm(120),
    },
    Melody {
        notes: NIRVANA_COME_AS_YOU_ARE_TWO_OCTAVES_UP,
        tempo: Bpm(120),
    },
];

struct Melody {
    notes: &'static [Note],
    tempo: Bpm,
}

const NIRVANA_COME_AS_YOU_ARE_TWO_OCTAVES_UP: &[Note] = &[
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 8#c3 (C#3)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8f3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 8f3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 8#c3 (C#3)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 8g3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8g3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 8#c3 (C#3)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8f3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 8f3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 8d3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Three,
        }),
    },
    // 8#c3 (C#3)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 8g3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 8c3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Three,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8g3
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Three,
        }),
    },
];

const NIRVANA_COME_AS_YOU_ARE_OCTAVE_UP: &[Note] = &[
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8#c2 (C#2)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8f2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8f2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8#c2 (C#2)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8g2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8g2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8#c2 (C#2)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8f2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8f2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8#c2 (C#2)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8g2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8g2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
];

const NIRVANA_COME_AS_YOU_ARE: &[Note] = &[
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 8#c1 (C#1)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8f1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 8f1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 8#c1 (C#1)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 8g1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8g1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 8#c1 (C#1)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8f1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 8f1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::F,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 8d1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::One,
        }),
    },
    // 8#c1 (C#1)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::CSharp,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 8g1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 8c1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::One,
        }),
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 4- (Quarter rest)
    Note {
        duration: NoteDuration::Quarter,
        pitch: None,
    },
    // 8g1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::One,
        }),
    },
];

const PAPA_ROACH_LAST_RESORT: &[Note] = &[
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16g2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16g2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16d2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8- (rest)
    Note {
        duration: NoteDuration::Eighth,
        pitch: None,
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16g2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16g2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 16a2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::A,
            octave: Octave::Two,
        }),
    },
    // 4d2
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8- (rest)
    Note {
        duration: NoteDuration::Eighth,
        pitch: None,
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 8g2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 16d2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 8d2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16- (rest)
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: None,
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16e2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 16g2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 16g2
    Note {
        duration: NoteDuration::Sixteenth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 8a2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::A,
            octave: Octave::Two,
        }),
    },
    // 4d2
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
];

const KINO_PACHKA_SIGARET: &[Note] = &[
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 8b2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::B,
            octave: Octave::Two,
        }),
    },
    // 8a2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::A,
            octave: Octave::Two,
        }),
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 4c2
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8g2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 4d2
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::D,
            octave: Octave::Two,
        }),
    },
    // 8a2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::A,
            octave: Octave::Two,
        }),
    },
    // 8#f2 (interpreted as F#2)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::FSharp,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 4b1
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::B,
            octave: Octave::One,
        }),
    },
    // 8b2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::B,
            octave: Octave::Two,
        }),
    },
    // 8a2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::A,
            octave: Octave::Two,
        }),
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 4c2
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8g2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::G,
            octave: Octave::Two,
        }),
    },
    // 8e2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::E,
            octave: Octave::Two,
        }),
    },
    // 8b1
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::B,
            octave: Octave::One,
        }),
    },
    // 4c2
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 8a2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::A,
            octave: Octave::Two,
        }),
    },
    // 8#f2 (F#2)
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::FSharp,
            octave: Octave::Two,
        }),
    },
    // 8c2
    Note {
        duration: NoteDuration::Eighth,
        pitch: Some(Pitch {
            note: PitchClass::C,
            octave: Octave::Two,
        }),
    },
    // 4b1
    Note {
        duration: NoteDuration::Quarter,
        pitch: Some(Pitch {
            note: PitchClass::B,
            octave: Octave::One,
        }),
    },
];

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
