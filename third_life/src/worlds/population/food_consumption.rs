use crate::{
    time::{DateChanged, GameDate},
    worlds::{
        config::WorldsConfig,
        food::{
            components::{CarbResource, FedStatus, MeatResource, ResourceOf},
            events::{CarbConsumedEvent, MeatConsumedEvent},
        },
        init_colonies, WorldColony, WorldEntity,
    },
    SimulationState,
};
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

use super::{
    CarbConsumed, Citizen, CitizenOf, DietMacroRatios, Male, MeatConsumed, Starving, StarvingStatus,
};

pub struct FoodConsumptionPlugin;
impl Plugin for FoodConsumptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_consumption_tracking, init_diets).after(init_colonies),
        )
        .add_systems(
            Update,
            (consume, update_carb_consumed, update_meat_consumed)
                .run_if(in_state(SimulationState::Running)),
        );
    }
}

fn init_diets(
    mut commands: Commands,
    colonies: Query<(Entity, &WorldEntity)>,
    world_configs: Res<WorldsConfig>,
) {
    let colonies_map = colonies
        .iter()
        .map(|(entity, WorldEntity { name })| (name, entity))
        .collect::<HashMap<_, _>>();
    for world in world_configs.worlds() {
        let entity = colonies_map.get(&world.name()).unwrap();
        commands.get_entity(*entity).map(|mut e| {
            e.try_insert(
                DietMacroRatios::from_config(world.population().diet_macro_ratios()).unwrap(),
            );
        });
    }
}

fn init_consumption_tracking(mut commands: Commands, colonies: Query<Entity, With<WorldColony>>) {
    for colony_entity in colonies.iter() {
        commands.get_entity(colony_entity).map(|mut c| {
            c.try_insert((CarbConsumed::default(), MeatConsumed::default()));
        });
    }
}

fn update_carb_consumed(
    mut carbs_consumed: EventReader<CarbConsumedEvent>,
    mut colonies: Query<(Entity, &mut CarbConsumed), With<WorldColony>>,
) {
    for carb_consumed in carbs_consumed.read() {
        colonies.get_mut(carb_consumed.colony).unwrap().1.amount += carb_consumed.amount;
    }
}

fn update_meat_consumed(
    mut meats_consumed: EventReader<MeatConsumedEvent>,
    mut colonies: Query<(Entity, &mut MeatConsumed), With<WorldColony>>,
) {
    for meat_consumed in meats_consumed.read() {
        colonies.get_mut(meat_consumed.colony).unwrap().1.amount += meat_consumed.amount;
    }
}

