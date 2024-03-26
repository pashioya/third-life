use bevy::{prelude::*, utils::hashbrown::HashMap};
use chrono::{Datelike, NaiveDate};
use rand_distr::num_traits::Float;

use crate::{
    time::{DateChanged, YearChanged},
    worlds::{
        config::WorldConfig,
        env_and_infra::components::{weighted_range, CivilInfrastructure},
        population::components::{
            CarbConsumed, CitizenOf, Employed, Pregnancy, Retiree, Youngling,
        },
        WorldColony,
    },
};

use super::{
    tracking::CarbProduced, CarbCreated, CarbResource, ResourceOf, WheatFarm, WheatFarmCreated,
    WheatFarmNeedsWorker, WheatFarmOf, WheatFarmer,
};

pub fn season_check_wheat(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut wheat_farms: Query<&mut WheatFarm>,
) {
    for DateChanged { date } in day_changed_event_reader.read() {
        if date.month() == 6 && date.day() == 1 {
            for mut wheat_farm in wheat_farms.iter_mut() {
                wheat_farm.harvested = 0.0;
            }
        }
    }
}

pub fn season_end_check_wheat(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut wheat_farms: Query<&mut WheatFarm>,
) {
    for DateChanged { date } in day_changed_event_reader.read() {
        if date.month() == 10 && date.day() == 27 {
            for mut wheat_farm in wheat_farms.iter_mut() {
                wheat_farm.harvested = wheat_farm.size;
            }
        }
    }
}

