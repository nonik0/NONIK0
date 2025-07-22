# Nick's Original Name Indicator Knicknack (version 0)

## Overview

<img src="https://github.com/nonik0/NONIK0/blob/main/images/nonik0.jpg" align="right" height="150" alt="Front of NONIK0 board powered on and showing NONIK0 on display"/>

It's a knicknack to indicate your name with the original design by me, Nick. Version 0. NONIK0 for short! The only version there will ever be. I wanted a name badge to wear sometimes and most other times just use for a fancy LED decoration, and also learn some more Rust. Add in a tad of scope creep and you have also have some games, beeps, and other extras, too. For more detail, I have a [project writeup](https://altonimb.us/nonik0/) where I talk about my process of completing this project start to finish, with more of a focus my experience working with Rust.

### Hardware
The hardware is designed in KiCad 8.0 and all design files are included in the [hardware/kicad8.0](hardware/kicad8.0) directory.

The main hardware components are:
- [HCMS-2914](https://www.broadcom.com/products/leds-and-displays/smart-alphanumeric-displays/serial-interface/hcms-2914) 5x7 dot matrix character display
- [ATtiny1604](https://www.microchip.com/en-us/product/attiny1604) 8-bit MCU
- [CMT-322-65-SMT-TR](https://www.digikey.com/en/products/detail/same-sky-formerly-cui-devices/CMT-322-65-SMT-TR/14682617) Piezo Buzzer 3.2mm x 3.2mm
- 2x [PTS526SMG](https://www.digikey.com/en/products/detail/c-k/PTS526SMG15JSMTR2-LFS/10056629) SPST push buttons

Revision #1:
- Swapped test pads and UPDI pads. The UPDI pads were opposite of the buzzer, which made getting the probe clip to remain clipped over the pads with the buzzer in the way more difficult than it needed to be.
- Added solder jumper to HCMS-291x SEL pin. Can optionally cut jumper to Vcc and solder other side to ground to use external oscillator for display.
- Optimized some routing for power and ground.

Revision #2:
- Fixed I2C pin assignment for external JST-SH connector
- Added piezo driver circuit

### Firmware

The firmware is written in Rust and can be found in the [firmware/rust](firmware/rust) directory. As part of the work, I wrote a new [HCMS-29xx driver](https://github.com/nonik0/hcms-29xx) and forked avr-hal to add a Feather32u4 support, which I used for bootstrapping the HCMS-29xx driver work. There is also an older PlatformIO project targetting the Adafruit Feather in the [firmware/cpp](firmware/cpp) directory.

The firmware is navigated using two buttons. Navigation and control is done with the two push buttons. A short press on the right button is a "next" action and will show the next option in the current context (i.e. the menu or active mode). A short press on the left button depends on the given context, and is either a "previous" action or a "alternate" action. A long press on the right button is an "enter/confirm" action and will enter a mode from the menu or, from within a mode, change the current page or enter a submenu. A long press on the left button will do an "exit" action which exits from a mode to the menu, or exits from a submenu from a mode.

#### Modes

- **Name:** The original inspiration for this project. Displays an 8 character name (or other string). Can enter an edit mode to update the name/value and is persisted in EEPROM.
- **Settings:** Change the current max brightness and max current settings for HCMS-29XX display, in addition to toggling the button tones on or off.
- **Vibes:** Inspired from the prototype's zigzag pattern I used for testing scrolling/smoothness. It reminded me of watching mountains pass by from a train windows, so I added this mode with mountains and clouds with a parallax effect. The speed of the clouds and mountains can be changed with buttons.
- **Sensors:** The first of the scope creep feartures. Uses the ADC and can read various channels includes the internal temperature and references voltage, but primary the external voltage on the external JUST port pins for use as a simple voltmeter. Has a settings page where the various ADC parameter can be alterned, I used this for my own learning and understanding of the ADC parameters when implementing.
- **Random:** A simple mode that has several pages to display random values for use as a decision tool: integer values, dice values, "eight ball" yes/maybe/no values, and random cuisine types.
- **Tunnel** A basic tunnel game where you control a runner that moves up/down with left/right buttons and you try to survive as long as possible as the tunnel shrinks.
- **Traffic** Another basic game where a you control a driver that advances to the right as you avoid other traffic (rectangles) progressing to the left.

## Design Images

<p align="center" width="100%">
  <img src="https://github.com/nonik0/NONIK0/blob/main/images/schematic.png" />
  <img src="https://github.com/nonik0/NONIK0/blob/main/images/pcb_layout_front.png"/>
  <img src="https://github.com/nonik0/NONIK0/blob/main/images/pcb_layout_back.png"/>
  <img src="https://github.com/nonik0/NONIK0/blob/main/images/render_front.png" />
  <img src="https://github.com/nonik0/NONIK0/blob/main/images/render_back.png" />
</p>


