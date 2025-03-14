// TODO: implement interrupt-based button handling
use embedded_hal::digital::InputPin;

pub enum ButtonEvent {
    BothPressed,
    BothHeld,
    LeftPressed,
    LeftHeld,
    LeftReleased,
    RightPressed,
    RightHeld,
    RightReleased
}

pub struct Buttons<LPin, RPin>
where
    LPin: InputPin,
    RPin: InputPin,
{
    left: LPin,
    right: RPin,
    left_held_cycles: u8,
    right_held_cycles: u8,
}

impl<LPin, RPin> Buttons<LPin, RPin>
where
    LPin: InputPin,
    RPin: InputPin,
{
    pub fn new(left: LPin, right: RPin) -> Self {
        Buttons {
            left,
            right,
            left_held_cycles: 0,
            right_held_cycles: 0,
        }
    }

    pub fn update(&mut self) -> Option<ButtonEvent> {
        let left_pressed = self.left.is_low().unwrap();
        let right_pressed = self.right.is_low().unwrap();

        // Handle both buttons pressed
        if left_pressed && right_pressed {
            // Increment counters for both buttons
            self.left_held_cycles += 1;
            self.right_held_cycles += 1;

            // Check for "just pressed" or "held" conditions
            if self.left_held_cycles == 1 || self.right_held_cycles == 1 {
                return Some(ButtonEvent::BothPressed);
            } else if self.left_held_cycles > 10 && self.right_held_cycles > 10 {
                return Some(ButtonEvent::BothHeld);
            }
        }
        // Handle left button
        else if left_pressed {
            self.left_held_cycles += 1;
            if self.left_held_cycles == 1 {
                return Some(ButtonEvent::LeftPressed);
            } else if self.left_held_cycles > 10 {
                return Some(ButtonEvent::LeftHeld);
            }
        } else if self.left_held_cycles > 0 {
            self.left_held_cycles = 0;
            return Some(ButtonEvent::LeftReleased);
        }
        // Handle right button
        else if right_pressed {
            self.right_held_cycles += 1;
            if self.right_held_cycles == 1 {
                return Some(ButtonEvent::RightPressed);
            } else if self.right_held_cycles > 10 {
                return Some(ButtonEvent::RightHeld);
            }
        } else if self.right_held_cycles > 0 {
            self.right_held_cycles = 0;
            return Some(ButtonEvent::RightReleased);
        }

        None
    }
}