pub fn check_wheat_farms_counts(
    mut commands: Commands,
    //wheat_farms: Query<(Entity, &WheatFarmOf), With<WheatFarm>>,
    //farm_workers: Query<(Entity, &WheatFarmer)>,
    mut colonies: Query<(
        Entity,
        &mut WorldColony,
        &mut CarbConsumed,
        &mut CarbProduced,
        &WorldConfig,
    )>,
    carb_resources: Query<(&ResourceOf, &CarbResource)>,
    mut year_changed_reader: EventReader<YearChanged>,
    mut created_events: EventWriter<WheatFarmCreated>,
) {
    let resource_map = carb_resources
        .iter()
        .map(|(e, r)| (e.colony, r))
        .collect::<HashMap<_, _>>();

    /*let mut farms_map = wheat_farms
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
        );*/

    for _ in year_changed_reader.read() {
        for (colony, mut world_colony, mut carb_consumed, mut carb_produced, world_config) in
            colonies.iter_mut()
        {
            let min_surplus = world_config.food().min_surplus_multiplier();
            if carb_consumed.amount * min_surplus > resource_map.get(&colony).unwrap().get_kgs() {
                let mut new_farm_count =
                    (carb_consumed.amount / carb_produced.amount).floor() as usize;
                if new_farm_count == 0 {
                    new_farm_count += 1;
                }
                if resource_map.get(&colony).unwrap().get_kgs() == 0.0 {
                    new_farm_count += 2;
                }
                warn!("Need new Wheat Farms");
                warn!(
                    "Consumption|production {:?}|{:?}",
                    carb_consumed.amount, carb_produced.amount
                );
                warn!("New Farm Count: {:?}", new_farm_count);
                let wheat_farm_size = world_config.food().wheat_farm_size();
                while world_colony.space_left() > wheat_farm_size && new_farm_count > 0 {
                    world_colony.take_up_farm_space(wheat_farm_size);
                    created_events.send(WheatFarmCreated { colony });
                    commands.spawn((
                        WheatFarm {
                            size: wheat_farm_size,
                            harvested: wheat_farm_size,
                        },
                        WheatFarmOf { colony },
                    ));
                    new_farm_count -= 1;
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
    mut commands: Commands,
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<WheatFarmNeedsWorker>,
    wheat_farms: Query<(Entity, &WheatFarmOf, &WheatFarm)>,
    farmers: Query<(Entity, &WheatFarmer, &CitizenOf)>,
    colonies: Query<(Entity, &CivilInfrastructure, &WorldConfig)>,
) {
    let mut farmers_map = farmers.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, Vec<Entity>>, (citizen, wheat_farmer, _)| {
            acc.entry(wheat_farmer.farm)
                .or_insert(Vec::new())
                .push(citizen);
            acc
        },
    );
    for DateChanged { date } in day_changed_event_reader.read() {
        let mut farms_map = wheat_farms.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, wheat_farm_of, _)| {
                acc.entry(wheat_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(0);
                acc
            },
        );

        for (_, wheat_farmer, colony_of) in farmers.iter() {
            farms_map
                .get_mut(&colony_of.colony)
                .unwrap()
                .entry(wheat_farmer.farm)
                .and_modify(|count| *count += 1);
        }



        let days_remaining = (NaiveDate::from_ymd_opt(date.year_ce().1 as i32, 10, 27).unwrap() - *date).num_days() as f32;
        //warn!("Days Remaining: {:?}", days_remaining);
        for (colony, farms) in farms_map {
            let farm_mech_value = colonies.get(colony).unwrap().1.farming_mechanization;
            let work_day_length = colonies.get(colony).unwrap().2.work_day_length();
            for (farm, farmer_count) in farms {
                let (_, _, wheat_farm) = wheat_farms.get(farm).unwrap();

                let mut total_workers_needed = 2;
                if wheat_farm.remaining_for_harvest() > 0. {
                    let work_hours_per_hec = weighted_range(1098.0, 8.0, farm_mech_value);
                    let hecs_remaining = wheat_farm.remaining_for_harvest();
                    let total_hours_needed = hecs_remaining * work_hours_per_hec;
                    total_workers_needed = (total_hours_needed / (work_day_length * days_remaining)).ceil() as usize;
                }
                //warn!("Total Workers needed: {:?}", total_workers_needed);

                if farmer_count < total_workers_needed {
                    for _ in 0..(total_workers_needed - farmer_count) {
                        event_writer.send(WheatFarmNeedsWorker { colony, farm });
                    }
                } else if farmer_count > total_workers_needed {
                    for _ in 0..farmer_count - total_workers_needed {
                        let farmer = farmers_map.get_mut(&farm).unwrap().pop().unwrap();
                        commands.get_entity(farmer).map(|mut f| {
                            f.remove::<WheatFarmer>();
                            f.remove::<Employed>();
                        });
                    }
                }

            }
        }
    }
}

pub fn get_farm_workers(
    mut commands: Commands,
    mut event_reader: EventReader<WheatFarmNeedsWorker>,
    free_citizens: Query<
        (Entity, &CitizenOf),
        (
            Without<Employed>,
            Without<Pregnancy>,
            Without<Youngling>,
            Without<Retiree>,
        ),
    >,
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
    colonies: Query<(Entity, &CivilInfrastructure, &WorldConfig)>,
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
            let farm_mech_value = colonies.get(colony).unwrap().1.farming_mechanization;
            let work_day_length = colonies.get(colony).unwrap().2.work_day_length();
            for (farm_entity, farmer_count) in farms {
                let available_work_hours = farmer_count as f32 * work_day_length;
                let work_hours_per_hec = weighted_range(1098.0, 8.0, farm_mech_value);
                let mut hec_worked = available_work_hours / work_hours_per_hec;

                let (_, mut wheat_farm, _) = wheat_farms.get_mut(farm_entity).unwrap();
                if hec_worked > wheat_farm.remaining_for_harvest() {
                    hec_worked = wheat_farm.remaining_for_harvest();
                }

                wheat_farm.harvested += hec_worked;

                let yield_per_hec = weighted_range(1500., 2670., farm_mech_value);

                let actual_yield = hec_worked * yield_per_hec;

                for (mut carb_resource, resource_of) in carb_resources.iter_mut() {
                    if resource_of.colony == colony {
                        carb_resource.add_kgs(actual_yield);
                        carb_created.send(CarbCreated {
                            colony,
                            amount: actual_yield,
                        });
                    }
                }
            }
        }
    }
}
