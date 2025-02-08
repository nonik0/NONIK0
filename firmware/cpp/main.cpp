#include "HCMS39xx.h"

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

#define NUM_CHARS 8

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

HCMS39xx dotMatrix(NUM_CHARS, DIN_PIN, RS_PIN, CLK_PIN, CE_PIN, BL_PIN, SEL_PIN);

#define MAXLEN 20
uint8_t displaydata[MAXLEN];
const char *Name1 = "Stella";
const char *Name2 = "Beau";



void setup()
{
  Serial.begin(9600);

  pinMode(RESET_PIN, OUTPUT);
  digitalWrite(RESET_PIN, HIGH);

  // put into high impedance mode
  pinMode(DOUT_PIN, INPUT);
  pinMode(OSC_PIN, INPUT);
  pinMode(VLOGIC_PIN, INPUT);
  pinMode(GND_PIN, INPUT);

  dotMatrix.begin();
  dotMatrix.displayUnblank();
  dotMatrix.setIntOsc();
}

uint32_t count1 = 0;
uint32_t count2 = 0;
void loop()
{
  count1 = (count1 + 1) % 10000;

  if (random(0, 2) % 2 == 0)
  {
    count2 = (count2 + 1) % 10000;
  }

  // shift count1 4 decimal places to the left
  uint32_t displayNumber = count1 * 10000 + count2;
  dotMatrix.print(displayNumber);
  delay(5);
}

