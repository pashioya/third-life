pub mod components;
use self::components::*;
pub mod events;
use self::events::*;
use self::tracking::CarbProduced;
use self::tracking::FoodProductionTrackingPlugin;
use self::tracking::MeatProduced;
pub mod cow_farming;
pub mod tracking;
pub mod wheat_farming;
use crate::time::GameDate;
use crate::time::YearChanged;
use crate::worlds::food::{cow_farming::*, wheat_farming::*};

use std::usize;

use crate::{time::DateChanged, SimulationState};
use bevy::ecs::world;
use bevy::{prelude::*, reflect::List, utils::HashMap};
use bevy_egui::{egui::Window, EguiContexts};
use chrono::Months;
use rand_distr::num_traits::Float;

use super::config::WorldConfig;
use super::population::components::CarbConsumed;
use super::population::components::MeatConsumed;
use super::{init_colonies, population::components::CitizenOf, WorldColony};

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
                check_for_more_wheat_farms,
                work_farm,
                check_cow_farm_workers,
                get_cow_farm_workers,
                check_for_more_cow_farms,
                work_cow_farm,
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
        .add_event::<FoodCreated>();
    }
}

fn init_food(
    mut commands: Commands,
    game_date: Res<GameDate>,
    colonies: Query<(Entity, &WorldConfig), With<WorldColony>>,
) {
    for (colony_entity, world_config) in colonies.iter() {
        let mut wheat_farms = Vec::new();
        for _ in 0..world_config.food().wheat_farms() {
            wheat_farms.push((
                WheatFarm {
                    size: 17.4,
                    harvested: 17.4,
                },
                WheatFarmOf {
                    colony: colony_entity,
                },
            ));
        }
        commands.spawn_batch(wheat_farms);

        for _ in 0..world_config.food().cow_farms() {
            let cow_farm_entity = commands
                .spawn((
                    CowFarm { size: 34.0 },
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

fn check_for_more_wheat_farms(
    mut commands: Commands,
    mut colonies: Query<(Entity, &mut CarbConsumed, &mut CarbProduced)>,
    mut year_changed_rader: EventReader<YearChanged>,
) {
    for _ in year_changed_rader.read() {
        for (colony, mut carb_consumed, mut carb_produced) in colonies.iter_mut() {
            if carb_produced.amount <= carb_consumed.amount {
                commands.spawn((
                    WheatFarm {
                        size: 17.4,
                        harvested: 17.4,
                    },
                    WheatFarmOf { colony },
                ));
            }
            carb_consumed.amount = 0.0;
            carb_produced.amount = 0.0;
        }
    }
}

fn check_for_more_cow_farms(
    mut commands: Commands,
    mut colonies: Query<(Entity, &mut MeatConsumed, &mut MeatProduced)>,
    mut year_changed_rader: EventReader<YearChanged>,
    game_date: Res<GameDate>
) {
    for _ in year_changed_rader.read() {
        for (colony, mut meat_consumed, mut meat_produced) in colonies.iter_mut() {
            if meat_produced.amount <= meat_consumed.amount {
                let cow_farm_entity = commands
                    .spawn((
                        CowFarm { size: 34.0 },
                        CowFarmOf {
                            colony,
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
                            birthday: game_date.date - Months::new(12),
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
                            birthday: game_date.date - Months::new(12),
                        },
                        CowOf {
                            cow_farm: cow_farm_entity,
                        },
                    ))
                }
                commands.spawn_batch(cows);
            }
            meat_produced.amount = 0.0;
            meat_consumed.amount = 0.0;
        }
    }
}
