pub mod components;
mod dying;
pub mod events;
mod food_consumption;
mod giving_birth;
mod growing;
mod relationships;

use components::*;
use dying::*;
use events::*;
use giving_birth::*;
use relationships::*;

use crate::{
    common::utils::roll_chance,
    time::{DateChanged, GameDate},
    SimulationState,
};
use bevy::prelude::*;
use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, SkewNormal};
use rnglib::{Language, RNG};

use self::{food_consumption::FoodConsumptionPlugin, growing::GrowingPlugin};

use super::{
    config::WorldConfig,
    food::components::{CowFarmer, WheatFarmer},
    init_colonies, WorldColony,
};

pub struct PopulationPlugin;

impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_citizens).chain().after(init_colonies),
        )
        .add_systems(
            Update,
            (update_population, check_birthdays, come_of_age, retirement)
                .run_if(in_state(SimulationState::Running)),
        )
        .add_plugins((
            GivingBirthPlugin,
            DeathsPlugin,
            RelationshipsPlugin,
            FoodConsumptionPlugin,
            GrowingPlugin,
        ))
        .add_event::<CitizenBirthday>();
    }
}

pub fn init_citizens(
    colonies: Query<(Entity, &WorldConfig), With<WorldColony>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CitizenCreated>,
    game_date: Res<GameDate>,
) {
    for (colony, pop_config) in colonies.iter() {
        let pop_config = pop_config.population();
        let mut rng = thread_rng();
        let name_rng = RNG::try_from(&Language::Roman).unwrap();
        let skew_normal = SkewNormal::new(
            pop_config.population_dist().location(),
            pop_config.population_dist().scale(),
            pop_config.population_dist().shape(),
        )
        .unwrap();
        let mut age_gen = skew_normal.sample_iter(&mut rng);
        let year = game_date.date.year_ce().1 as usize;

        for _ in 0..pop_config.population_size() {
            let age = age_gen.next().unwrap().floor() as usize;
            let birthday = NaiveDate::from_yo_opt(
                (year - age).try_into().unwrap(),
                thread_rng().gen_range(1..=365),
            )
            .unwrap();
            let genetic_height = pop_config.height_dist().average();
            let genetic_weight = pop_config.weight_dist().average();
            let daily_growth =( genetic_height - NEW_BORN_HEIGHT) / 9125.0;
            let daily_fattening = (genetic_weight - NEW_BORN_WEIGHT) / 9125.0;

            let height = (age * 365) as f32 * daily_growth + NEW_BORN_HEIGHT;
            let weight = (age * 365) as f32 * daily_fattening + NEW_BORN_WEIGHT;


            let citizen = Citizen {
                name: name_rng.generate_name(),
                birthday,
                genetic_height,
                genetic_weight,
                height,
                weight,
                daily_growth,
                daily_fattening,
            };
            if game_date.years_since(birthday).unwrap() >= 18 as u32 {
                match roll_chance(50) {
                    true => commands.spawn((citizen, Employable, CitizenOf { colony }, Male)),
                    false => commands.spawn((
                        citizen,
                        Employable,
                        CitizenOf { colony },
                        Female {
                            children_had: 0,
                            last_child_birth_date: None,
                        },
                    )),
                };
            } else {
                match roll_chance(50) {
                    true => commands.spawn((citizen, CitizenOf { colony }, Male)),
                    false => commands.spawn((
                        citizen,
                        CitizenOf { colony },
                        Female {
                            children_had: 0,
                            last_child_birth_date: None,
                        },
                    )),
                };
            }
            event_writer.send(CitizenCreated { age, colony });
        }
        commands.entity(colony).try_insert(Population::default());
    }
}

