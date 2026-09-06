// NOTE: Example data: 30 bytes.
// AA FF 03 00 0E 03 B1 86 10 00 40 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 CC
// |---header| |---- goal 1 info ----| |---- goal 2 info ----| |---- goal 3 info ----| |eof|
//
// Objective 1 x-coordinate: 0x0E + 0x03 * 256 = 782
// 0 - 782 = -782 mm
// Objective 1 y-coordinate: 0xB1 + 0x86 * 256 = 34481
// 34481 - 2^15 = 1713 mm
// Goal 1 speed：0x10 + 0x00 * 256 = 16
// 0 -16 =-16 cm/s
// Target 1 distance resolution: 0x40 +0x01* 256 = 320 mm
//                    target (x, y)
//                         *
//                        /|
//                       / |
//      actual distance /  | y (forward)
//                     /θ  |
//            Radar  *-----+
//                    x (sideways)
//     azimuth angle θ = atan2(x, y)
//
//  Distance Resolution:
//  Radar → | gate | gate | gate | gate | ...
// approximately <distance_resolution> each
#[derive(Debug)]
pub struct TargetInfo {
    pub x_coordinate: i16,
    pub y_coordinate: i16,
    pub speed: i16,
    pub actual_distance: u32,
    pub distance_resolution: u16,
}

pub fn process_data(data: &[u8]) -> TargetInfo {
    let mut target_info = TargetInfo {
        x_coordinate: 0,
        y_coordinate: 0,
        speed: 0,
        actual_distance: 0,
        distance_resolution: 0,
    };

    // == X coord ==//
    // the bytes is little endian
    // data[0] = 0x0E
    // data[1] = 0x03
    // raw = 0x030E = 782
    let raw = u16::from_le_bytes([data[0], data[1]]);

    // magnitude, ignore the sign bit
    let magnitude = (raw & 0x7FFF) as i16; // ignore the sign bit with & 0x7FFF
    // The most significant bit of raw is sign bit
    target_info.x_coordinate = if raw & 0x8000 != 0 {
        // positive
        magnitude
    } else {
        //negative
        -magnitude
    };
    // == X coord ==//

    // == Y coord ==//
    let raw = u16::from_le_bytes([data[2], data[3]]);
    let magnitude = (raw & 0x7FFF) as i16;
    target_info.y_coordinate = if raw & 0x8000 != 0 {
        magnitude
    } else {
        -magnitude
    };
    // == Y coord ==//

    // == speed  ==//
    let raw = u16::from_le_bytes([data[4], data[5]]);
    let magnitude = (raw & 0x7FFF) as i16;
    target_info.speed = if raw & 0x8000 != 0 {
        magnitude
    } else {
        -magnitude
    };
    // == speed ==//

    // == distance reso  ==//
    target_info.distance_resolution = u16::from_le_bytes([data[6], data[7]]);
    // == distance reso  ==//

    // == actual distance == //
    // convert x, y to i32 to prevent overflow
    let x = i32::from(target_info.x_coordinate);
    let y = i32::from(target_info.y_coordinate);
    target_info.actual_distance = ((x.pow(2) + y.pow(2)) as u32).isqrt();
    // == actual distance == //

    target_info
}
