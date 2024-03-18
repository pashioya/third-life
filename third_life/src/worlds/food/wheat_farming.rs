use bevy::{prelude::*, utils::{hashbrown::HashMap}};
use chrono::{Datelike, NaiveDate};

use crate::{
    time::{DateChanged, GameDate, YearChanged},
    worlds::{
        config::WorldConfig,
        population::components::{CarbConsumed, Citizen, CitizenOf, Employed},
        WorldColony,
    },
};

use super::{
    tracking::CarbProduced, CarbCreated, CarbResource, CowFarmer, ResourceOf, WheatFarm,
    WheatFarmNeedsWorker, WheatFarmOf, WheatFarmer,
};

pub fn season_check_wheat(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut wheat_farms: Query<&mut WheatFarm>,
    game_date: Res<GameDate>,
) {
    for _ in day_changed_event_reader.read() {
        if game_date.date.month() == 6 && game_date.date.day() == 1 {
            warn!("Harvest season has begun {:?}", game_date.date);
            for mut wheat_farm in wheat_farms.iter_mut() {
                wheat_farm.harvested = 0.0;
            }
        }
    }
}


pub fn check_wheat_farms_counts(
    mut commands: Commands,
    wheat_farms: Query<(Entity, &WheatFarmOf), With<WheatFarm>>,
    farm_workers: Query<(Entity, &WheatFarmer)>,
    mut colonies: Query<(
        Entity,
        &mut WorldColony,
        &mut CarbConsumed,
        &mut CarbProduced,
        &WorldConfig,
    )>,
    carb_resources: Query<(&ResourceOf, &CarbResource)>,
    mut year_changed_reader: EventReader<YearChanged>,
) {
    let resource_map = carb_resources
        .iter()
        .map(|(e, r)| (e.colony, r))
        .collect::<HashMap<_, _>>();

    let mut farms_map = wheat_farms
        .iter()
        .fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, Vec<Entity>>, (farm_entity, wheat_farm_of)| {
                acc.entry(wheat_farm_of.colony).or_insert(Vec::new()).push(farm_entity);
                acc
            },
        );

    let farmers_map = farm_workers
        .iter()
        .fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, Vec<Entity>>, (farmer_entity, wheat_farmer)| {
                acc.entry(wheat_farmer.farm).or_insert(Vec::new()).push(farmer_entity);
                acc
            },
        );
    
    for _ in year_changed_reader.read() {
        for (colony, mut world_colony, mut carb_consumed, mut carb_produced, world_config) in
            colonies.iter_mut()
        {
            let min_surplus = world_config.food().min_surplus_multiplier();
            if carb_consumed.amount * min_surplus > resource_map.get(&colony).unwrap().get_kgs() {
                let wheat_farm_size = 17.4;
                if world_colony.space_left() > wheat_farm_size {
                    world_colony.used += wheat_farm_size;
                    commands.spawn((
                        WheatFarm {
                            size: wheat_farm_size,
                            harvested: wheat_farm_size,
                        },
                        WheatFarmOf { colony },
                    ));
                }
            }

            /* this could be used for despawning later
            let max_surplus = world_config.food().max_surplus_multiplier();

            if carb_consumed.amount*max_surplus < resource_map.get(&colony).unwrap().get_kgs() {
                //TODO why is this Vec not mutable?
                if let Some(farm) = farms_map.get_mut(&colony).unwrap().pop() {
                    for farmer_entity in farmers_map.get(&farm).unwrap() {
                        let _ = commands.get_entity(*farmer_entity)
                            .map(|mut e| {
                                e.remove::<WheatFarmer>();
                                e.remove::<Employed>();
                            });
                    } 
                    commands.get_entity(farm).map(|mut e| {
                        e.despawn(); 
                    });
                    world_colony.used -= wheat_farm_size;
                }
            }*/

            carb_consumed.amount = 0.0;
            carb_produced.amount = 0.0;
        }
    }
}
pub fn check_farm_workers(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<WheatFarmNeedsWorker>,
    wheat_farms: Query<(Entity, &WheatFarmOf), With<WheatFarm>>,
    farmers: Query<(&WheatFarmer, &CitizenOf)>,
) {
    for _ in day_changed_event_reader.read() {
        let mut farms_map = wheat_farms.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, wheat_farm_of)| {
                acc.entry(wheat_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(0);
                acc
            },
        );

        for (wheat_farmer, colony_of) in farmers.iter() {
            farms_map
                .get_mut(&colony_of.colony)
                .unwrap()
                .entry(wheat_farmer.farm)
                .and_modify(|count| *count += 1);
        }

        for (colony, farms) in farms_map {
            for (farm, farmer_count) in farms {
                if farmer_count < 4 {
                    for _ in 0..(4 - farmer_count) {
                        event_writer.send(WheatFarmNeedsWorker { colony, farm });
                    }
                }
            }
        }
    }
}

pub fn get_farm_workers(
    mut commands: Commands,
    mut event_reader: EventReader<WheatFarmNeedsWorker>,
    free_citizens: Query<(Entity, &CitizenOf), Without<Employed>>,
) {
    for needs_worker_event in event_reader.read() {
        for (citizen, citizen_of) in free_citizens.iter() {
            if citizen_of.colony == needs_worker_event.colony {
                commands.get_entity(citizen).map(|mut c| {
                    c.try_insert((
                        WheatFarmer {
                            farm: needs_worker_event.farm,
                        },
                        Employed,
                    ));
                });
                break;
            }
        }
    }
}

pub fn work_farm(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut wheat_farms: Query<(Entity, &mut WheatFarm, &WheatFarmOf)>,
    farmers: Query<(&WheatFarmer, &CitizenOf)>,
    mut carb_resources: Query<(&mut CarbResource, &ResourceOf)>,
    mut carb_created: EventWriter<CarbCreated>,
) {
    for _ in day_changed_event_reader.read() {
        let mut farms_map = wheat_farms.iter_mut().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, _, wheat_farm_of)| {
                acc.entry(wheat_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(0);
                acc
            },
        );

        for (wheat_farmer, colony_of) in farmers.iter() {
            farms_map
                .get_mut(&colony_of.colony)
                .unwrap()
                .entry(wheat_farmer.farm)
                .and_modify(|count| *count += 1);
        }

        for (colony, farms) in farms_map {
            for (farm_entity, farmer_count) in farms {
                let (_, mut wheat_farm, _) = wheat_farms.get_mut(farm_entity).unwrap();
                // 1.0 signifies multiplier for 1 8 hour work day
                // harvested_amount is in ha
                let mut harvested_amount = 1.0 * (farmer_count as f32);
                if harvested_amount > wheat_farm.size - wheat_farm.harvested {
                    harvested_amount = wheat_farm.size - wheat_farm.harvested;
                }
                wheat_farm.harvested += harvested_amount;
                if harvested_amount > 0.0 {
                    for (mut carb_resource, resource_of) in carb_resources.iter_mut() {
                        if resource_of.colony == colony {
                            let amount = harvested_amount * 2670.0;
                            carb_resource.add_kgs(amount);
                            carb_created.send(CarbCreated { colony, amount });
                        }
                    }
                }
            }
        }
    }
}