/// consumes meat and carb resources to feed citizens.
///
/// Will try to feed the citizens according to their target diet but will also
/// just feed them anything that is available as long as their caloric needs are
/// met.
///
/// - If the citizens are not fed at all then they will either get the Starving
/// component or increase the daycount.
/// - If they do get fed but not to their total caloric needs it doesnt reduce
/// their daycount on the Starving component but it also doesnt increase it.
/// - If they get completly fed then the daycount is reduced or the component
/// completly removed if the daycount goes down to 0.
///
/// I already implemented a optimization where we first check if the colony
/// has enough resoures to feed the whole colony in which case I avoid having
/// to feed each single citizen.
///
/// Single events for each colony are cast out at the end to signify how many
/// carbs and meat have been consumed in total.
///
/// TODO: if citizens are not fed the correct ratio as their diet says then
/// nothing happends, somehow implement that that is a thing
fn consume(
    game_date: Res<GameDate>,
    mut commands: Commands,
    mut day_changed_event_reader: EventReader<DateChanged>,
    worlds: Query<(Entity, &DietMacroRatios)>,
    mut citizens: Query<(
        Entity,
        &Citizen,
        &CitizenOf,
        Option<&mut Starving>,
        Option<&Male>,
    )>,
    mut carb_resources: Query<(&mut CarbResource, &ResourceOf)>,
    mut meat_resources: Query<(&mut MeatResource, &ResourceOf)>,
    mut carb_consumed_eventwriter: EventWriter<CarbConsumedEvent>,
    mut meat_consumed_eventwriter: EventWriter<MeatConsumedEvent>,
) {
    let mut colony_carb_res = carb_resources
        .iter_mut()
        .map(|e| (e.1.colony, e.0))
        .collect::<HashMap<_, _>>();
    let mut colony_meat_res = meat_resources
        .iter_mut()
        .map(|e| (e.1.colony, e.0))
        .collect::<HashMap<_, _>>();

    let colony_diets_map = worlds
        .iter()
        .map(|e| (e.0, ideal_ratio_total(e.1)))
        .collect::<HashMap<_, _>>();

    let mut daily_cals_per_colony = HashMap::new();

    let colony_to_citizens = citizens.iter_mut().fold(
        HashMap::new(),
        |mut acc, (citizen_entity, citizen, of, starving_optional, male)| {
            let age = game_date.years_since(citizen.birthday).unwrap() as f32;
            let daily_calories = match male {
                Some(_) => {
                    66.4730 + 13.7516 * citizen.weight + 5.0033 * citizen.height - 6.7550 * age
                }
                None => 655.0955 + 9.5634 * citizen.weight + 1.8496 * citizen.height - 4.6756 * age,
            };
            *daily_cals_per_colony.entry(of.colony).or_insert(0.) += daily_calories;

            acc.entry(of.colony).or_insert(vec![]).push((
                citizen_entity,
                citizen,
                of,
                starving_optional,
                daily_calories,
            ));
            acc
        },
    );

    let day_events = day_changed_event_reader.read().collect::<Vec<_>>();

    let mut carb_consumed_events = HashMap::new();
    let mut meat_consumed_events = HashMap::new();

    let mut update_events = |colony: Entity, fed_status: &FedStatus| match fed_status {
        FedStatus::FedCorrectDiet { carbs, meat }
        | FedStatus::FedInccorectDiet { carbs, meat }
        | FedStatus::InsuficcentlyFed { carbs, meat } => {
            *carb_consumed_events.entry(colony).or_insert(0.) += carbs;
            *meat_consumed_events.entry(colony).or_insert(0.) += meat;
        }
        FedStatus::NotFed => (),
    };

    let mut completely_fed_colonies = HashMap::new();
    for (colony, meat_ratio) in colony_diets_map.iter() {
        let Some(total_daily_caloric_needs) = daily_cals_per_colony.get(&colony) else {
            continue;
        };
        let total_caloric_needs = total_daily_caloric_needs * day_events.len() as f32;
        let meat_amount = total_caloric_needs * meat_ratio;
        let carb_amount = total_caloric_needs * (1. - meat_ratio);

        let carb_res = colony_carb_res.get_mut(&colony).unwrap();
        let meat_res = colony_meat_res.get_mut(&colony).unwrap();
        if carb_res.has_sufficent_calories(carb_amount)
            && meat_res.has_sufficent_calories(meat_amount)
        {
            let status = feed_citizen(
                colony_carb_res.get_mut(&colony).unwrap(),
                colony_meat_res.get_mut(&colony).unwrap(),
                carb_amount,
                meat_amount,
            );
            update_events(*colony, &status);
            completely_fed_colonies.insert(*colony, status);
        }
    }

    colony_to_citizens
        .into_iter()
        .for_each(|(colony, mut citizens_of_colony)| {
            citizens_of_colony.shuffle(&mut thread_rng());
            match completely_fed_colonies.get(&colony) {
                // ATTENTION: this feed_status is does not contain the calory
                // numbers for a single citizen but for the whole colony
                Some(feed_status) => {
                    let statuses = (0..day_events.len())
                        .into_iter()
                        .map(|_| feed_status)
                        .collect::<Vec<_>>();
                    citizens_of_colony
                        .into_iter()
                        .filter_map(|(entity, _, _, starving_optional, _)| {
                            starving_optional.map(|starving| (entity, starving))
                        })
                        .for_each(|(entity, ref mut starving)| {
                            remove_starving_if_correct(
                                entity,
                                &mut commands,
                                starving.feed_many(&statuses),
                            );
                        });
                }
                None => {
                    for _ in day_events.iter() {
                        citizens_of_colony.iter_mut().for_each(
                            |(citizen_entity, _, _, ref mut starving_optional, daily_cal)| {
                                let meat_ratio = colony_diets_map.get(&colony).unwrap();
                                let carb_res = colony_carb_res.get_mut(&colony).unwrap();
                                let meat_res = colony_meat_res.get_mut(&colony).unwrap();

                                let meat_amount = *daily_cal * *meat_ratio;
                                let carb_amount = *daily_cal * (1. - *meat_ratio);

                                let feeding_status =
                                    feed_citizen(carb_res, meat_res, carb_amount, meat_amount);
                                update_events(colony, &feeding_status);
                                match starving_optional {
                                    Some(ref mut starving) => {
                                        let starving_status = (*starving).feed(&feeding_status);
                                        remove_starving_if_correct(
                                            *citizen_entity,
                                            &mut commands,
                                            starving_status,
                                        );
                                    }
                                    None if feeding_status == FedStatus::NotFed => {
                                        commands.get_entity(*citizen_entity).map(|mut e| {
                                            e.try_insert(Starving::started());
                                        });
                                    }
                                    _ => (),
                                }
                            },
                        );
                    }
                }
            }
        });

    carb_consumed_events
        .into_iter()
        .for_each(|(colony, amount)| {
            carb_consumed_eventwriter.send(CarbConsumedEvent { colony, amount });
        });
    meat_consumed_events
        .into_iter()
        .for_each(|(colony, amount)| {
            meat_consumed_eventwriter.send(MeatConsumedEvent { colony, amount });
        });
}

