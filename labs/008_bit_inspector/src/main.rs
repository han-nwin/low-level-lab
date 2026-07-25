use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Inspect { value: String },
    SetBit { value: String, position: u8 },
    ClearBit { value: String, position: u8 },
    ToggleBit { value: String, position: u8 },
    TestBit { value: String, position: u8 },
    Reverse { value: String },
    Swap { value: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Commands::Inspect { value } => inspect(&value),
        Commands::SetBit { value, position } => set_bit(&value, position),
        Commands::ClearBit { value, position } => clear_bit(&value, position),
        Commands::ToggleBit { value, position } => toggle_bit(&value, position),
        Commands::TestBit { value, position } => test_bit(&value, position),
        Commands::Reverse { value } => reverse(&value),
        Commands::Swap { value } => swap(&value),
    }
}

fn is_hex(value: &str) -> bool {
    // check if it's a hex
    if !value.starts_with("0x") && !value.starts_with("0X") {
        return false;
    }
    let hex_value = value.replace("0x", "").replace("0X", "");
    if !hex_value.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    true
}

fn is_valid_position(position: u8) -> bool {
    // check if the position is valid
    if position > 31 {
        eprintln!("Position {} is out of range (0..=31)", position);
        return false;
    }
    true
}

fn inspect(hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Inspecting {}\n", hex);

    if !is_hex(hex) {
        return Err("Invalid hex value".into());
    }
    let hex_value = hex.replace("0x", "").replace("0X", ""); // remove the 0x or 0X

    println!("Hex: {}", hex);

    let mut binary = String::from("0b");
    for char in hex_value.chars() {
        match char {
            '0' => binary.push_str("0000"),
            '1' => binary.push_str("0001"),
            '2' => binary.push_str("0010"),
            '3' => binary.push_str("0011"),
            '4' => binary.push_str("0100"),
            '5' => binary.push_str("0101"),
            '6' => binary.push_str("0110"),
            '7' => binary.push_str("0111"),
            '8' => binary.push_str("1000"),
            '9' => binary.push_str("1001"),
            'A' | 'a' => binary.push_str("1010"),
            'B' | 'b' => binary.push_str("1011"),
            'C' | 'c' => binary.push_str("1100"),
            'D' | 'd' => binary.push_str("1101"),
            'E' | 'e' => binary.push_str("1110"),
            'F' | 'f' => binary.push_str("1111"),
            _ => {
                return Err("Invalid hex value {char}".into());
            }
        }
    }
    println!("Binary: {}", binary);

    let u_32 = u32::from_str_radix(&hex_value, 16)?; // 16 mean base 16 aka hex
    println!("As u32: {}", u_32);

    let i_32 = i32::from_be_bytes(u_32.to_be_bytes()); // convert u32 to be bytes then convert to
    // i32
    println!("As i32: {}", i_32);

    let big_endian = u_32.to_be_bytes();
    println!("As big endian bytes: {:02X?}", big_endian);

    let octal_big_endian = big_endian
        .iter()
        .map(|b| format!("{:03o}", b))
        .collect::<Vec<String>>();
    println!("As big endian octal: {octal_big_endian:?}");

    let little_endian = u_32.to_le_bytes();
    println!("As little endian bytes: {:02X?}", little_endian);

    let native_endian = u_32.to_ne_bytes();
    println!("As native endian bytes: {:03X?}", native_endian);

    Ok(())
}

// Example output for set_bit
// set-bit 0x00 0
// Expected: 0x01
//
// set-bit 0x08 1
// Expected: 0x0A
//
// set-bit 0xF0 3
// Expected: 0xF8
fn set_bit(hex: &str, position: u8) -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting bit {} in {}", position, hex);
    if !is_hex(hex) {
        return Err("Invalid hex value".into());
    }
    if !is_valid_position(position) {
        return Err("Invalid position".into());
    }

    let u_32 = u32::from_str_radix(&hex.replace("0x", "").replace("0X", ""), 16)?; // 16 mean base 16 aka hex
    let mask = 1 << position;

    let result = u_32 | mask; // set the bit with the mask by ORing it with the mask
    println!("Result: 0x{result:02X?}");

    Ok(())
}

