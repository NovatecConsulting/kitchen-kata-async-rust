use crate::{food::Food, handler::Station};
use futures::{stream::FuturesUnordered, StreamExt};
use smol::Task;
use std::fmt;
use tracing::debug;

pub struct Kitchen {
    stations: Vec<Station>,
    food_to_prepare: Vec<Food>,
    finished_meal: Vec<Food>,
}

fn show_vec(v: &Vec<impl fmt::Display>) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

impl fmt::Debug for Kitchen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("Kitchen");
        builder.field("stations", &show_vec(&self.stations));
        if !self.food_to_prepare.is_empty() {
            builder.field("food_to_prepare", &show_vec(&self.food_to_prepare));
        }

        if !self.finished_meal.is_empty() {
            builder.field("finished_meal", &show_vec(&self.finished_meal));
        }
        builder.finish()
    }
}

impl Kitchen {
    pub fn new(food_to_prepare: Vec<Food>) -> Self {
        Self::with_stations(food_to_prepare, Station::all_stations())
    }
    pub fn with_stations(food_to_prepare: Vec<Food>, stations: Vec<Station>) -> Self {
        Self {
            stations,
            food_to_prepare,
            finished_meal: vec![],
        }
    }
    pub fn run(&mut self) {
        smol::run(self.find_more_work())
    }
    async fn find_more_work(&mut self) {
        debug!("Starting kitchen: {:?}", self);
        let mut working_stations = FuturesUnordered::new();
        let mut stations: Vec<_> = self.stations.drain(..).collect();
        let mut food_to_prepare: Vec<Food> = self.food_to_prepare.drain(..).collect();
        while !food_to_prepare.is_empty() || !working_stations.is_empty() {
            let mut unassignable_food = Vec::new();
            let mut idle_stations = Vec::new();
            'food: loop {
                debug!(
                    "Putting {} back in {}",
                    show_vec(&idle_stations),
                    show_vec(&stations)
                );
                stations.extend(idle_stations.drain(..));
                match food_to_prepare.pop() {
                    None => break,
                    Some(food) => {
                        debug!("Attempting to handle {} with {}", food, show_vec(&stations));
                        while let Some(mut station) = stations.pop() {
                            if station.can_prepare(&food) {
                                debug!("Sending {} to {}", food, station);
                                working_stations.push(Task::spawn(async move {
                                    let food = station.prepare(food).await;
                                    (food, station)
                                }));
                                continue 'food;
                            } else {
                                idle_stations.push(station);
                            }
                        }
                        debug!(
                            "Couldn't handle {} right now, no free station for {:?}",
                            food,
                            food.cooking_steps.front()
                        );
                        unassignable_food.push(food);
                    }
                }
            }
            food_to_prepare.extend(unassignable_food.drain(..));
            if let Some((food, station)) = working_stations.next().await {
                stations.push(station);
                if food.has_steps_left() {
                    debug!("Queueing {} again", food);
                    food_to_prepare.push(food);
                } else {
                    debug!("Completed {}", food);
                    self.finished_meal.push(food);
                }
            }
        }
        self.stations.extend(stations.drain(..));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{food::CookingStep, handler::Handler};
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    fn init_tracing() {
        // a builder for `FmtSubscriber`.
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::INFO)
            // completes the builder.
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    #[test]
    fn food_is_cooked_correctly() {
        init_tracing();
        let mut kitchen = Kitchen::with_stations(
            vec![potatoes(), steak(), cheese(), fruit_cake()],
            Station::all_stations_with_handler(PERFECT_HANDLER),
        );
        kitchen.run();

        debug!("Kitchen after run: {:?}", kitchen);
        assert_eq!(kitchen.finished_meal.len(), 4);

        for finished_meal in &kitchen.finished_meal {
            assert!(!finished_meal.borked);
        }
    }

    #[test]
    fn burned_potatoes() {
        init_tracing();
        let mut kitchen = Kitchen::with_stations(
            vec![potatoes()],
            vec![
                Station::with_handler(CookingStep::Peel, PERFECT_HANDLER),
                Station::with_handler(CookingStep::Grill, FAILING_HANDLER),
            ],
        );

        kitchen.run();

        assert_eq!(kitchen.finished_meal.len(), 1);
        assert!(kitchen.finished_meal.first().unwrap().borked);
        assert_eq!(
            kitchen.finished_meal.first().unwrap().to_string(),
            "Peeled and burned Potatoes"
        );
    }

    #[test]
    fn steak_fails_and_potatoes_succeed() {
        init_tracing();
        let mut kitchen = Kitchen::with_stations(
            vec![potatoes(), steak()],
            vec![
                Station::with_handler(CookingStep::Cut, PERFECT_HANDLER),
                Station::with_handler(CookingStep::Peel, PERFECT_HANDLER),
                Station::with_handler(CookingStep::Spice, FAILING_HANDLER),
                Station::with_handler(CookingStep::Grill, PERFECT_HANDLER),
            ],
        );

        kitchen.run();

        assert_eq!(kitchen.finished_meal.len(), 2);

        let steak = kitchen
            .finished_meal
            .iter()
            .find(|meal| meal.name == "Steak")
            .unwrap();
        let potatoes = kitchen
            .finished_meal
            .iter()
            .find(|meal| meal.name == "Potatoes")
            .unwrap();

        assert!(steak.borked);
        assert_eq!(steak.to_string(), "Cut and pepper-covered Steak");

        assert!(!potatoes.borked);
        assert_eq!(potatoes.to_string(), "Peeled and grilled Potatoes");
    }

    static PERFECT_HANDLER: Handler = Handler { error_chance: 0 };
    static FAILING_HANDLER: Handler = Handler { error_chance: 100 };

    fn potatoes() -> Food {
        Food::new("Potatoes", vec![CookingStep::Peel, CookingStep::Grill])
    }

    fn steak() -> Food {
        Food::new(
            "Steak",
            vec![CookingStep::Cut, CookingStep::Spice, CookingStep::Grill],
        )
    }

    fn cheese() -> Food {
        Food::new("Cheese", vec![CookingStep::Spice, CookingStep::Grill])
    }

    fn fruit_cake() -> Food {
        Food::new(
            "Fruit Cake",
            vec![
                CookingStep::Peel,
                CookingStep::Cut,
                CookingStep::Spice,
                CookingStep::Bake,
            ],
        )
    }
}
