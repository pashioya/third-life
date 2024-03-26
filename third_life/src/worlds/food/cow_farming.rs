use std::usize;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use chrono::{Datelike, Months, NaiveDate};
use rand_distr::num_traits::Float;

use crate::{
    common::utils::roll_chance,
    time::{DateChanged, GameDate, YearChanged},
    worlds::{
        config::WorldConfig,
        env_and_infra::components::{weighted_range, CivilInfrastructure},
        population::components::{
            CitizenOf, Employed, MeatConsumed, Pregnancy, Retiree, Youngling,
        },
        WorldColony,
    },
};

use super::{
    tracking::MeatProduced, Cow, CowFarm, CowFarmCreated, CowFarmNeedsWorker, CowFarmOf, CowFarmer,
    CowOf, IsBreeder, IsBull, MeatCreated, MeatResource, ResourceOf,
};

pub fn mark_breeders(
    mut commands: Commands,
    cow_farms: Query<Entity, With<CowFarm>>,
    breeding_bulls: Query<(Entity, &CowOf), With<IsBreeder>>,
    bulls: Query<(Entity, &CowOf), (With<IsBull>, Without<IsBreeder>)>,
) {
    let mut farms_map = cow_farms.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, HashMap<&str, Vec<Entity>>>, farm_entity| {
            acc.entry(farm_entity)
                .or_insert(HashMap::new())
                .entry("breeders")
                .or_insert(Vec::new());

            acc.entry(farm_entity)
                .or_insert(HashMap::new())
                .entry("bulls")
                .or_insert(Vec::new());
            acc
        },
    );

    for (cow_entity, cow_of) in breeding_bulls.iter() {
        farms_map
            .get_mut(&cow_of.cow_farm)
            .unwrap()
            .get_mut("breeders")
            .unwrap()
            .push(cow_entity);
    }
    for (cow_entity, cow_of) in bulls.iter() {
        farms_map
            .get_mut(&cow_of.cow_farm)
            .unwrap()
            .get_mut("bulls")
            .unwrap()
            .push(cow_entity);
    }

    for (_, animals) in farms_map {
        if animals.get("breeders").unwrap().len() < 2 {
            if animals.get("bulls").unwrap().len() > 0 {
                commands
                    .get_entity(animals.get("bulls").unwrap()[0])
                    .map(|mut b| {
                        b.try_insert(IsBreeder);
                    });
            }
        }
    }
}

pub fn breed_cows(
    mut commands: Commands,
    mut day_changed_event_reader: EventReader<DateChanged>,
    cows: Query<(&Cow, &CowOf), Without<IsBull>>,
) {
    for day in day_changed_event_reader.read() {
        if day.date.month() == 6 && day.date.day() == 1 {
            let mut cows_to_spawn: Vec<_> = Vec::new();
            let mut bulls_to_spawn: Vec<_> = Vec::new();
            for (_, cow_of) in cows.iter() {
                match roll_chance(50) {
                    true => cows_to_spawn.push((
                        Cow { birthday: day.date },
                        CowOf {
                            cow_farm: cow_of.cow_farm,
                        },
                    )),
                    false => bulls_to_spawn.push((
                        Cow { birthday: day.date },
                        CowOf {
                            cow_farm: cow_of.cow_farm,
                        },
                        IsBull,
                    )),
                }
            }
            commands.spawn_batch(cows_to_spawn);
            commands.spawn_batch(bulls_to_spawn);
        }
    }
}

