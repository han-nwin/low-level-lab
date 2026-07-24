use i2cdev::core::*;
use std::time::Duration;
// NOTE:
// add target to .cargo/config.toml
// [build]
// target = "aarch64-unknown-linux-gnu"
// because I'm on Mac -> this to tell compiler this is meant for the linux on the pi
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::thread;

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

        // NOTE: move wheel motor command is 0x01
        self.write_array(0x01, &data)?;

        Ok(())
    }

    fn stop_all_wheels(&mut self) -> Result<(), LinuxI2CError> {
        let motor_ids = [MotorId::Zero, MotorId::One, MotorId::Two, MotorId::Three];
        for id in motor_ids {
            self.move_wheel_motor(MotorInfo {
                id,
                direction: MotorDirection::Backward,
                speed: 0,
            })?;
        }
        Ok(())
    }

    fn move_bot_right(&mut self, speed: u8) -> Result<(), LinuxI2CError> {
        // 0 / 2 move up
        self.move_wheel_motor(MotorInfo {
            id: MotorId::Zero,
            direction: MotorDirection::Forward,
            speed,
        })?;
        self.move_wheel_motor(MotorInfo {
            id: MotorId::Three,
            direction: MotorDirection::Forward,
            speed,
        })?;

        // 1 / 3 move back
        self.move_wheel_motor(MotorInfo {
            id: MotorId::One,
            direction: MotorDirection::Backward,
            speed,
        })?;
        self.move_wheel_motor(MotorInfo {
            id: MotorId::Two,
            direction: MotorDirection::Backward,
            speed,
        })?;

        Ok(())
    }

    fn move_bot_left(&mut self, speed: u8) -> Result<(), LinuxI2CError> {
        // 0 / 2 move back
        self.move_wheel_motor(MotorInfo {
            id: MotorId::Zero,
            direction: MotorDirection::Backward,
            speed,
        })?;
        self.move_wheel_motor(MotorInfo {
            id: MotorId::Three,
            direction: MotorDirection::Backward,
            speed,
        })?;

        // 1 / 3 move up
        self.move_wheel_motor(MotorInfo {
            id: MotorId::One,
            direction: MotorDirection::Forward,
            speed,
        })?;
        self.move_wheel_motor(MotorInfo {
            id: MotorId::Two,
            direction: MotorDirection::Forward,
            speed,
        })?;

        Ok(())
    }
}

fn main() -> Result<(), LinuxI2CError> {
    // initiate a new device
    let device = LinuxI2CDevice::new("/dev/i2c-1", RASBOT_I2C_ADDRESS)?;
    // give it to the bot struct
    let mut bot = Bot::new(RASBOT_I2C_ADDRESS, device);

    // move all wheels
    let motor_ids = [MotorId::Zero, MotorId::One, MotorId::Two, MotorId::Three];
    for id in motor_ids {
        bot.move_wheel_motor(MotorInfo {
            id,
            direction: MotorDirection::Forward,
            speed: 255,
        })?;
    }

    // run for about 10 seconds
    thread::sleep(Duration::from_secs(10));

    // stop all wheels
    bot.stop_all_wheels()?;

    // move bot right for 3 secs
    bot.move_bot_right(255)?;
    thread::sleep(Duration::from_secs(3));
    bot.stop_all_wheels()?;

    Ok(())
}
