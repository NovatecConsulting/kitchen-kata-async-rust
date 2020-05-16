use crate::food::{CookingStep, Food, COOKING_STEPS};
use anyhow::Result;
use rand::Rng;
use smol::Timer;
use std::{
    fmt::{self, Debug},
    time::Duration,
};
use tracing::{info, warn};

#[derive(Clone, Copy, Debug)]
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
    async fn handle(&mut self, food: &mut Food, step: CookingStep) -> Result<(), &'static str> {
        info!("{} some {}", step, food.name);
        //rand::thread_rng().gen_range(1, 4);
        let sleep_duration = Duration::from_secs(1);
        Timer::after(sleep_duration).await;
        if rand::thread_rng().gen_range(0, 100) < self.error_chance {
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

impl fmt::Display for Station {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} Station ({}% failure)",
            self.handles_step, self.handler.error_chance
        )
    }
}

impl Station {
    pub fn new(handles_step: CookingStep) -> Self {
        Self::with_handler(handles_step, Handler::new())
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
    #[cfg(test)]
    pub fn all_stations_with_handler(handler: Handler) -> Vec<Self> {
        COOKING_STEPS
            .iter()
            .map(|cooking_step| Self::with_handler(*cooking_step, handler))
            .collect()
    }
    pub async fn prepare(&mut self, mut food: Food) -> Food {
        let step = food.cooking_steps.pop_front().unwrap();
        if food.borked {
            warn!(
                "This {} is {}, we can't be {} that!",
                food.name,
                food.conditions.last().unwrap(),
                step
            );
        } else if let Err(e) = self.handler.handle(&mut food, step).await {
            warn!(
                "Something went wrong while {} {}: [{}]",
                self.handles_step, food.name, e
            );
            food.add_mishap(self.handles_step);
        } else {
            food.add_step_result(step);
        }
        food
    }
    pub fn can_prepare(&self, food: &Food) -> bool {
        food.cooking_steps.get(0) == Some(&self.handles_step)
    }
}
