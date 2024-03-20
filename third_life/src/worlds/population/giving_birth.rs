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
use rand_distr::{Distribution, Normal};
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

pub fn citizen_births(
    mut commands: Commands,
    mut date_event_reader: EventReader<DateChanged>,
    mut event_writer: EventWriter<CitizenCreated>,
    mut pregnant_women: Query<(Entity, &mut Female, &mut Pregnancy, &CitizenOf), With<Pregnancy>>,
    mut colonies: Query<(Entity, &mut Population, &SanitationInfrastructure, &WorldConfig), With<WorldColony>>,

) {
    for date_event in date_event_reader.read() {
        for (entity, mut w_female, pregnancy, citizen_of) in &mut pregnant_women.iter_mut() {
            if pregnancy.baby_due_date == date_event.date {
                for (colony, mut population, sanitation_infra, config) in colonies.iter_mut() {
                    if citizen_of.colony == colony {
                        // No live birth if live birth mortality rate is too high
                        if roll_chance(
                            ((sanitation_infra.live_birth_mortality_rate / 1000.0) * 100.0) as u8,
                        ) {
                            continue;
                        }
                        let name_rng = RNG::try_from(&Language::Roman).unwrap();
                        let name = name_rng.generate_name();
                        // if higher than 0.5 then the baby is likely lower than average, else higher.
                        let genetic_height = Normal::new(config.population().height_dist().average(), 7.0).unwrap().sample(&mut rand::thread_rng());
                        let genetic_weight = Normal::new(config.population().weight_dist().average(), 10.0).unwrap().sample(&mut rand::thread_rng());

                        // amount of growth per day
                        let daily_growth = (genetic_height - NEW_BORN_HEIGHT)/ 9125.0;
                        let daily_fattening = (genetic_weight - NEW_BORN_WEIGHT) / 9125.0;

                        match roll_chance(50) {
                            true => commands.spawn((
                                MaleCitizenBundle::new(
                                    name,
                                    colony,
                                    date_event.date,
                                    genetic_height,
                                    genetic_weight,
                                    daily_growth,
                                    daily_fattening
                                ),
                                Youngling,
                            )),
                            false => commands.spawn((
                                FemaleCitizenBundle::new(
                                    name,
                                    colony,
                                    date_event.date,
                                    genetic_height,
                                    genetic_weight,
                                    daily_growth,
                                    daily_fattening
                                ),
                                Youngling,
                            )),
                        };
                        w_female.children_had += 1;
                        w_female.last_child_birth_date = Some(date_event.date);
                        population.yearly_infant_births += 1;
                        event_writer.send(CitizenCreated { age: 0, colony });
                    }
                }
                commands.get_entity(entity).map(|mut e| {
                    e.remove::<Pregnancy>();
                    e.try_insert(Employable);
                });
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
                    e.try_insert(Employable);
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
                game_date.date + chrono::Duration::days(thread_rng().gen_range(5..=20) as i64);

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
                + chrono::Duration::days(thread_rng().gen_range(5..=6))
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
                    + chrono::Duration::days(thread_rng().gen_range(547..=650))
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
                                .checked_add_signed(chrono::Duration::days(pregnancy_term))
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
        * (1.0 - urbanization))
        .abs();

    // adjust the pregnancy chance by reducing it by 10% for each child
    preg_chance = preg_chance * (1.0 - 0.1f32).powi(number_of_children as i32) * 100.0;

    roll_chance(preg_chance as u8)
}
