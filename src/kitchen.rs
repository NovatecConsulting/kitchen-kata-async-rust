use crate::{food::Food, handler::Station};

pub struct Kitchen {
    stations: Vec<Station>,
    food_to_prepare: Vec<Food>,
    finished_meal: Vec<Food>,
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
        while !self.food_to_prepare.is_empty() {
            self.find_more_work();
        }
    }
    fn find_more_work(&mut self) {
        let (done, todo) = self
            .food_to_prepare
            .drain(..)
            .partition::<Vec<_>, _>(|food| food.cooking_steps.is_empty());
        self.finished_meal.extend(done);
        self.food_to_prepare = todo;

        for food in &mut self.food_to_prepare {
            for station in &self.stations {
                if station.can_prepare(food) {
                    station.prepare(food);
                    return;
                }
            }
        }

        if !self.food_to_prepare.is_empty() {
            println!("DEADLOCK DETECTED");
            println!("SHUT IT DOWN");
            panic!("Deadlock")
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::{food::CookingStep, handler::Handler};

    #[test]
    fn food_is_cooked_correctly() {
        let mut kitchen = Kitchen::with_stations(
            vec![potatoes(), steak(), cheese(), fruit_cake()],
            Station::all_stations_with_handler(PERFECT_HANDLER),
        );
        kitchen.run();

        assert_eq!(kitchen.finished_meal.len(), 4);

        for finished_meal in &kitchen.finished_meal {
            assert!(!finished_meal.borked);
        }
    }

    #[test]
    fn burned_potatoes() {
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