pub fn check_cow_farm_workers(
    mut commands: Commands,
    colonies_infrastructure: Query<(Entity, &CivilInfrastructure)>,
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<CowFarmNeedsWorker>,
    mut cow_farms: Query<(Entity, &mut CowFarm, &CowFarmOf), With<CowFarm>>,
    cows: Query<&CowOf>,
    farmers: Query<(Entity, &CowFarmer, &CitizenOf)>,
) {
    let cow_meat_weight = 250.;
    let cows_count_map =
        cows.iter()
            .fold(HashMap::new(), |mut acc: HashMap<Entity, usize>, cow_of| {
                *acc.entry(cow_of.cow_farm).or_insert(0) += 1;
                acc
            });
    let mut farmers_map = farmers.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, Vec<Entity>>, (citizen, cow_farmer, _)| {
            acc.entry(cow_farmer.farm)
                .or_insert(Vec::new())
                .push(citizen);
            acc
        },
    );
    for _ in day_changed_event_reader.read() {
        let mut farms_map = cow_farms.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, HashMap<Entity, usize>>, (farm_entity, _, cow_farm_of)| {
                acc.entry(cow_farm_of.colony)
                    .or_insert(HashMap::new())
                    .entry(farm_entity)
                    .or_insert(0);
                acc
            },
        );

        for (_, cow_farmer, colony_of) in farmers.iter() {
            farms_map
                .get_mut(&colony_of.colony)
                .unwrap()
                .entry(cow_farmer.farm)
                .and_modify(|count| *count += 1);
        }

        for (colony, farms) in farms_map {
            let hours_needed_to_butcher_cow = cow_meat_weight
                * weighted_range(
                    1.3,
                    0.055,
                    colonies_infrastructure
                        .get(colony)
                        .unwrap()
                        .1
                        .farming_mechanization,
                );
            for (farm_entity, farmer_count) in farms {
                let (_, mut farm, _) = cow_farms.get_mut(farm_entity).unwrap();
                if farmer_count < 1 {
                    farm.hours_worked = farm.hours_worked / 2.;
                }
                if farm.hours_worked
                    > *cows_count_map.get(&farm_entity).unwrap() as f32
                        * hours_needed_to_butcher_cow
                        * 1.2
                {
                    if farm.farmers_wanted > 1 {
                        farm.farmers_wanted -= 1;
                    }
                } else if farm.hours_worked
                    < *cows_count_map.get(&farm_entity).unwrap() as f32
                        * hours_needed_to_butcher_cow
                        * 0.9
                {
                    if farm.farmers_wanted < 15 {
                        farm.farmers_wanted += 1;
                    }
                }
                if farmer_count < farm.farmers_wanted {
                    for _ in 0..(farm.farmers_wanted - farmer_count) {
                        event_writer.send(CowFarmNeedsWorker {
                            colony,
                            farm: farm_entity,
                        });
                    }
                } else if farmer_count > farm.farmers_wanted {
                    for _ in 0..farmer_count - farm.farmers_wanted {
                        let farmer = farmers_map.get_mut(&farm_entity).unwrap().pop().unwrap();
                        commands.get_entity(farmer).map(|mut f| {
                            f.remove::<CowFarmer>();
                            f.remove::<Employed>();
                        });
                    }
                }
            }
        }
    }
}

