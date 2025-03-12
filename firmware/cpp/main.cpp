#include "HCMS39xx.h"

#define NUM_CHARS 8

#ifdef ARDUINO_ARCH_MEGAAVR
#define DIN_PIN PIN_PA6   // 4
#define RS_PIN PIN_PA4    // 2
#define CLK_PIN PIN_PA3   // 13
#define CE_PIN PIN_PA2    // 12
#define BL_PIN PIN_PA1    // 11
#define RESET_PIN PIN_PB0 // 9
#define BTN1_PIN PIN_PA7  // 5
#define BTN2_PIN PIN_PB3  // 6
#define SDA_PIN PIN_PB1   // 8
#define SCL_PIN PIN_PB2   // 7
#define BUZZ_PIN PIN_PA5  // 3
#endif

#ifdef ARDUINO_AVR_FEATHER32U4
// static pins:
// 2: OSC: NC
// 8: BLANK: GND
// 10: SEL: HIGH
// 12: Reset: HIGH

// pin mappings feather proto:
// top display
// 7 | CE       | SDA/2
// 8 | BLANK    | SCL/3
// 9 | GND      | 5
// 10| SEL      | 6
// 11| Vlogic   | 9
// 12| Reset    | 10
// bottom display
// 1 | Data Out | SCK
// 2 | OSC      | MOSI
// 3 | Vled     | MISO
// 4 | Data In  | RX/0
// 5 | RS       | TX/1
// 6 | CLK      | GND (have to cut connection and route another pin)
#define DIN_PIN 0
#define RS_PIN 1
#define CLK_PIN 11
#define CE_PIN 2
#define BL_PIN 3
#define SEL_PIN 6
#define RESET_PIN 10
#define DOUT_PIN SCK
#define OSC_PIN MOSI
#define VLOGIC_PIN 9
#define VLED_PIN MISO
#define GND_PIN 5
#endif

#if defined(ARDUINO_ARCH_ESP32)
#define DIN_PIN 35
#define RS_PIN 37
#define CLK_PIN 36
#define CE_PIN 34
// missing pins
#endif

HCMS39xx hcms29xx(NUM_CHARS, DIN_PIN, RS_PIN, CLK_PIN, CE_PIN, BL_PIN, SEL_PIN);
uint8_t brightness = 0x0C;
HCMS39xx::DISPLAY_CURRENT current = HCMS39xx::CURRENT_4_0_mA;

void setup()
{
  Serial.begin(9600);

#if defined(RESET_PIN)
  pinMode(RESET_PIN, OUTPUT);
  digitalWrite(RESET_PIN, HIGH);
#endif

#if defined(ARDUINO_ARCH_MEGAAVR)
  pinMode(BTN1_PIN, INPUT_PULLUP);
  pinMode(BTN2_PIN, INPUT_PULLUP);
#endif

#if defined(ARDUINO_AVR_FEATHER32U4)
  // put into high impedance
  pinMode(DOUT_PIN, INPUT);
  pinMode(OSC_PIN, INPUT);
  pinMode(VLOGIC_PIN, INPUT);
  pinMode(GND_PIN, INPUT);
#endif

  hcms29xx.begin();
  hcms29xx.displayUnblank();
  hcms29xx.setIntOsc();
  hcms29xx.setBrightness(brightness);
  hcms29xx.setCurrent(current);
}

uint32_t count1 = 0;
uint32_t count2 = 0;
void loop()
{
#if defined(ARDUINO_ARCH_MEGAAVR)
  if (digitalRead(BTN1_PIN) == LOW)
  {
    Serial.println("Button 1 pressed");
    tone(BUZZ_PIN, 4000, 500);
    count1 = 0;

    brightness = (brightness + 1) % 12;
    dotMatrix.setBrightness(brightness);
  }

  if (digitalRead(BTN2_PIN) == LOW)
  {
    Serial.println("Button 2 pressed");
    tone(BUZZ_PIN, 8000, 500);
    count2 = 0;

    if (current == HCMS39xx::CURRENT_4_0_mA)
    {
      current = HCMS39xx::CURRENT_6_4_mA;
    }
    else if (current == HCMS39xx::CURRENT_6_4_mA)
    {
      current = HCMS39xx::CURRENT_9_3_mA;
    }
    else if (current == HCMS39xx::CURRENT_9_3_mA)
    {
      current = HCMS39xx::CURRENT_12_8_mA;
    }
    else if (current == HCMS39xx::CURRENT_12_8_mA)
    {
      current = HCMS39xx::CURRENT_4_0_mA;
    }
    dotMatrix.setCurrent(current);
  }
#endif

  count1 = (count1 + 1) % 10000;

  if (random(0, 2) % 2 == 0)
  {
    count2 = (count2 + 1) % 10000;
  }

  // shift count1 4 decimal places to the left
  uint32_t displayNumber = count1 * 10000 + count2;
  hcms29xx.print(displayNumber);
  delay(5);
}
