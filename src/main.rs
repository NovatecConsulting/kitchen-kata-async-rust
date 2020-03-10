use food::{CookingStep, Food};
use kitchen::Kitchen;

mod food;
mod handler;
mod kitchen;

fn main() {
    let mut kitchen = Kitchen::new(vec![Food::new(
        "Potatoes",
        vec![CookingStep::Cut, CookingStep::Spice, CookingStep::Grill],
    )]);
    kitchen.run();
}
