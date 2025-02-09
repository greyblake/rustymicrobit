use microbit::hal::time::Hertz;

#[derive(Clone, Copy)]
pub struct Note {
    pub duration: NoteDuration,

    /// The pitch of the note, if any.
    /// None means a pause.
    pub pitch: Option<Pitch>,
}

#[derive(Clone, Copy)]
pub enum NoteDuration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

#[derive(Clone, Copy)]
pub struct Pitch {
    pub note: PitchClass,
    pub octave: Octave,
}

#[derive(Clone, Copy)]
pub enum Octave {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy)]
pub enum PitchClass {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

// Frequency for octave 4
pub fn get_frequency_for_octave4(note: PitchClass) -> Hertz {
    let freq = match note {
        PitchClass::C => 2093,
        PitchClass::CSharp => 2217,
        PitchClass::D => 2349,
        PitchClass::DSharp => 2489,
        PitchClass::E => 2637,
        PitchClass::F => 2794,
        PitchClass::FSharp => 2960,
        PitchClass::G => 3136,
        PitchClass::GSharp => 3322,
        PitchClass::A => 3520,
        PitchClass::ASharp => 3729,
        PitchClass::B => 3951,
    };
    Hertz(freq)
}

pub fn get_pitch_freq(pitch: Pitch) -> Hertz {
    let octave4_freq = get_frequency_for_octave4(pitch.note).0;

    let freq = match pitch.octave {
        Octave::Four => octave4_freq,
        Octave::Three => octave4_freq / 2,
        Octave::Two => octave4_freq / 4,
        Octave::One => octave4_freq / 8,
    };

    Hertz(freq)
}

/// Tempo
#[derive(Clone, Copy)]
pub struct Bpm(pub u32);

pub fn note_duration_to_ms(duration: NoteDuration, tempo: Bpm) -> u32 {
    let ms_per_beat = 60_000 / tempo.0;

    let ms = match duration {
        NoteDuration::Whole => ms_per_beat * 4,
        NoteDuration::Half => ms_per_beat * 2,
        NoteDuration::Quarter => ms_per_beat,
        NoteDuration::Eighth => ms_per_beat / 2,
        NoteDuration::Sixteenth => ms_per_beat / 4,
    };

    ms
}

pub struct Melody {
    pub notes: &'static [Note],
    pub tempo: Bpm,
}
