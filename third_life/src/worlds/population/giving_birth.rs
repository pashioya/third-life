use crate::{
    common::utils::roll_chance,
    time::{DateChanged, GameDate, MonthChanged},
    worlds::{
        config::WorldConfig, env_and_infra::components::{CivilInfrastructure, SanitationInfrastructure}, wealth::components::Treasury, WorldColony
    },
    SimulationState,
};
use bevy::prelude::*;
use bevy_egui::egui::ahash::{HashMap, HashMapExt};
use rand::{thread_rng, Rng};
use rnglib::{Language, RNG};

use super::{components::*, events::*};

pub struct GivingBirthPlugin;

impl Plugin for GivingBirthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init_ovulation,
                end_ovulation,
                init_miscarriage,
                init_pregnancies,
                citizen_births,
            )
                .run_if(in_state(SimulationState::Running)),
        )
        .add_event::<CitizenCreated>();
    }
}

fn get_randome_name() -> String {
    let Ok(name_rng) = RNG::new(&Language::Elven) else {
        return get_randome_name();
    };
    name_rng.generate_name()
}

pub fn citizen_births(
    mut commands: Commands,
    mut date_event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<CitizenCreated>,
    mut pregnant_women: Query<(Entity, &Citizen, &mut Female, &mut Pregnancy, &CitizenOf), With<Pregnancy>>,
    mut colonies: Query<(&mut Population, &WorldConfig), With<WorldColony>>,

) {
    if let Some(DateChanged { date }) = date_event_reader.read().last() {
        for (entity, Citizen { birthday, .. }, mut w_female, pregnancy, CitizenOf { colony }) in &mut pregnant_women.iter_mut() {
            if pregnancy.baby_due_date <= *date {
                if let Ok((mut population, config)) = colonies.get_mut(*colony) {
                    let mother_age = date.years_since(*birthday).unwrap() as usize;
                    
                    create_citizen(*date, *colony, *date, config.population())
                        .spawn(&mut commands);

                    w_female.children_had += 1;
                    w_female.last_child_birth_date = Some(*date);
                    population.yearly_infant_births += 1;
                    event_writer.send(CitizenCreated { 
                        age: 0, colony: *colony, mother_age: Some(mother_age)
                    });
                    commands.get_entity(entity).map(|mut e| {
                        e.remove::<Pregnancy>();
                    });
                };
            }
        }
    }
}

pub fn init_miscarriage(
    mut commands: Commands,
    mut event_reader: EventReader<MonthChanged>,
    mut pregnant_women: Query<(Entity, &Citizen), With<Pregnancy>>,
    game_date: Res<GameDate>,
) {
    for _ in event_reader.read() {
        for (entity, w_citizen) in &mut pregnant_women {
            if miscarriage_chance(game_date.date.years_since(w_citizen.birthday).unwrap() as u8) {
                commands.get_entity(entity).map(|mut e| {
                    e.remove::<Pregnancy>();
                });
            }
        }
    }
}

pub fn miscarriage_chance(age: u8) -> bool {
    match age {
        18..=19 => roll_chance(17),
        20..=24 => roll_chance(11),
        25..=29 => roll_chance(10),
        30..=34 => roll_chance(11),
        35..=39 => roll_chance(17),
        40..=44 => roll_chance(33),
        45.. => roll_chance(57),
        _ => false,
    }
}

pub fn init_ovulation(
    mut commands: Commands,
    mut event_reader: EventReader<MonthChanged>,
    game_date: Res<GameDate>,
    women: Query<(Entity, &mut Citizen), (With<Female>, Without<Pregnancy>, Without<Ovulation>)>,
) {
    for _ in event_reader.read() {
        for (entity, _) in &women {
            let ovulation_start_date =
                game_date.date + chrono::Duration::try_days(thread_rng().gen_range(5..=20) as i64).unwrap();

            commands.get_entity(entity).map(|mut e| {
                e.try_insert(Ovulation {
                    ovulation_start_date,
                });
            });
        }
    }
}

pub fn end_ovulation(
    mut commands: Commands,
    mut date_event_reader: EventReader<DateChanged>,
    women: Query<(Entity, &Citizen, &Ovulation)>,
) {
    for date_event in date_event_reader.read() {
        for (entity, _, ovulation) in &women {
            if ovulation.ovulation_start_date
                + chrono::Duration::try_days(thread_rng().gen_range(5..=6)).unwrap()
                == date_event.date
            {
                commands.get_entity(entity).map(|mut e| {
                    e.remove::<Ovulation>();
                });
            }
        }
    }
}

pub fn init_pregnancies(
    mut commands: Commands,
    mut event_reader: EventReader<DateChanged>,
    infra: Query<(
        Entity,
        &Treasury,
        &CivilInfrastructure,
        &SanitationInfrastructure,
    )>,
    citizens: Query<
        (Entity, &Citizen, &CitizenOf, &Female),
        (With<Ovulation>, With<Spouse>, Without<Pregnancy>),
    >,
) {
    for date_event in event_reader.read() {
        let infra_map = infra.iter().fold(
            HashMap::new(),
            |mut acc, (colony_entity, treasury, w_civil_infra, w_san_infra)| {
                acc.insert(colony_entity, (treasury, w_civil_infra, w_san_infra));
                acc
            },
        );

        for (w_entity, w_citizen, citizen_of, female) in &citizens {
            let (w_treasury, w_civil_infra, w_san_infra) =
                infra_map.get(&citizen_of.colony).unwrap();

            if !female.last_child_birth_date.is_none()
                && (female.last_child_birth_date).unwrap()
                    + chrono::Duration::try_days(thread_rng().gen_range(547..=650)).unwrap()
                    > date_event.date
            {
                continue;
            }

            if pregnancy_desire(
                w_treasury.total_wealth,
                w_treasury.old_wealth,
                w_civil_infra.urbanization_index,
                w_san_infra.global_hunger_index,
                female.children_had,
            ) {
                if pregnancy_chance(date_event.date.years_since(w_citizen.birthday).unwrap() as u8)
                {
                    let pregnancy_term = thread_rng().gen_range(270..=280);
                    commands.get_entity(w_entity).map(|mut e| {
                        e.try_insert(Pregnancy {
                            baby_due_date: date_event
                                .date
                                .checked_add_signed(chrono::Duration::try_days(pregnancy_term).unwrap())
                                .unwrap(),
                        });
                    });
                }
            }
        }
    }
}

pub fn pregnancy_chance(age: u8) -> bool {
    let age_f32 = age as f32;
    let pregnancy_chance = -0.0005893368566 * age_f32.powf(4.0)
        + 0.0730945581099 * age_f32.powf(3.0)
        - 3.3813849411076 * age_f32.powf(2.0)
        + 66.904528373158 * age_f32
        - 390.6749280259455;
    roll_chance(pregnancy_chance as u8)
}

pub fn pregnancy_desire(
    economy_t: f32,
    economy_tm1: f32,
    urbanization: f32,
    global_hunger_index: f32,
    number_of_children: usize,
) -> bool {
    let mut preg_chance = (2.1
        * urbanization
        * (economy_t / economy_tm1)
        * (4.5 - global_hunger_index)
        * (2.0 - urbanization))
        .abs();
    
    // adjust the pregnancy chance by reducing it by 10% for each child
    preg_chance = preg_chance * (1.0 - 0.1f32).powi(number_of_children as i32) * 100.0;

    roll_chance(preg_chance as u8)
}