fn remove_starving_if_correct(
    citizen_entity: Entity,
    commands: &mut Commands,
    starving_status: StarvingStatus,
) {
    match starving_status {
        StarvingStatus::StoppedStarving => {
            commands.get_entity(citizen_entity).map(|mut e| {
                e.remove::<Starving>();
            });
        }
        _ => (),
    }
}

/// Removes the correct amount of resources in the expected reatios if present
/// but adjusts to feed citizen with other resources if there is not enough in
/// the exepcted resources.
///
/// TODO: should have some way of affecting the citizen if the ratio is not
/// achived
fn feed_citizen(
    carb: &mut Mut<CarbResource>,
    meat: &mut Mut<MeatResource>,
    carb_amount: f32,
    meat_amount: f32,
) -> FedStatus {
    match (
        carb.consume_calories(carb_amount),
        meat.consume_calories(meat_amount),
    ) {
        (Ok(consumed_carb_calories), Ok(consumed_meat_calories)) => FedStatus::correct_diet_cals(
            consumed_carb_calories,
            carb.quality,
            consumed_meat_calories,
            meat.quality,
        ),
        (Ok(consumed_carb_calories), Err(_)) => {
            let missing_meat_cals = meat_amount - meat.total_calories();
            match carb.consume_calories(missing_meat_cals) {
                Ok(extra_consumed_carbs_calories) => FedStatus::incorrect_diet_cals(
                    consumed_carb_calories + extra_consumed_carbs_calories,
                    carb.quality,
                    meat.consume_all(),
                    meat.quality,
                ),
                Err(_) => FedStatus::insufficent_cals(
                    consumed_carb_calories,
                    carb.quality,
                    meat.consume_all(),
                    meat.quality,
                ),
            }
        }
        (Err(_), Ok(consumed_meat_calories)) => {
            let missing_carb_cals = carb_amount - carb.total_calories();
            match meat.consume_calories(missing_carb_cals) {
                Ok(extra_consumed_meat_calories) => {
                    carb.consume_all();
                    FedStatus::incorrect_diet_cals(
                        carb.consume_all(),
                        carb.quality,
                        consumed_meat_calories + extra_consumed_meat_calories,
                        meat.quality,
                    )
                }
                Err(_) => FedStatus::insufficent_cals(
                    carb.consume_all(),
                    carb.quality,
                    consumed_meat_calories,
                    meat.quality,
                ),
            }
        }
        (Err(_), Err(_)) => match (carb.total_calories() > 0., meat.total_calories() > 0.) {
            (false, false) => FedStatus::not_fed(),
            (_, _) => FedStatus::insufficent_cals(
                carb.consume_all(),
                carb.quality,
                meat.consume_all(),
                meat.quality,
            ),
        },
    }
}

/// Finds where between the upper and lower bounds the target ratio can be
/// achived.
///
/// As an example if target is 0.2, lower 0.1 and upper 0.3 then the output
/// should be 0.5, meaning that half way between the upper and lower bound
/// the target ratio can be found.
///
/// Upper and lower bounds need to be like so aswell, meaning the upper >= lower
/// which is why the invert variable exists. This varaible allows for inversion
/// before returning allowing for a indirect switching of the lower and upper
/// bounds.
fn ideal_ratio(target_ratio: f32, invert: bool, lower_bound: f32, upper_bound: f32) -> f32 {
    let ret = match (target_ratio < upper_bound, target_ratio > lower_bound) {
        (false, true) => 1.,
        (true, false) => 0.,
        (false, false) => {
            let diff_upp = (upper_bound - target_ratio).abs();
            let diff_low = (lower_bound - target_ratio).abs();
            if diff_low < diff_upp {
                0.
            } else if diff_upp < diff_low {
                1.
            } else {
                0.5
            }
        }
        (true, true) => (target_ratio - lower_bound) / (upper_bound - lower_bound),
    };
    if invert {
        1. - ret
    } else {
        ret
    }
}

/// Given a target ratio returns a number from 0 - 1 that determines how
/// much of meats (0) and how much of carbs (1) is in the diet.
///
/// This means that if the furnction returns 0 then the diet should only be
/// meat if it returns 1 then the diet should only be carbs.
fn ideal_ratio_total(target_ratio: &DietMacroRatios) -> f32 {
    let carbs_ratio = ideal_ratio(
        target_ratio.carbs(),
        false,
        MeatResource::CARBS,
        CarbResource::CARBS,
    );
    let protein_ratio = ideal_ratio(
        target_ratio.protein(),
        true,
        CarbResource::PROTEIN,
        MeatResource::PROTEIN,
    );
    let fats_ratio = ideal_ratio(
        target_ratio.fat(),
        true,
        CarbResource::FATS,
        MeatResource::FATS,
    );
    (carbs_ratio + protein_ratio + fats_ratio) / 3.
}
