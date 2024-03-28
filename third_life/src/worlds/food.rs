pub mod components;
use self::components::*;
pub mod events;
use self::events::*;
use self::tracking::FoodProductionTrackingPlugin;
pub mod cow_farming;
pub mod tracking;
pub mod wheat_farming;
use crate::time::GameDate;
use crate::worlds::food::{cow_farming::*, wheat_farming::*};

use std::usize;

use crate::SimulationState;
use bevy::prelude::*;
use chrono::Months;
use rand_distr::num_traits::Float;

use super::config::WorldConfig;
use super::{init_colonies, WorldColony};

pub struct FoodPlugin;
impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            init_food.after(init_colonies),
        )
        .add_systems(
            Update,
            (
                season_check_wheat,
                mark_breeders,
                breed_cows,
                check_farm_workers,
                get_farm_workers,
                check_wheat_farms_counts,
                work_farm,
                check_cow_farm_workers,
                get_cow_farm_workers,
                check_for_more_cow_farms,
                work_cow_farm,
                butcher_cows,
                /*add_cow_bundles,
                add_breeder_bundles,
                spawn_cows,
                spawn_bulls,*/
            )
                .run_if(in_state(SimulationState::Running)),
        )
        .add_plugins(FoodProductionTrackingPlugin)
        .add_event::<WheatFarmNeedsWorker>()
        .add_event::<CowFarmNeedsWorker>()
        .add_event::<WheatFarmCreated>()
        .add_event::<WheatFarmRemoved>()
        .add_event::<CowFarmCreated>()
        .add_event::<CowFarmRemoved>()
        .add_event::<CarbCreated>()
        .add_event::<MeatCreated>()
        .add_event::<MeatConsumedEvent>()
        .add_event::<CarbConsumedEvent>()
        .add_event::<FoodCreated>()
        .init_resource::<CowsToSpawn>()
        .init_resource::<BullsToSpawn>();
    }
}

fn init_food(
    mut commands: Commands,
    game_date: Res<GameDate>,
    mut colonies: Query<(Entity, &mut WorldColony, &WorldConfig), With<WorldColony>>,
) {
    for (colony_entity, mut world_colony, world_config) in colonies.iter_mut() {
        let mut wheat_farms = Vec::new();
        let wheat_farm_size = world_config.food().wheat_farm_size();
        for _ in 0..world_config.food().wheat_farms() {
            if world_colony.space_left() > wheat_farm_size {
                world_colony.take_up_farm_space(wheat_farm_size);
                wheat_farms.push((
                    WheatFarm {
                        size: wheat_farm_size,
                        harvested: wheat_farm_size,
                    },
                    WheatFarmOf {
                        colony: colony_entity,
                    },
                ));
            }
        }
        commands.spawn_batch(wheat_farms);

        let cow_farm_size = world_config.food().cow_farm_size();
        for _ in 0..world_config.food().cow_farms() {
            if world_colony.space_left() > cow_farm_size {
                world_colony.take_up_farm_space(cow_farm_size);
                let cow_farm_entity = commands
                    .spawn((
                        CowFarm { size: cow_farm_size, farmers_wanted: 4, hours_worked: 0., },
                        CowFarmOf {
                            colony: colony_entity,
                        },
                    ))
                    .id();
                let mut cows = Vec::new();
                let mut bulls = Vec::new();
                //47 is min starting cows and we want to have 10 ready to harvest right away
                let total_cows = 57.0;
                let total_bulls = (total_cows / 25.0).ceil() as usize;
                for _ in 0..total_bulls {
                    bulls.push((
                        Cow {
                            birthday: game_date.date - Months::new(24),
                        },
                        IsBull,
                        IsBreeder,
                        CowOf {
                            cow_farm: cow_farm_entity,
                        },
                    ))
                }
                commands.spawn_batch(bulls);
                for _ in 0..(total_cows as usize - total_bulls) {
                    cows.push((
                        Cow {
                            birthday: game_date.date - Months::new(24),
                        },
                        CowOf {
                            cow_farm: cow_farm_entity,
                        },
                    ))
                }
                commands.spawn_batch(cows);
            }
        }

        commands.spawn((
            CarbResource::from_amount(world_config.food().starting_carb()),
            ResourceOf {
                colony: colony_entity,
            },
        ));
        commands.spawn((
            MeatResource::from_amount(world_config.food().starting_carb()),
            ResourceOf {
                colony: colony_entity,
            },
        ));
    }
}




