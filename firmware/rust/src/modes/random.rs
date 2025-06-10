use super::ModeHandler;
use crate::{Context, Display, Event, Rand, SavedSettings, Setting, NUM_CHARS, NUM_COLS};
use hcms_29xx::CHAR_WIDTH;
use random_trait::Random as _;

const EIGHT_BALL_RESPONSES: [&[u8; 8]; 27] = [
    b"   Yes  ",
    b"  Yeah! ",
    b"No doubt",
    b"It is so",
    b"Yeyeyeye",
    b"  Fo sho",
    b" Oh, si!",
    b" Jawohl!",
    b"Oui oui!",
    b"True dat",
    b" Likely ",
    b"  Maybe ",
    b"Maybe no",
    b"Unlikely",
    b" Please.",
    b"    No  ",
    b"   Naw  ",
    b"  Never ",
    b"Nooooooo",
    b"No no no",
    b"  Nein  ",
    b"  Nyet  ",
    b"   Wat? ",
    b"   Dunno",
    b" Ask cat",
    b" Ask cat",
    b"u cappin",
];
const CUISINE_RESPONSES: [&[u8; 8]; 25] = [
    b" Cambodn",
    b" Chinese",
    b"  Cuban ",
    b" Ethiopn",
    b"Filipino",
    b" French ",
    b" German ",
    b"  Greek ",
    b" Indian ",
    b"Indonesn",
    b" Israeli",
    b" Italian",
    b"Japanese",
    b" Koreans",
    b" Malaysn",
    b" Mexican",
    b" Oaxacan",
    b"  Pizza ",
    b" Russian",
    b"Schezwan",
    b" Spanish",
    b"  Thai  ",
    b" Turkish",
    b" Venezln",
    b" Vietnam",
];
const DICE_COLS: [[u8; CHAR_WIDTH]; 6] = [
    [0x00, 0x00, 0x08, 0x00, 0x00],
    [0x20, 0x00, 0x00, 0x00, 0x02],
    [0x20, 0x00, 0x08, 0x00, 0x02],
    [0x22, 0x00, 0x00, 0x00, 0x22],
    [0x22, 0x00, 0x08, 0x00, 0x22],
    [0x2A, 0x00, 0x00, 0x00, 0x2A],
];

#[derive(Clone, Copy)]
enum Page {
    IntegerBase10 = 0,
    // hex integer?
    RollD6,
    EightBall,
    Cuisine,
}

pub struct Random {
    last_update: u16,
    cur_page: Page,
}

impl Random {
    pub fn new_with_settings(settings: &SavedSettings) -> Self {
        let saved_page = settings.read_setting_byte(Setting::RandomPage);
        let page = match saved_page {
            1 => Page::RollD6,
            2 => Page::EightBall,
            3 => Page::Cuisine,
            _ => Page::IntegerBase10,
        };

        Random {
            last_update: 0,
            cur_page: page,
        }
    }

    fn format_integer_base10(buf: &mut [u8], value: u32) {
        let mut value = value;
        for index in (0..NUM_CHARS).rev() {
            buf[index] = b'0' + (value % 10) as u8;
            value /= 10;
        }
    }

    fn format_eight_ball(buf: &mut [u8], index: u8) {
        buf.copy_from_slice(EIGHT_BALL_RESPONSES[index as usize % EIGHT_BALL_RESPONSES.len()]);
    }

    fn format_cuisine(buf: &mut [u8], index: u8) {
        buf.copy_from_slice(CUISINE_RESPONSES[index as usize % CUISINE_RESPONSES.len()]);
    }

    fn roll_d6_message(value: u32, display: &mut Display) {
        let mut value = value;
        let mut col_buf = [0; NUM_COLS];
        for index in (0..NUM_CHARS).rev() {
            let digit_cols = DICE_COLS[(value % 6) as usize];
            for (i, &col) in digit_cols.iter().enumerate() {
                col_buf[index * CHAR_WIDTH + i] = col;
            }
            value /= 6;
        }
        display.print_cols(&col_buf).unwrap();
    }
}

impl ModeHandler for Random {
    #[inline(never)]
    fn update(&mut self, event: &Option<Event>, context: &mut Context, display: &mut Display) {
        let mut update = context.needs_update(&mut self.last_update);

        if let Some(event) = event {
            match event {
                Event::LeftHeld => {
                    context.to_menu();
                    return;
                }
                Event::RightHeld => {
                    self.cur_page = match self.cur_page {
                        Page::IntegerBase10 => Page::RollD6,
                        Page::RollD6 => Page::EightBall,
                        Page::EightBall => Page::Cuisine,
                        Page::Cuisine => Page::IntegerBase10,
                    };
                    context
                        .settings
                        .save_setting_byte(Setting::RandomPage, self.cur_page as u8);
                    update = true;
                }
                Event::LeftPressed | Event::RightPressed => {
                    update = true;
                }

                _ => {}
            }
        }

        if update {
            let mut buf = [b' '; NUM_CHARS];
            let rand_value = Rand::default().get_u32();

            match self.cur_page {
                Page::IntegerBase10 => Self::format_integer_base10(&mut buf, rand_value),
                Page::EightBall => Self::format_eight_ball(&mut buf, rand_value as u8),
                Page::Cuisine => Self::format_cuisine(&mut buf, rand_value as u8),
                Page::RollD6 => {
                    Self::roll_d6_message(rand_value, display);
                    return;
                }
            }

            display.print_ascii_bytes(&buf).unwrap()
        }
    }
}
