use crate::food::{CookingStep, Food, COOKING_STEPS};
use rand::Rng;
use std::{thread, time::Duration};

#[derive(Clone, Copy)]
pub struct Handler {
    pub error_chance: u8,
}

impl Handler {
    fn new() -> Self {
        Self::with_error_chance(10)
    }
    fn with_error_chance(error_chance: u8) -> Self {
        Handler { error_chance }
    }
    fn handle(self, food: &mut Food, step: CookingStep) -> Result<(), &'static str> {
        println!("{} some {}", step, food.name);
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_secs(rng.gen_range(1, 4)));
        if rng.gen_range(0, 100) <= self.error_chance {
            Err("Oops!")
        } else {
            Ok(())
        }
    }
}

pub struct Station {
    handler: Handler,
    handles_step: CookingStep,
}

impl Station {
    pub fn new(handles_step: CookingStep) -> Self {
        Self {
            handles_step,
            handler: Handler::new(),
        }
    }
    pub fn with_handler(handles_step: CookingStep, handler: Handler) -> Self {
        Self {
            handles_step,
            handler,
        }
    }
    pub fn all_stations() -> Vec<Self> {
        COOKING_STEPS
            .iter()
            .map(|cooking_step| Self::new(*cooking_step))
            .collect()
    }
    pub fn all_stations_with_handler(handler: Handler) -> Vec<Self> {
        COOKING_STEPS
            .iter()
            .map(|cooking_step| Self::with_handler(*cooking_step, handler))
            .collect()
    }
    pub fn prepare(&self, food: &mut Food) {
        let step = food.cooking_steps.pop_front().unwrap();
        if food.borked {
            println!(
                "This {} is {}, we can't be {} that!",
                food.name,
                food.conditions.last().unwrap(),
                step
            );
        } else {
            if let Err(e) = self.handler.handle(food, step) {
                println!(
                    "Something went wrong while {} {}: [{}]",
                    self.handles_step, food.name, e
                );
                food.add_mishap(self.handles_step);
            } else {
                food.add_step_result(step);
            }
        }
    }
    pub fn can_prepare(&self, food: &Food) -> bool {
        food.cooking_steps.get(0) == Some(&self.handles_step)
    }
}
