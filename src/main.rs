use kitchen::Kitchen;

mod food;
mod handler;
mod kitchen;

fn main() {
    let mut kitchen = Kitchen::new(vec![]);
    kitchen.run();
}
