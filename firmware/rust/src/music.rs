use embedded_hal::delay::DelayNs;

// Note constants
pub const CN: u8 = 0; // C  (C normal)
pub const CS: u8 = 1; // C# (C sharp)
pub const DF: u8 = CS; // Db (D flat)
pub const DN: u8 = 2; // D  (D normal)
pub const DS: u8 = 3; // D# (D sharp)
pub const EF: u8 = DS; // Eb (E flat)
pub const EN: u8 = 4; // E  (E normal)
pub const FN: u8 = 5; // F  (F normal)
pub const FS: u8 = 6; // F# (F sharp)
pub const GF: u8 = FS; // Gb (G flat)
pub const GN: u8 = 7; // G  (G normal)
pub const GS: u8 = 8; // G# (G sharp)
pub const AF: u8 = GS; // Ab (A flat)
pub const AN: u8 = 9; // A  (A normal)
pub const AS: u8 = 10; // A# (A sharp)
pub const BF: u8 = AS; // Bb (B flat)
pub const BN: u8 = 11; // B  (B normal)

pub const NOTE_COUNT: u8 = 12;
pub const OCTAVE_MASK: u16 = 0xF000;
pub const NOTE_MASK: u16 = 0x0F00;
pub const TIMING_MASK: u16 = 0x00FF;
pub const SILENCE: u8 = 0xF;

pub const A4_FREQUENCY: f32 = 440.0;
pub const MIN_OCTAVE: u8 = 1; // actually 0 but I want 8 steps and normal piezo buzzers def can't play that low
pub const MAX_OCTAVE: u8 = 8;

// Macro to create a note
pub const fn n(note: u8, octave: u8, timing: u8) -> u16 {
    ((octave as u16) << 12) | ((note as u16) << 8) | (timing as u16)
}

// Macro to create a pause
pub const fn pause(timing: u8) -> u16 {
    n(SILENCE, SILENCE, timing)
}

pub const END: u16 = 0;

pub const NOTES: [&str; 12] = [
    "C", "Cs", "D", "Ds", "E", "F", "Fs", "G", "Gs", "A", "As", "B",
];

pub fn get_note_frequency(note: u8, octave: u8) -> u32 {
    // includes defined SILENCE value for note and octave
    if note > BN || octave > MAX_OCTAVE {
        return 0;
    }

    let note = if note < CN {
        CN
    } else if note > BN {
        BN
    } else {
        note
    };
    let semitone_distance = (note as i32) - (AN as i32) + 12 * ((octave as i32) - 4);
    // Precalculated frequencies for semitone distances from A4 (index 0 = A4, -48..+48)
    const FREQUENCY_TABLE: [u32; 97] = [
        27, 29, 31, 33, 35, 37, 39, 41, 44, 46, 49, 52, // -48..-37
        55, 58, 62, 65, 69, 73, 78, 82, 87, 93, 98, 104, // -36..-25
        110, 117, 123, 131, 139, 147, 156, 165, 175, 185, 196, 208, // -24..-13
        220, 233, 247, 262, 277, 294, 311, 330, 349, 370, 392, 415, // -12..-1
        440, 466, 494, 523, 554, 587, 622, 659, 698, 740, 784, 831, // 0..11
        880, 932, 988, 1047, 1109, 1175, 1245, 1319, 1397, 1480, 1568, 1661, // 12..23
        1760, 1865, 1976, 2093, 2217, 2349, 2489, 2637, 2794, 2960, 3136, 3322, // 24..35
        3520, 3729, 3951, 4186, 4435, 4699, 4978, 5274, 5588, 5920, 6272, 6645, // 36..47
        7040, // 48
    ];
    let table_offset = 48; // so A4 (semitone_distance 0) is at index 48
    let idx = (semitone_distance + table_offset) as isize;
    if idx < 0 || idx as usize >= FREQUENCY_TABLE.len() {
        0
    } else {
        FREQUENCY_TABLE[idx as usize]
    }
}

// Play a single note (or rest)
pub fn play_note(
    buzzer: &mut crate::tone::Tone,
    delay: &mut crate::Delay,
    note_index: u8,
    octave: u8,
    timing: u8,
    timing_unit_ms: u32,
    octave_adjust: i8,
) {
    let octave = (octave as i8 + octave_adjust) as u8;
    let frequency = get_note_frequency(note_index, octave);
    if frequency > 0 {
        buzzer.tone(frequency, timing as u32 * timing_unit_ms);
    } else {
        buzzer.no_tone();
    }
    delay.delay_ms(timing as u32 * timing_unit_ms);
    buzzer.no_tone();
    delay.delay_ms(timing_unit_ms >> 1);
}

