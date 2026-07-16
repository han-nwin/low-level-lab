use characteristics::gui::{Button, Draw, Screen};
use std::string::String;

// User write their own type
struct SelectBox {
    width: u32,
    height: u32,
    options: Vec<String>,
}

impl Draw for SelectBox {
    fn draw(&self) {
        let width = self.width as usize;
        let height = self.height as usize;

        if width < 2 || height < 2 {
            println!("Width and height must be at least 2.");
            return;
        }

        println!("┌{}┐", "─".repeat(width - 2));

        for _ in 0..height / 2 - 1 {
            println!("│{}│", " ".repeat(width - 2));
        }
        for option in &self.options {
            if option.len().is_multiple_of(2) {
                println!(
                    "│{}{option}👇🏻{}│",
                    " ".repeat((width - option.len()) / 2 - 2),
                    " ".repeat((width - option.len()) / 2 - 2)
                );
            } else {
                println!(
                    "│{}{option}👇🏻{}│",
                    " ".repeat((width - option.len()) / 2 - 1),
                    " ".repeat((width - option.len()) / 2 - 2)
                );
            }
        }
        for _ in 0..height / 2 - 1 {
            println!("│{}│", " ".repeat(width - 2));
        }

        println!("└{}┘", "─".repeat(width - 2));
    }
}

fn main() {
    let button = Button {
        width: 10,
        height: 10,
        label: "Hello".to_string(),
    };
    button.draw();

    let select_box = SelectBox {
        width: 20,
        height: 10,
        options: vec!["option 12".to_string(), "option 2".to_string()],
    };
    select_box.draw();

    println!("Create a screen");
    let screen = Screen {
        components: vec![
            Box::new(SelectBox {
                width: 20,
                height: 10,
                options: vec![
                    String::from("Yes"),
                    String::from("Maybe"),
                    String::from("No"),
                ],
            }),
            Box::new(Button {
                width: 10,
                height: 3,
                label: String::from("Ok"),
            }),
            Box::new(String::from("Hello")),
        ],
    };
    screen.run();
}
