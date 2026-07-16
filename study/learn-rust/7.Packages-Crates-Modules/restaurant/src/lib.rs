// declare the 2 module
mod back_of_house;
pub mod front_of_house;

pub use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    // Access through a re-exported module shortcut.
    hosting::add_to_waitlist();

    // Access through an absolute path from the crate root.
    crate::front_of_house::hosting::seat_at_table();

    let mut meal = back_of_house::Breakfast::summer("Rye");
    meal.toast = String::from("Wheat");
    println!("I'd like {} toast, please.", meal.toast);
    println!("Seasonal fruit: {}", meal.seasonal_fruit());

    let _starter = back_of_house::Appetizer::Soup;
    let _other_starter = back_of_house::Appetizer::Salad;

    back_of_house::fix_incorrect_order();
}

fn deliver_order() {
    println!("order delivered");
}
