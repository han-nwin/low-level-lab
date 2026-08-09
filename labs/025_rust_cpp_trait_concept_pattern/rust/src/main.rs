trait Led {
    fn turn_on(&mut self);
    fn turn_off(&mut self);
    fn is_on(&self) -> bool;
}

struct ConsoleLed {
    on: bool,
}

struct MemoryLed {
    on: bool,
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

fn blink_once<T: Led>(led: &mut T) {
    led.turn_on();
    println!("LED state: {}", led.is_on());
    led.turn_off();
    println!("LED state: {}", led.is_on());
}

fn main() {
    let mut console = ConsoleLed { on: false };
    let mut memory = MemoryLed {
        on: false,
        toggle_count: 0,
    };
    blink_once(&mut console);
    blink_once(&mut memory);
    println!("toggles: {}", memory.toggle_count);
}
