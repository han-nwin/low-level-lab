trait Led {
    fn turn_on(&mut self);
    fn turn_off(&mut self);
    fn is_on(&self) -> bool;
}

trait Dimmable {
    fn set_brightness(&mut self, value: u8);
}

struct ConsoleLed {
    on: bool,
}

struct MemoryLed {
    on: bool,
    brightness: u8,
    toggle_count: u32,
}

impl Led for ConsoleLed {
    fn turn_on(&mut self) {
        if !self.on {
            self.on = true;
        }
    }

    fn turn_off(&mut self) {
        if self.on {
            self.on = false;
        }
    }

    fn is_on(&self) -> bool {
        self.on
    }
}

impl Led for MemoryLed {
    fn turn_on(&mut self) {
        if !self.on {
            self.on = true;
            self.toggle_count += 1;
        }
    }

    fn turn_off(&mut self) {
        if self.on {
            self.on = false;
            self.toggle_count += 1;
        }
    }

    fn is_on(&self) -> bool {
        self.on
    }
}

impl Dimmable for MemoryLed {
    fn set_brightness(&mut self, value: u8) {
        self.brightness = value;
    }
}

fn blink_once<T: Led>(led: &mut T) {
    led.turn_on();
    println!("LED state: {}", led.is_on());
    led.turn_off();
    println!("LED state: {}", led.is_on());
}

fn test_dimmable<L>(led: &mut L, value: u8)
where
    L: Led + Dimmable,
{
    led.set_brightness(value);
}

fn main() {
    let mut console = ConsoleLed { on: false };
    let mut memory = MemoryLed {
        on: false,
        brightness: 0,
        toggle_count: 0,
    };
    blink_once(&mut console);
    blink_once(&mut memory);
    println!("toggles: {}", memory.toggle_count);

    test_dimmable(&mut memory, 100);
    println!("brightness: {}", memory.brightness);
}
