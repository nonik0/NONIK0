#include "HCMS39xx.h"
#include <Wire.h>

#define NUM_CHARS 8

#ifdef ARDUINO_ARCH_MEGAAVR
// static pins:
// SEL: HIGH
#define DIN_PIN PIN_PA6   // 4
#define RS_PIN PIN_PA4    // 2
#define CLK_PIN PIN_PA3   // 13
#define CE_PIN PIN_PA2    // 12
#define BL_PIN PIN_PA1    // 11
//#define RESET_PIN PIN_PB0 // 9
#define RESET_PIN PIN_PB2 // 7

#define BTN1_PIN PIN_PA7  // 5
#define BTN2_PIN PIN_PB3  // 6

#define SDA_PIN PIN_PB1   // 8
//#define SCL_PIN PIN_PB2 // 7
#define SCL_PIN PIN_PB0   // 9

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

#ifdef SEL_PIN
HCMS39xx hcms29xx(NUM_CHARS, DIN_PIN, RS_PIN, CLK_PIN, CE_PIN, BL_PIN, SEL_PIN);
#else
HCMS39xx hcms29xx(NUM_CHARS, DIN_PIN, RS_PIN, CLK_PIN, CE_PIN, BL_PIN);
#endif
uint8_t brightness = 0x0C;
HCMS39xx::DISPLAY_CURRENT current = HCMS39xx::CURRENT_4_0_mA;

void setup()
{
  Wire.begin();

  hcms29xx.begin();
  hcms29xx.displayUnblank();
  hcms29xx.setIntOsc();
  hcms29xx.setBrightness(brightness);
  hcms29xx.setCurrent(current);
  
  hcms29xx.print("I2C SCAN");
  delay(1000);
}

void loop()
{
  hcms29xx.clear();
  hcms29xx.print("SCANNING");
  delay(1000);

  int deviceCount = 0;
  for (byte address = 1; address < 127; address++)
  {
    Wire.beginTransmission(address);
    byte error = Wire.endTransmission();
    
    if (error == 0)
    {
      deviceCount++;
      
      char buffer[10];
      snprintf(buffer, sizeof(buffer), "ACK:0x%02X", address);
      hcms29xx.clear();
      hcms29xx.print(buffer);
      delay(2000);
    }
  }
}
