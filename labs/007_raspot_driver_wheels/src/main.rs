use i2cdev::core::*;
// NOTE:
// add target to .cargo/config.toml
// [build]
// target = "aarch64-unknown-linux-gnu"
// because I'm on Mac -> this to tell compiler this is meant for the linux on the pi
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

const RASBOT_I2C_ADDRESS: u16 = 0x2B;

struct Bot {
    i2c_address: u16,
    linux_i2c_device: LinuxI2CDevice,
}
#[repr(u8)]
enum MotorId {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[repr(u8)]
enum MotorDirection {
    Forward = 0,
    Backward = 1,
}

struct MotorInfo {
    id: MotorId,
    direction: MotorDirection,
    speed: u8,
}

impl Bot {
    fn new(addr: u16, linux_i2c_device: LinuxI2CDevice) -> Bot {
        Bot {
            i2c_address: addr,
            linux_i2c_device,
        }
    }

    fn write_byte(&mut self, reg: u8, data: u8) -> Result<(), LinuxI2CError> {
        self.linux_i2c_device.smbus_write_byte_data(reg, data)
    }

    fn write_array(&mut self, reg: u8, data: &[u8]) -> Result<(), LinuxI2CError> {
        self.linux_i2c_device.smbus_write_i2c_block_data(reg, data)
    }

    fn move_wheel_motor(&mut self, motor_info: MotorInfo) -> Result<(), LinuxI2CError> {
        let data: [u8; 3] = [
            motor_info.id as u8,
            motor_info.direction as u8,
            motor_info.speed,
        ];

        self.write_array(0x01, &data)?;

        Ok(())
    }
}

fn main() -> Result<(), LinuxI2CError> {
    // initiate a new device
    let device = LinuxI2CDevice::new("/dev/i2c-1", RASBOT_I2C_ADDRESS)?;
    // give it to the bot struct
    let mut bot = Bot::new(RASBOT_I2C_ADDRESS, device);

    // move wheel 0
    bot.move_wheel_motor(MotorInfo {
        id: MotorId::Zero,
        direction: MotorDirection::Forward,
        speed: 255,
    })?;

    Ok(())
}