// Example output for clear_bit
// clear-bit 0x00 0
// Expected: 0x00
//
// clear-bit 0x07 1
// Expected: 0x05
//
// clear-bit 0xF0 4
// Expected: 0xE0
fn clear_bit(hex: &str, position: u8) -> Result<(), Box<dyn std::error::Error>> {
    println!("Clearing bit {} in {}", position, hex);
    if !is_hex(hex) {
        return Err("Invalid hex value".into());
    }
    if !is_valid_position(position) {
        return Err("Invalid position".into());
    }

    let u_32 = u32::from_str_radix(&hex.replace("0x", "").replace("0X", ""), 16)?; // 16 mean base 16 aka hex
    let mask = !(1 << position);

    let result = u_32 & mask; // clear the bit with the mask by ANDing it with the mask
    println!("Result: 0x{result:02X?}");

    Ok(())
}

// toggle-bit 0x00 0
// Expected: 0x01
//
// toggle-bit 0x0A 1
// Expected: 0x08
//
// toggle-bit 0xF0 3
// Expected: 0xF8
fn toggle_bit(hex: &str, position: u8) -> Result<(), Box<dyn std::error::Error>> {
    println!("Toggling bit {} in {}", position, hex);
    if !is_hex(hex) {
        return Err("Invalid hex value".into());
    }
    if !is_valid_position(position) {
        return Err("Invalid position".into());
    }

    let u_32 = u32::from_str_radix(&hex.replace("0x", "").replace("0X", ""), 16)?; // 16 mean base 16 aka hex
    let mask = 1 << position;

    let result = u_32 ^ mask; //toggle the bit with the mask by XORing it with the mask
    println!("Result: 0x{result:02X?}");

    Ok(())
}

// test-bit 0x01 0
// Expected: true
//
// test-bit 0x0A 1
// Expected: true
//
// test-bit 0xF0 3
// Expected: false
fn test_bit(hex: &str, position: u8) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing bit {} in {} if it's on", position, hex);
    if !is_hex(hex) {
        return Err("Invalid hex value".into());
    }
    if !is_valid_position(position) {
        return Err("Invalid position".into());
    }

    let u_32 = u32::from_str_radix(&hex.replace("0x", "").replace("0X", ""), 16)?; // 16 mean base 16 aka hex
    let mask = 1 << position;

    let result = u_32 & mask; // test the bit with the mask by ANDing it with the mask
    //
    let result_tf = result != 0;
    println!("Result: {result_tf}");

    Ok(())
}

// reverse 0x00000001
// Expected: 0x80000000
//
// reverse 0x0000000F
// Expected: 0xF0000000
//
// reverse 0x12345678
// Expected: 0x1E6A2C48
fn reverse(hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reversing {}", hex);
    if !is_hex(hex) {
        return Err("Invalid hex value".into());
    }

    let u_32 = u32::from_str_radix(&hex.replace("0x", "").replace("0X", ""), 16)?; // 16 mean base 16 aka hex
    // make it big endian then read as little endian = reversed
    let mut big_endian = u_32.to_be_bytes();
    // reverse inside each big endian byte
    // for byte in big_endian.iter_mut() {
    //     // NOTE: * turns byte into a mutable reference
    //     *byte = byte.reverse_bits();
    // }

    // Our own reversed bit implementation
    // read each bit and move it to the opposition position
    let mut reversed_bit_big_endian: Vec<u8> = Vec::new();
    for byte in big_endian.iter_mut() {
        let mut reverse: u8 = 0;

        for position in 0..8 {
            // read a bit
            // NOTE: * turns byte into a mutable reference
            let bit = (*byte >> position) & 1;
            // move it to opposition position
            let opposite = 7 - position;

            // record that bit to the result
            reverse |= bit << opposite;
        }
        reversed_bit_big_endian.push(reverse);
    }
    let reversed = u32::from_le_bytes(reversed_bit_big_endian.try_into().unwrap()); // try_into
    // turn Vec to array

    println!("Result: 0x{reversed:02X?}");
    Ok(())
}

//swap 0x12345678
// Expected: 0x78563412
//
// swap 0xAABBCCDD
// Expected: 0xDDCCBBAA
//
// swap 0x000000FF
// Expected: 0xFF000000
fn swap(hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Swapping {}", hex);
    if !is_hex(hex) {
        return Err("Invalid hex value".into());
    }

    let u_32 = u32::from_str_radix(&hex.replace("0x", "").replace("0X", ""), 16)?; // 16 mean base 16 aka hex
    // make it big endian bytes then read backward, no need to flip inside each byte
    let big_endian = u_32.to_be_bytes();
    // reverse inside each big endian byte
    let swapped = u32::from_le_bytes(big_endian);
    println!("Result: 0x{swapped:02X?}");
    Ok(())
}
