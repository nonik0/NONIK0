use embedded_hal::digital::InputPin;

const PRESS_CYCLES: u8 = 1;
const HOLD_CYCLES: u8 = 20;
const DEBOUNCE_CYCLES: u8 = 3;

pub enum InputEvent {
    BothPressed,
    BothHeld,
    LeftPressed,
    LeftReleased,
    LeftHeld,
    LeftHeldReleased,
    RightPressed,
    RightHeld,
    RightReleased,
    RightHeldReleased,
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
    left_debounce_cycles: u8,
    right_debounce_cycles: u8,
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
            left_debounce_cycles: 0,
            right_debounce_cycles: 0,
        }
    }

    pub fn update(&mut self) -> Option<InputEvent> {
        let mut left_pressed = self.left.is_low().unwrap();
        let mut right_pressed = self.right.is_low().unwrap();
    
        // If debouncing is active, ignore presses
        if self.left_debounce_cycles > 0 {
            self.left_debounce_cycles -= 1;
            left_pressed = false;
        }
    
        if self.right_debounce_cycles > 0 {
            self.right_debounce_cycles -= 1;
            right_pressed = false;
        }
    
        // Handle both buttons pressed
        if left_pressed && right_pressed {
            // Increment counters for both buttons
            self.left_held_cycles += 1;
            self.right_held_cycles += 1;

            // Check for "just pressed" or "held" conditions
            if self.left_held_cycles == 1 || self.right_held_cycles == 1 {
                return Some(InputEvent::BothPressed);
            } else if self.left_held_cycles > 10 && self.right_held_cycles > 10 {
                return Some(InputEvent::BothHeld);
            }
        }

        // Handle left button
        if left_pressed {
            self.left_held_cycles += 1;
            if self.left_held_cycles == PRESS_CYCLES {
                return Some(InputEvent::LeftPressed);
            } else if self.left_held_cycles == HOLD_CYCLES {
                return Some(InputEvent::LeftHeld);
            }
        } else if self.left_held_cycles > 0 {
            let was_held = self.left_held_cycles >= HOLD_CYCLES;
    
            self.left_debounce_cycles = DEBOUNCE_CYCLES;
            self.left_held_cycles = 0;
    
            if was_held  {
                return Some(InputEvent::LeftHeldReleased);
            } else {
                return Some(InputEvent::LeftReleased);
            }
        }
    
        // Handle right button
        if right_pressed {
            self.right_held_cycles += 1;
            if self.right_held_cycles == PRESS_CYCLES {
                return Some(InputEvent::RightPressed);
            } else if self.right_held_cycles == HOLD_CYCLES {
                return Some(InputEvent::RightHeld);
            }
        } else if self.right_held_cycles > 0 {
            let was_held = self.right_held_cycles >= HOLD_CYCLES;
    
            self.right_debounce_cycles = DEBOUNCE_CYCLES;
            self.right_held_cycles = 0;
    
            if was_held {
                return Some(InputEvent::RightHeldReleased);
            } else {
                return Some(InputEvent::RightReleased);
            }
        }
    
        None
    }
}    