pub fn get_cow_farm_workers(
    mut commands: Commands,
    mut event_reader: EventReader<CowFarmNeedsWorker>,
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
                        CowFarmer {
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

pub fn work_cow_farm(
    mut day_changed_event_reader: EventReader<DateChanged>,
    colonies_config: Query<(Entity, &WorldConfig)>,
    mut cow_farms: Query<(Entity, &mut CowFarm, &CowFarmOf)>,
    farmers: Query<(&CowFarmer, &CitizenOf)>,
) {
    for _ in day_changed_event_reader.read() {
        let farmers_map = farmers.iter().fold(
            HashMap::new(),
            |mut acc: HashMap<Entity, usize>, (cow_farmer, _)| {
                *acc.entry(cow_farmer.farm).or_insert(0) += 1;
                acc
            },
        );

        for (farm_entity, farmer_count) in farmers_map {
            let (_, mut cow_farm, CowFarmOf { colony }) = cow_farms.get_mut(farm_entity).unwrap();
            let available_work_hours =
                farmer_count as f32 * colonies_config.get(*colony).unwrap().1.work_day_length();
            cow_farm.hours_worked += available_work_hours;
        }
    }
}

pub fn butcher_cows(
    mut commands: Commands,
    colonies_infrastructure: Query<(Entity, &CivilInfrastructure)>,
    mut meat_resources: Query<(&mut MeatResource, &ResourceOf)>,
    mut cow_farms: Query<(Entity, &mut CowFarm, &CowFarmOf)>,
    bulls: Query<(Entity, &Cow, &CowOf), (With<IsBull>, Without<IsBreeder>)>,
    cows: Query<(Entity, &Cow, &CowOf), (Without<IsBull>, Without<IsBreeder>)>,
    mut meat_created: EventWriter<MeatCreated>,
    mut day_changed_event_reader: EventReader<DateChanged>,
) {
    let mut farms_map = cow_farms.iter_mut().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, HashMap<Entity, Vec<(Entity, &Cow)>>>,
         (farm_entity, _, cow_farm_of)| {
            acc.entry(cow_farm_of.colony)
                .or_insert(HashMap::new())
                .entry(farm_entity)
                .or_insert(Vec::new());
            acc
        },
    );

    for (_, farms) in farms_map.iter_mut() {
        for (cow_entity, cow, cow_of) in cows.iter() {
            farms
                .entry(cow_of.cow_farm)
                .and_modify(|f| f.push((cow_entity, cow)));
        }
    }
    for (_, farms) in farms_map.iter_mut() {
        for (bull_entity, cow, cow_of) in bulls.iter() {
            farms
                .entry(cow_of.cow_farm)
                .and_modify(|f| f.push((bull_entity, cow)));
        }
    }

    let cow_meat_weight = 250.;

    for day in day_changed_event_reader.read() {
        for (colony, farms) in farms_map.iter_mut() {
            let farm_mech_value = colonies_infrastructure
                .get(*colony)
                .unwrap()
                .1
                .farming_mechanization;
            let mut harvested_meat = 0.0;
            for (farm_entity, cows) in farms.iter_mut() {
                let (_, mut farm, _) = cow_farms.get_mut(*farm_entity).unwrap();
                let hours_needed_to_butcher_cow =
                    cow_meat_weight * weighted_range(1.3, 0.055, farm_mech_value);

                if farm.hours_worked > hours_needed_to_butcher_cow {
                    let mut too_young = 0;
                    let mut harvest_count =
                        (farm.hours_worked / hours_needed_to_butcher_cow).floor() as usize;
                    while let Some((cow_entity, cow)) = cows.pop() {
                        if harvest_count <= 0 || cows.len() + too_young <= 47 {
                            break;
                        }
                        if get_age_in_months(day.date, cow.birthday) > 18 {
                            commands.get_entity(cow_entity).map(|mut e| e.despawn());
                            harvest_count -= 1;
                            farm.hours_worked -= hours_needed_to_butcher_cow;
                            harvested_meat += cow_meat_weight;
                        } else {
                            too_young += 1;
                        }
                    }
                }
            }

            for (mut meat_resource, resource_of) in meat_resources.iter_mut() {
                if resource_of.colony == *colony {
                    meat_resource.add_kgs(harvested_meat);
                    meat_created.send(MeatCreated {
                        colony: *colony,
                        amount: harvested_meat,
                    });
                }
            }
        }
    }
}

pub fn check_for_more_cow_farms(
    mut commands: Commands,
    mut colonies: Query<(
        Entity,
        &mut WorldColony,
        &mut MeatConsumed,
        &mut MeatProduced,
        &WorldConfig,
    )>,
    mut year_changed_rader: EventReader<YearChanged>,
    meat_resources: Query<(&ResourceOf, &MeatResource)>,
    game_date: Res<GameDate>,
    mut created_events: EventWriter<CowFarmCreated>,
) {
    let resource_map = meat_resources
        .iter()
        .map(|(e, r)| (e.colony, r))
        .collect::<HashMap<_, _>>();

    for _ in year_changed_rader.read() {
        for (colony, mut world_colony, mut meat_consumed, mut meat_produced, world_config) in
            colonies.iter_mut()
        {
            let min_surplus = world_config.food().min_surplus_multiplier();

            if meat_consumed.amount * min_surplus > resource_map.get(&colony).unwrap().get_kgs() {
                let mut new_farm_count =
                    (meat_consumed.amount / meat_produced.amount).floor() as usize;
                if new_farm_count == 0 {
                    new_farm_count += 1;
                }
                if resource_map.get(&colony).unwrap().get_kgs() == 0.0 {
                    new_farm_count += 2;
                }
                warn!("Need new Cow Farms");
                warn!(
                    "Consumption|production {:?}|{:?}",
                    meat_consumed.amount, meat_produced.amount
                );
                warn!("New Farm Count: {:?}", new_farm_count);
                let cow_farm_size = world_config.food().cow_farm_size();
                while world_colony.space_left() > cow_farm_size && new_farm_count > 0 {
                    world_colony.take_up_farm_space(cow_farm_size);
                    created_events.send(CowFarmCreated { colony });
                    let cow_farm_entity = commands
                        .spawn((
                            CowFarm {
                                size: cow_farm_size,
                                farmers_wanted: 4,
                                hours_worked: 0.,
                            },
                            CowFarmOf { colony },
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
                    new_farm_count -= 1;
                }
            }
            meat_produced.amount = 0.0;
            meat_consumed.amount = 0.0;
        }
    }
}
fn get_age_in_months(game_date: NaiveDate, birthday: NaiveDate) -> i32 {
    let years = game_date.year() as i32 - birthday.year() as i32;
    let mut months = game_date.month() as i32 - birthday.month() as i32;
    months += years * 12;

    months
}
