fn cook_order() {
    println!("order cooked");
}

pub fn fix_incorrect_order() {
    cook_order();
    super::deliver_order();
}

pub struct Breakfast {
    pub toast: String,
    seasonal_fruit: String,
}

impl Breakfast {
    pub fn summer(toast: &str) -> Self {
        Self {
            toast: toast.to_string(),
            seasonal_fruit: String::from("peaches"),
        }
    }

    pub fn seasonal_fruit(&self) -> &str {
        &self.seasonal_fruit
    }
}

pub enum Appetizer {
    Soup,
    Salad,
}
