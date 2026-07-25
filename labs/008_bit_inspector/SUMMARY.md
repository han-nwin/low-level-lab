| Command                            | Purpose                                                                                 | Example                    |
| ---------------------------------- | --------------------------------------------------------------------------------------- | -------------------------- |
| **inspect VALUE**                  | Display every representation of the value (binary, hex, signed, unsigned, bytes, etc.). | `inspect 0xDEADBEEF`       |
| **set-bit VALUE POSITION**         | Force one bit to 1.                                                                     | `set-bit 0x10 0`           |
| **clear-bit VALUE POSITION**       | Force one bit to 0.                                                                     | `clear-bit 0xFF 7`         |
| **toggle-bit VALUE POSITION**      | Flip one bit.                                                                           | `toggle-bit 0x8 3`         |
| **test-bit VALUE POSITION**        | Check whether a bit is set.                                                             | `test-bit 0x20 5`          |
| **reverse VALUE**                  | Reverse all bits in the integer.                                                        | `reverse 0x12345678`       |
| **rotate VALUE AMOUNT LEFT/RIGHT** | Circularly rotate bits left or right.                                                   | `rotate 0x12345678 8 left` |
| **swap VALUE**                     | Swap the byte order (endianness).                                                       | `swap 0x12345678`          |
| **sign-extend VALUE SOURCE_BITS**  | Extend a smaller signed integer to 32 bits.                                             | `sign-extend 0xFF 8`       |
