use crate::{food::Food, handler::Station};
use futures::{future::select_all, FutureExt};
use smol::Task;
use std::rc::Rc;

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
        smol::run(self.find_more_work())
    }
    async fn find_more_work(&mut self) {
        let food_amount = self.food_to_prepare.len();
        let (send_to_prepare, recv_to_prepare) = piper::chan(food_amount);
        let (send_done, recv_done) = piper::chan(1);

        let mut station_handles = vec![];

        let mut station_futs = select_all(self.stations.drain(..).into_iter().map(|station| {
            let station = Rc::new(station);
            let send_done = send_done.clone();
            station_handles.push(station.clone());
            Task::local(async move { station.prepare(send_done).await })
        }))
        .fuse();

        let mut station_feeder = Task::local(async move {
            while let Some(food_to_prepare) = recv_to_prepare.recv().await {
                if let Some(station) = station_handles
                    .iter()
                    .find(|it| it.can_prepare(&food_to_prepare))
                {
                    station.send(food_to_prepare).await;
                }
            }
        })
        .fuse();

        for food_to_prepare in self.food_to_prepare.drain(..) {
            send_to_prepare.send(food_to_prepare).await;
        }

        let (send_finished, recv_finished) = piper::chan(1);

        let mut output_distributor = Task::local(async move {
            while let Some(done_food) = recv_done.recv().await {
                if done_food.cooking_steps.is_empty() {
                    send_finished.send(done_food).await;
                } else {
                    send_to_prepare.send(done_food).await;
                }
            }
        })
        .fuse();

        futures::select!(
            _ = station_futs => (),
            _ = station_feeder => (),
            _ = output_distributor => ()
        );

        while let Some(finished_food) = recv_finished.recv().await {
            self.finished_meal.push(finished_food);
            if self.finished_meal.len() == food_amount {
                break;
            }
        }
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
            .with_max_level(Level::TRACE)
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