// Play a song (blocking)
pub fn play_song(buzzer: &mut crate::tone::Tone, display: &mut crate::Display) {
    // These could be parameters or settings
    let timing_unit_ms: u32 = 15; // Default tempo
    let octave_adjust: i8 = 3;
    let song = &SUPER_MARIO;
    let mut delay = crate::Delay::new();
    let mut note_index = 0;
    loop {
        let note = song[note_index];
        if note == END {
            break;
        }
        let octave = ((note & OCTAVE_MASK) >> 12) as u8;
        let index = ((note & NOTE_MASK) >> 8) as u8;
        let timing = (note & TIMING_MASK) as u8;
        if index < NOTE_COUNT {
            display
                .print_ascii_bytes(NOTES[index as usize].as_bytes())
                .unwrap();
        } else {
            display.print_ascii_bytes(b" ").unwrap();
        }

        play_note(
            buzzer,
            &mut delay,
            index,
            octave,
            timing,
            timing_unit_ms,
            octave_adjust,
        );
        note_index += 1;
    }
}

pub const SUPER_MARIO: [u16; 322] = [
    n(EN, 5, 4),
    n(EN, 5, 4),
    pause(4),
    n(EN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(EN, 5, 4), // 1
    n(GN, 5, 8),
    pause(8),
    n(GN, 4, 4),
    pause(8),
    n(CN, 5, 12),
    n(GN, 4, 4),
    pause(8),
    n(EN, 4, 12), // 3
    n(AN, 4, 8),
    n(BN, 4, 8),
    n(AS, 4, 4),
    n(AN, 4, 8),
    n(GN, 4, 6),
    n(EN, 5, 6),
    n(GN, 5, 6),
    n(AN, 5, 8),
    n(FN, 5, 4),
    n(GN, 5, 4),
    pause(4),
    n(EN, 5, 8),
    n(CN, 5, 4),
    n(DN, 5, 4),
    n(BN, 4, 12),
    n(CN, 5, 12),
    n(GN, 4, 4),
    pause(8),
    n(EN, 4, 12), // repeats from 3
    n(AN, 4, 8),
    n(BN, 4, 8),
    n(AS, 4, 4),
    n(AN, 4, 8),
    n(GN, 4, 6),
    n(EN, 5, 6),
    n(GN, 5, 6),
    n(AN, 5, 8),
    n(FN, 5, 4),
    n(GN, 5, 4),
    pause(4),
    n(EN, 5, 8),
    n(CN, 5, 4),
    n(DN, 5, 4),
    n(BN, 4, 12),
    pause(8),
    n(GN, 5, 4),
    n(FS, 5, 4),
    n(FN, 5, 4),
    n(DS, 5, 8),
    n(EN, 5, 4), // 7
    pause(4),
    n(GS, 4, 4),
    n(AN, 4, 4),
    n(CN, 4, 4),
    pause(4),
    n(AN, 4, 4),
    n(CN, 5, 4),
    n(DN, 5, 4),
    pause(8),
    n(DS, 5, 8),
    pause(4),
    n(DN, 5, 12),
    n(CN, 5, 16),
    pause(16),
    pause(8),
    n(GN, 5, 4),
    n(FS, 5, 4),
    n(FN, 5, 4),
    n(DS, 5, 8),
    n(EN, 5, 4), // repeats from 7
    pause(4),
    n(GS, 4, 4),
    n(AN, 4, 4),
    n(CN, 4, 4),
    pause(4),
    n(AN, 4, 4),
    n(CN, 5, 4),
    n(DN, 5, 4),
    pause(8),
    n(DS, 5, 8),
    pause(4),
    n(DN, 5, 12),
    n(CN, 5, 16),
    pause(16),
    n(CN, 5, 4),
    n(CN, 5, 8),
    n(CN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(DN, 5, 8), // 11
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(AN, 4, 4),
    n(GN, 4, 16),
    n(CN, 5, 4),
    n(CN, 5, 8),
    n(CN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(DN, 5, 4),
    n(EN, 5, 4), // 13
    pause(32),
    n(CN, 5, 4),
    n(CN, 5, 8),
    n(CN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(DN, 5, 8),
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(AN, 4, 4),
    n(GN, 4, 16),
    n(EN, 5, 4),
    n(EN, 5, 4),
    pause(4),
    n(EN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(EN, 5, 8),
    n(GN, 5, 8),
    pause(8),
    n(GN, 4, 8),
    pause(8),
    n(CN, 5, 12),
    n(GN, 4, 4),
    pause(8),
    n(EN, 4, 12), // 19
    n(AN, 4, 8),
    n(BN, 4, 8),
    n(AS, 4, 4),
    n(AN, 4, 8),
    n(GN, 4, 6),
    n(EN, 5, 6),
    n(GN, 5, 6),
    n(AN, 5, 8),
    n(FN, 5, 4),
    n(GN, 5, 4),
    pause(4),
    n(EN, 5, 8),
    n(CN, 5, 4),
    n(DN, 5, 4),
    n(BN, 4, 12),
    n(CN, 5, 12),
    n(GN, 4, 4),
    pause(8),
    n(EN, 4, 12), // repeats from 19
    n(AN, 4, 8),
    n(BN, 4, 8),
    n(AS, 4, 4),
    n(AN, 4, 8),
    n(GN, 4, 6),
    n(EN, 5, 6),
    n(GN, 5, 6),
    n(AN, 5, 8),
    n(FN, 5, 4),
    n(GN, 5, 4),
    pause(4),
    n(EN, 5, 8),
    n(CN, 5, 4),
    n(DN, 5, 4),
    n(BN, 4, 12),
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(GN, 4, 4),
    pause(8),
    n(GS, 4, 8), // 23
    n(AN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(AN, 4, 16),
    n(DN, 5, 6),
    n(AN, 5, 6),
    n(AN, 5, 6),
    n(AN, 5, 6),
    n(GN, 5, 6),
    n(FN, 5, 6),
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(AN, 4, 4),
    n(GN, 4, 16), // 26
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(GN, 4, 4),
    pause(8),
    n(GS, 4, 8),
    n(AN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(AN, 4, 16),
    n(BN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(FN, 5, 6),
    n(EN, 5, 6),
    n(DN, 5, 6),
    n(CN, 5, 4),
    n(EN, 4, 8),
    n(EN, 4, 4),
    n(CN, 4, 16),
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(GN, 4, 4),
    pause(8),
    n(GS, 4, 8), // repeats from 23
    n(AN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(AN, 4, 16),
    n(DN, 5, 6),
    n(AN, 5, 6),
    n(AN, 5, 6),
    n(AN, 5, 6),
    n(GN, 5, 6),
    n(FN, 5, 6),
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(AN, 4, 4),
    n(GN, 4, 16), // 26
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(GN, 4, 4),
    pause(8),
    n(GS, 4, 8),
    n(AN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(AN, 4, 16),
    n(BN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(FN, 5, 6),
    n(EN, 5, 6),
    n(DN, 5, 6),
    n(CN, 5, 4),
    n(EN, 4, 8),
    n(EN, 4, 4),
    n(CN, 4, 16),
    n(CN, 5, 4),
    n(CN, 5, 8),
    n(CN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(DN, 5, 4),
    n(EN, 5, 4),
    pause(32),
    n(CN, 5, 4),
    n(CN, 5, 8),
    n(CN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(DN, 5, 8), // 33
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(AN, 4, 4),
    n(GN, 4, 16),
    n(EN, 5, 4),
    n(EN, 5, 4),
    pause(4),
    n(EN, 5, 4),
    pause(4),
    n(CN, 5, 4),
    n(EN, 5, 8),
    n(GN, 5, 8),
    pause(8),
    n(GN, 4, 8),
    pause(8),
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(GN, 4, 4),
    pause(8),
    n(GS, 4, 8),
    n(AN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(AN, 4, 16),
    n(DN, 5, 6),
    n(AN, 5, 6),
    n(AN, 5, 6),
    n(AN, 5, 6),
    n(GN, 5, 6),
    n(FN, 5, 6),
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(AN, 4, 4),
    n(GN, 4, 16), // 40
    n(EN, 5, 4),
    n(CN, 5, 8),
    n(GN, 4, 4),
    pause(8),
    n(GS, 4, 8),
    n(AN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(AN, 4, 16),
    n(BN, 4, 4),
    n(FN, 5, 8),
    n(FN, 5, 4),
    n(FN, 5, 6),
    n(EN, 5, 6),
    n(DN, 5, 6),
    n(CN, 5, 4),
    n(EN, 4, 8),
    n(EN, 4, 4),
    n(CN, 4, 16),
    // game over sound
    n(CN, 5, 12),
    n(GN, 4, 12),
    n(EN, 4, 8), // 45
    n(AN, 4, 6),
    n(BN, 4, 6),
    n(AN, 4, 6),
    n(GS, 4, 6),
    n(AS, 4, 6),
    n(GS, 4, 6),
    n(GN, 4, 4),
    n(DN, 4, 4),
    n(EN, 4, 24),
    END,
];