pub fn update_population(
    mut date_change_event_reader: EventReader<DateChanged>,
    mut populations: Query<(Entity, &mut Population)>,
    citizens: Query<(&Citizen, &CitizenOf)>,
    working_pop: Query<&CitizenOf, (Without<Youngling>, Without<Retiree>, Without<Pregnancy>)>,
    younglings: Query<&CitizenOf, With<Youngling>>,
    retirees: Query<&CitizenOf, With<Retiree>>,
    women: Query<(&Citizen, &CitizenOf, &Female), Without<Youngling>>,
) {
    for date_event in date_change_event_reader.read() {
        for (colony, mut population) in &mut populations.iter_mut() {
            population.count = citizens
                .iter()
                .filter(|(_, citizen_of)| citizen_of.colony == colony)
                .count();

            population.younglings = younglings.iter().filter(|&y| y.colony == colony).count();
            population.retirees = retirees.iter().filter(|&y| y.colony == colony).count();
            population.working_pop = working_pop.iter().filter(|&y| y.colony == colony).count();

            let all_women_children_had: Vec<f32> = women
                .iter()
                .filter_map(|(_, citizen_of, female)| {
                    if citizen_of.colony == colony {
                        Some(female.children_had as f32)
                    } else {
                        None
                    }
                })
                .collect();

            population.average_children_per_mother =
                all_women_children_had.iter().sum::<f32>() / all_women_children_had.len() as f32;

            let all_citizen_ages: Vec<usize> = citizens
                .iter()
                .filter_map(|(citizen, citizen_of)| {
                    if citizen_of.colony == colony {
                        Some(date_event.date.years_since(citizen.birthday).unwrap() as usize)
                    } else {
                        None
                    }
                })
                .collect();

            let all_citizen_weights: Vec<f32> = citizens
                .iter()
                .filter_map(|(citizen, citizen_of)| {
                    if citizen_of.colony == colony {
                        Some(citizen.weight)
                    } else {
                        None
                    }
                })
                .collect();
            let all_citizen_heights: Vec<f32> = citizens
                .iter()
                .filter_map(|(citizen, citizen_of)| {
                    if citizen_of.colony == colony {
                        Some(citizen.height)
                    } else {
                        None
                    }
                })
                .collect();

            if population.count == 0 {
                population.average_age = 0;
                population.average_weight = 0.0;
                population.average_height = 0.0;
            } else {
                population.average_age = all_citizen_ages.iter().sum::<usize>() / population.count;
                population.average_weight =
                    all_citizen_weights.iter().sum::<f32>() / population.count as f32;
                population.average_height =
                    all_citizen_heights.iter().sum::<f32>() / population.count as f32;
            }
            if date_event.date.year() != date_event.date.pred_opt().unwrap().year() {
                population.yearly_infant_births = 0;
                population.yearly_infant_deaths = 0;
            }
        }
    }
}

pub fn check_birthdays(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut birthday_events: EventWriter<CitizenBirthday>,
    citizens: Query<(Entity, &Citizen, &CitizenOf)>,
) {
    for day_changed_event in day_changed_event_reader.read() {
        let game_date = day_changed_event.date;
        for (citizen_entity, citizen, citizen_of) in citizens.iter() {
            if game_date.month() == citizen.birthday.month()
                && game_date.day() == citizen.birthday.day()
            {
                birthday_events.send(CitizenBirthday {
                    entity: citizen_entity,
                    colony: citizen_of.colony,
                    age: game_date.years_since(citizen.birthday).unwrap() as usize,
                });
            }
        }
    }
}

pub fn come_of_age(
    mut commands: Commands,
    mut birthday_event_reader: EventReader<CitizenBirthday>,
    colonies: Query<(Entity, &WorldConfig)>,
) {
    for birthday in birthday_event_reader.read() {
        let config = colonies.get(birthday.colony).unwrap().1;
        if birthday.age == config.population().age_of_adult() {
            commands.get_entity(birthday.entity).map(|mut e| {
                e.remove::<Youngling>();
            });
        }
    }
}

pub fn retirement(
    mut commands: Commands,
    mut birthday_event_reader: EventReader<CitizenBirthday>,
    colonies: Query<(Entity, &WorldConfig)>,
) {
    for birthday in birthday_event_reader.read() {
        let config = colonies.get(birthday.colony).unwrap().1;
        if birthday.age == config.population().age_of_retirement() {
            commands.get_entity(birthday.entity).map(|mut e| {
                e.remove::<WheatFarmer>();
                e.remove::<CowFarmer>();
                e.remove::<Employed>();
                e.try_insert(Retiree);
            });
        }
    }
}
