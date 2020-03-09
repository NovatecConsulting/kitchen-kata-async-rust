use crate::food::{CookingStep, Food, COOKING_STEPS};
use async_std::{
    sync::{channel, Receiver, Sender},
    task,
};
use rand::Rng;
use std::time::Duration;

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
    async fn handle(self, food: &mut Food, step: CookingStep) -> Result<(), &'static str> {
        println!("{} some {}", step, food.name);
        let mut rng = rand::thread_rng();
        task::sleep(Duration::from_secs(rng.gen_range(1, 4))).await;
        if rng.gen_range(0, 100) < self.error_chance {
            Err("Oops!")
        } else {
            Ok(())
        }
    }
}

pub struct Station {
    handler: Handler,
    handles_step: CookingStep,
    pub input: (Sender<Food>, Receiver<Food>),
    pub output: (Sender<Food>, Receiver<Food>),
}

impl Station {
    pub fn new(handles_step: CookingStep) -> Self {
        Self::with_handler(handles_step, Handler::new())
    }
    pub fn with_handler(handles_step: CookingStep, handler: Handler) -> Self {
        Self {
            handles_step,
            handler,
            input: channel(1),
            output: channel(1),
        }
    }
    pub fn all_stations() -> Vec<Self> {
        COOKING_STEPS
            .iter()
            .map(|cooking_step| Self::new(*cooking_step))
            .collect()
    }
    #[cfg(test)]
    pub fn all_stations_with_handler(handler: Handler) -> Vec<Self> {
        COOKING_STEPS
            .iter()
            .map(|cooking_step| Self::with_handler(*cooking_step, handler))
            .collect()
    }
    pub async fn prepare(&self) {
        while let Some(mut food) = self.input.1.recv().await {
            let step = food.cooking_steps.pop_front().unwrap();
            if food.borked {
                println!(
                    "This {} is {}, we can't be {} that!",
                    food.name,
                    food.conditions.last().unwrap(),
                    step
                );
            } else {
                if let Err(e) = self.handler.handle(&mut food, step).await {
                    println!(
                        "Something went wrong while {} {}: [{}]",
                        self.handles_step, food.name, e
                    );
                    food.add_mishap(self.handles_step);
                } else {
                    food.add_step_result(step);
                }
            }
            self.output.0.send(food).await;
        }
    }
    pub fn can_prepare(&self, food: &Food) -> bool {
        food.cooking_steps.get(0) == Some(&self.handles_step)
    }
}
