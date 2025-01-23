use avr_progmem::progmem;

progmem! {
    pub static progmem FONT5X7: [u8;770] = [

        //  Character 0x00 is not printable since 0x00 is used to indicate NULL terminator in c-strings
        //  So use the first bitmap slot (5 elements) in the array to specify the Font meta-data:
        //  ASCII offset, last ASCII character, don't care, don't care, don't care
        //  ASCII offset is the ASCII value of the first defined bitmap in this table. It should never be less than 1.
        0x01, 0x99, 0x00, 0x00, 0x00, // 0x00

        // The first 32 ASCII codes (0x00 to 0x1F) are normally non-printable control characters.
        // So use these slots for characters not defined by ASCII
        0x30, 0x48, 0x45, 0x40, 0x20, // 0x01 (inverted question mark) - changed from 0x30, 0x45, 0x48, 0x40, 0x30
        0x45, 0x29, 0x11, 0x29, 0x45, // 0x02 (x bar)
        0x7D, 0x09, 0x11, 0x21, 0x7D, // 0x03 (N bar)
        0x7D, 0x09, 0x05, 0x05, 0x79, // 0x04 (n bar)
        0x38, 0x44, 0x44, 0x38, 0x44, // 0x05 (alpha)
        0x7E, 0x01, 0x29, 0x2E, 0x10, // 0x06 (beta)
        0x30, 0x4A, 0x4D, 0x49, 0x30, // 0x07 (delta)
        0x60, 0x50, 0x48, 0x50, 0x60, // 0x08 (Delta)
        0x1E, 0x04, 0x04, 0x38, 0x40, // 0x09 (eta)
        0x3E, 0x49, 0x49, 0x49, 0x3E, // 0x0A (theta)
        0x62, 0x14, 0x08, 0x10, 0x60, // 0x0B (lambda)
        0x40, 0x3C, 0x20, 0x20, 0x1C, // 0x0C (mu)
        0x08, 0x7C, 0x04, 0x7C, 0x02, // 0x0D (pi)
        0x38, 0x44, 0x44, 0x3C, 0x04, // 0x0E (sigma)
        0x41, 0x63, 0x55, 0x49, 0x41, // 0x0F (Sigma)
        0x10, 0x08, 0x78, 0x08, 0x04, // 0x10 (tau)
        0x18, 0x24, 0x7E, 0x24, 0x18, // 0x11 (phi)
        0x5E, 0x61, 0x01, 0x61, 0x5E, // 0x12 (Omega)
        0x78, 0x14, 0x15, 0x14, 0x78, // 0x13 (ring A, Angstrom)
        0x38, 0x44, 0x45, 0x3C, 0x40, // 0x14 (ring a)
        0x78, 0x15, 0x14, 0x15, 0x78, // 0x15 (umlaut A)
        0x38, 0x45, 0x44, 0x3D, 0x40, // 0x16 (umlaut a)
        0x3C, 0x43, 0x42, 0x43, 0x3C, // 0x17 (umlaut O)
        0x38, 0x45, 0x44, 0x45, 0x38, // 0x18 (umlaut o)
        0x3C, 0x41, 0x40, 0x41, 0x3C, // 0x19 (umlaut U)
        0x38, 0x42, 0x40, 0x42, 0x38, // 0x1A (umlaut u)
        0x08, 0x08, 0x2A, 0x1C, 0x08, // 0x1B (right arrow)
        0x20, 0x7E, 0x02, 0x02, 0x02, // 0x1C (square root)
        0x12, 0x19, 0x15, 0x12, 0x00, // 0x1D (squared, superscript 2)
        0x48, 0x7E, 0x49, 0x41, 0x42, // 0x1E (pound sterling)
        0x01, 0x12, 0x7C, 0x12, 0x01, // 0x1F (yen)
        // Standard printable ASCII characters start at 32 (0x20)
        0x00, 0x00, 0x00, 0x00, 0x00, // 0x20 (space)
        0x00, 0x5F, 0x00, 0x00, 0x00, // 0x21 !
        0x00, 0x03, 0x00, 0x03, 0x00, // 0x22 "
        0x14, 0x7F, 0x14, 0x7F, 0x14, // 0x23 #
        0x24, 0x2A, 0x7F, 0x2A, 0x12, // 0x24 $
        0x23, 0x13, 0x08, 0x64, 0x62, // 0x25 %
        0x36, 0x49, 0x56, 0x20, 0x50, // 0x26 &
        0x00, 0x0B, 0x07, 0x00, 0x00, // 0x27 '
        0x00, 0x00, 0x3E, 0x41, 0x00, // 0x28 (
        0x00, 0x41, 0x3E, 0x00, 0x00, // 0x29 )
        0x08, 0x2A, 0x1C, 0x2A, 0x08, // 0x2A *
        0x08, 0x08, 0x3E, 0x08, 0x08, // 0x2B +
        0x00, 0x58, 0x38, 0x00, 0x00, // 0x2C ,
        0x08, 0x08, 0x08, 0x08, 0x08, // 0x2D -
        0x00, 0x30, 0x30, 0x00, 0x00, // 0x2E .
        0x20, 0x10, 0x08, 0x04, 0x02, // 0x2F /
        0x3E, 0x51, 0x49, 0x45, 0x3E, // 0x30 0
        0x00, 0x42, 0x7F, 0x40, 0x00, // 0x31 1
        0x62, 0x51, 0x49, 0x49, 0x46, // 0x32 2
        0x22, 0x41, 0x49, 0x49, 0x36, // 0x33 3
        0x18, 0x14, 0x12, 0x7F, 0x10, // 0x34 4
        0x27, 0x45, 0x45, 0x45, 0x39, // 0x35 5
        0x3C, 0x4A, 0x49, 0x49, 0x30, // 0x36 6
        0x01, 0x71, 0x09, 0x05, 0x03, // 0x37 7
        0x36, 0x49, 0x49, 0x49, 0x36, // 0x38 8
        0x06, 0x49, 0x49, 0x29, 0x1E, // 0x39 9
        0x00, 0x36, 0x36, 0x00, 0x00, // 0x3A :
        0x00, 0x56, 0x36, 0x00, 0x00, // 0x3B ; (changed from 0x00, 0x56, 0x3B, 0x00, 0x00)
        0x00, 0x08, 0x14, 0x22, 0x41, // 0x3C <
        0x14, 0x14, 0x14, 0x14, 0x14, // 0x3D =
        0x41, 0x22, 0x14, 0x08, 0x00, // 0x3E >
        0x02, 0x01, 0x51, 0x09, 0x06, // 0x3F ?
        0x3E, 0x41, 0x5D, 0x55, 0x1E, // 0x40 @
        0x7E, 0x09, 0x09, 0x09, 0x7E, // 0x41 A
        0x7F, 0x49, 0x49, 0x49, 0x36, // 0x42 B
        0x3E, 0x41, 0x41, 0x41, 0x22, // 0x43 C
        0x41, 0x7f, 0x41, 0x41, 0x3E, // 0x44 D  (changed from 0x7F, 0x41, 0x41, 0x41, 0x3E)
        0x7F, 0x49, 0x49, 0x49, 0x41, // 0x45 E
        0x7F, 0x09, 0x09, 0x09, 0x01, // 0x46 F
        0x3E, 0x41, 0x41, 0x51, 0x32, // 0x47 G
        0x7F, 0x08, 0x08, 0x08, 0x7F, // 0x48 H
        0x00, 0x41, 0x7F, 0x41, 0x00, // 0x49 I
        0x20, 0x40, 0x40, 0x40, 0x3F, // 0x4A J
        0x7F, 0x08, 0x14, 0x22, 0x41, // 0x4B K
        0x7F, 0x40, 0x40, 0x40, 0x40, // 0x4C L
        0x7F, 0x02, 0x0C, 0x02, 0x7F, // 0x4D M
        0x7F, 0x04, 0x08, 0x10, 0x7F, // 0x4E N
        0x3E, 0x41, 0x41, 0x41, 0x3E, // 0x4F O
        0x7F, 0x09, 0x09, 0x09, 0x06, // 0x50 P
        0x3E, 0x41, 0x51, 0x21, 0x5E, // 0x51 Q
        0x7F, 0x09, 0x19, 0x29, 0x46, // 0x52 R
        0x26, 0x49, 0x49, 0x49, 0x32, // 0x53 S
        0x01, 0x01, 0x7F, 0x01, 0x01, // 0x54 T
        0x3F, 0x40, 0x40, 0x40, 0x3F, // 0x55 U
        0x07, 0x18, 0x60, 0x18, 0x07, // 0x56 V
        0x7F, 0x20, 0x18, 0x20, 0x7F, // 0x57 W
        0x63, 0x14, 0x08, 0x14, 0x63, // 0x58 X
        0x03, 0x04, 0x78, 0x04, 0x03, // 0x59 Y
        0x61, 0x51, 0x49, 0x45, 0x43, // 0x5A Z
        0x00, 0x00, 0x7F, 0x41, 0x41, // 0x5B [
        0x02, 0x04, 0x08, 0x10, 0x20, // 0x5C (backslash - escape character)
        0x41, 0x41, 0x7F, 0x00, 0x00, // 0x5D ]
        0x04, 0x02, 0x01, 0x02, 0x04, // 0x5E ^ (changed from 0x04, 0x02, 0x7F, 0x02, 0x04)
        0x40, 0x40, 0x40, 0x40, 0x40, // 0x5F _ (underscore)
        0x00, 0x07, 0x0B, 0x00, 0x00, // 0x60 `
        0x38, 0x44, 0x44, 0x3C, 0x40, // 0x61 a
        0x7F, 0x48, 0x44, 0x44, 0x38, // 0x62 b
        0x38, 0x44, 0x44, 0x44, 0x44, // 0x63 c
        0x38, 0x44, 0x44, 0x48, 0x7F, // 0x64 d
        0x38, 0x54, 0x54, 0x54, 0x08, // 0x65 e
        0x08, 0x7E, 0x09, 0x02, 0x00, // 0x66 f
        0x08, 0x14, 0x54, 0x54, 0x3C, // 0x67 g
        0x7F, 0x08, 0x04, 0x04, 0x78, // 0x68 h
        0x00, 0x44, 0x7D, 0x40, 0x00, // 0x69 i
        0x20, 0x40, 0x44, 0x3D, 0x00, // 0x6A j
        0x00, 0x7F, 0x10, 0x28, 0x44, // 0x6B k
        0x00, 0x41, 0x7F, 0x40, 0x00, // 0x6C l
        0x78, 0x04, 0x18, 0x04, 0x78, // 0x6D m
        0x7C, 0x08, 0x04, 0x04, 0x78, // 0x6E n
        0x38, 0x44, 0x44, 0x44, 0x38, // 0x6F o
        0x7C, 0x14, 0x24, 0x24, 0x18, // 0x70 p
        0x18, 0x24, 0x14, 0x7C, 0x40, // 0x71 q
        0x00, 0x7C, 0x08, 0x04, 0x04, // 0x72 r
        0x48, 0x54, 0x54, 0x54, 0x20, // 0x73 s
        0x04, 0x3E, 0x44, 0x20, 0x00, // 0x74 t
        0x3C, 0x40, 0x40, 0x20, 0x7C, // 0x75 u
        0x1C, 0x20, 0x40, 0x20, 0x1C, // 0x76 v
        0x3C, 0x40, 0x30, 0x40, 0x3C, // 0x77 w
        0x44, 0x28, 0x10, 0x28, 0x44, // 0x78 x
        0x0C, 0x50, 0x50, 0x50, 0x3C, // 0x79 y (changed from 0x04, 0x48, 0x30, 0x08, 0x04)
        0x44, 0x64, 0x54, 0x4C, 0x44, // 0x7A z
        0x00, 0x08, 0x36, 0x41, 0x00, // 0x7B {
        0x00, 0x00, 0x7F, 0x00, 0x00, // 0x7C | (changed from 0x00, 0x00, 0x77, 0x00, 0x00)
        0x00, 0x41, 0x36, 0x08, 0x00, // 0x7D }
        0x04, 0x02, 0x04, 0x08, 0x04, // 0x7E ~ (changed from 0x08, 0x04, 0x08, 0x10, 0x08)
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF,  // 0x7F Solid block (replaces ASCII definition of DEL) (changed from 0x2A, 0x55, 0x2A, 0x55, 0x2A)
        // Additional user-defined characters can be added here, up to 0xFF (i.e., 8-bits for a max of 255 character definitions)
        // Be sure to update index 1 (line 32 above, second value in the "00" definition) with the updated highest supported character value when adding characters.

        // Extra characters
        0x2A, 0x55, 0x2A, 0x55, 0x2A, // 0x80 Every other pixel on/off
        0x08, 0x1C, 0x3E, 0x7F, 0x00, // 0x81 Left pointing triangle
        0x00, 0x7F, 0x3E, 0x1C, 0x08, // 0x82 Right pointing triangle
        0x08, 0x0C, 0x0E, 0x0C, 0x08, // 0x83 Up pointing triangle
        0x08, 0x18, 0x38, 0x18, 0x08, // 0x84 Down pointing triangle
        0x08, 0x1C, 0x2A, 0x08, 0x08, // 0x85 Left arrow
        0x08, 0x08, 0x2A, 0x1C, 0x08, // 0x86 Right arrow
        0x04, 0x02, 0x7F, 0x02, 0x04, // 0x87 Up arrow
        0x10, 0x20, 0x7F, 0x20, 0x10, // 0x88 Down arrow
        0x00, 0x00, 0x07, 0x00, 0x00, // 0x89 ' Apostrophe/straight single quote
        0x00, 0x07, 0x00, 0x07, 0x00, // 0x8A " Straight double quote
        0x00, 0x01, 0x02, 0x04, 0x00, // 0x8B ` Simple back-tick
        0x7F, 0x41, 0x41, 0x41, 0x7F, // 0x8C box
        0x00, 0x08, 0x1C, 0x08, 0x00, // 0x8D inner product/dot product
        0x00, 0x00, 0x08, 0x00, 0x00, // 0x8E inner product/dot product (single pixel)
        0x00, 0x14, 0x08, 0x14, 0x00, // 0x8F cross product
        0x00, 0x7F, 0x00, 0x7F, 0x00, // 0x90 || logigal or
        0x08, 0x14, 0x2A, 0x14, 0x22, // 0x91 << double angle bracket
        0x22, 0x14, 0x2A, 0x14, 0x08, // 0x92 >> double angle bracket
        0x30, 0x38, 0x34, 0x32, 0x31, // 0x93 <= less than or equal
        0x31, 0x32, 0x34, 0x38, 0x30, // 0x94 >= greater than or equal
        0x2A, 0x2A, 0x2A, 0x2A, 0x2A, // 0x95 triple bar, equivalence
        0x10, 0x00, 0x02, 0x00, 0x10, // 0x96 therefore
        0x04, 0x04, 0x04, 0x04, 0x1C, // 0x97 not
        0x00, 0x00, 0x02, 0x05, 0x02, // 0x98 degrees
        0x08, 0x08, 0x2A, 0x08, 0x08, // 0x99 division
    ];
}
