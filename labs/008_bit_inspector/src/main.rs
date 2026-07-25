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
    SetBit { value: String, position: u32 },
    ClearBit { value: String, position: u32 },
    ToggleBit { value: String, position: u32 },
    TestBit { value: String, position: u32 },
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

fn inspect(hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Inspecting {}\n", hex);

    // check if it's a hex
    if !hex.starts_with("0x") && !hex.starts_with("0X") {
        return Err("Invalid hex value".into());
    }
    let hex_value = hex.replace("0x", "").replace("0X", "");
    if !hex_value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid hex value {c}".into());
    }

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

fn set_bit(value: &str, position: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting bit {} in {}", position, value);
    Ok(())
}

fn clear_bit(value: &str, position: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Clearing bit {} in {}", position, value);
    Ok(())
}

fn toggle_bit(value: &str, position: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Toggling bit {} in {}", position, value);
    Ok(())
}

fn test_bit(value: &str, position: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing bit {} in {}", position, value);
    Ok(())
}

fn reverse(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reversing {}", value);
    Ok(())
}

fn swap(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Swapping {}", value);
    Ok(())
}
