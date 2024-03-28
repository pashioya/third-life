use super::{components::*, events::*};
use crate::{
    time::{DateChanged, GameDate},
    worlds::{config::WorldConfig, env_and_infra::components::{EcosystemVitality, EnvironmentalHealth, SanitationInfrastructure}},
    SimulationState,
};
use bevy::prelude::*;

use chrono::Days;
use rand::Rng;

use std::{collections::HashMap, iter::zip};

pub struct DeathsPlugin;

impl Plugin for DeathsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (old_age_death, starvation, infant_mortality)
                .run_if(in_state(SimulationState::Running)),
        )
        .add_event::<CitizenDied>();
    }
}

pub fn old_age_death(
    mut date_changed: EventReader<DateChanged>,
    worlds: Query<(Entity, &WorldConfig, &EnvironmentalHealth, &EcosystemVitality)>,
    mut commands: Commands,
    citizens: Query<(Entity, &CitizenOf, &Citizen)>,
    game_date: Res<GameDate>,
    mut death_events: EventWriter<CitizenDied>,
) {
    let epi_map = worlds
        .iter()
        .map(|(e, w, env, eco)| {
            let val = (env.total_avg_val() + eco.total_avg_val()) / 2.;
            let spread = w.population().life_expectancy_spread();
            (e, (val * 100., spread))
        })
        .collect::<HashMap<_, _>>();

    let days_passed = date_changed.read().collect::<Vec<_>>();
    let citizens = citizens.iter().collect::<Vec<_>>();
    let mut rng = rand::thread_rng();
    let probs = (0..citizens.len())
        .into_iter()
        .map(|_| rng.gen())
        .collect::<Vec<f32>>();

    zip(citizens, probs)
        .into_iter()
        .fold(
            HashMap::new(),
            |mut acc: HashMap<_, _>, ((entity, colony, citizen), prob)| {
                acc.entry(colony.colony)
                    .or_insert(vec![])
                    .push((entity, citizen, prob));
                acc
            },
        )
        .into_iter()
        .for_each(|(colony, citizens)| {
            let (life_exp, spread) = epi_map.get(&colony).unwrap();
            citizens.into_iter().for_each(|(e, c, rnd)| {
                // INFO: Small optimization. Age is only calculated once meaning that
                // if the birthday is in the days passed we are still using the old
                // age. Since
                let age = game_date.years_since(c.birthday).unwrap_or(0) as f32;
                for _ in &days_passed {
                    if died(age, *life_exp, *spread, rnd) {
                        death_events.send(CitizenDied::old_age(colony, e));
                        commands.get_entity(e).map(|mut e| {
                            e.despawn();
                            Some(e)
                        });
                        break;
                    }
                }
            })
        });
}

/// returns whether the person has dies or not based on the parameters. Is
/// intended to be queried for each day.
///
/// If the probability of death at 70 is 0.6 than that value us divided by
/// 365. so that it reflects the probability of dying for every single day.
pub fn died(age: f32, life_exp: f32, spread: f32, rnd: f32) -> bool {
    //FIXME: This probability is wrong will kill too many people
    let act = (1. / (1. + ((-age + life_exp) / spread).exp())) / 365.;
    rnd <= act
}

fn starvation(
    mut commands: Commands,
    mut death_events: EventWriter<CitizenDied>,
    starving_citizens: Query<(Entity, &Citizen, &CitizenOf, &Starving)>,
    mut population: Query<&mut Population>,
    game_date: Res<GameDate>,
) {
    for (entity, citizen, CitizenOf { colony }, starving) in starving_citizens.iter() {
        if starving.died() {
            if game_date.date.years_since(citizen.birthday).unwrap_or(0) <= 5 {
                let _ = population.get_mut(*colony).map(|mut p| {
                    p.yearly_infant_deaths += 1;
                });
            }

            commands.get_entity(entity).map(|mut e| {
                e.despawn();
            });
            death_events.send(CitizenDied::starved(*colony, entity));
        }
    }
}

pub fn infant_mortality(
    game_date: Res<GameDate>,
    mut new_days: EventReader<DateChanged>,
    mut commands: Commands,
    mut infra: Query<(Entity, &SanitationInfrastructure, &mut Population)>,
    citizens: Query<(Entity, &CitizenOf, &Citizen)>,
    mut death_event: EventWriter<CitizenDied>,
) {
    let rates = infra
        .into_iter()
        .map(|e| (e.0, e.1.live_birth_mortality_rate))
        .collect::<HashMap<_, _>>();

    let events = new_days.read();

    let first_date = game_date.date - Days::new(events.len() as u64);

    let mut rng = rand::thread_rng();

    citizens.into_iter().for_each(|(entity, of, citizen)| {
        if citizen.birthday > game_date.date && citizen.birthday < first_date {
            return;
        }
        let days_since_birth = (game_date.date - citizen.birthday).num_days() as usize;

        if days_since_birth > (365 - events.len()) && days_since_birth <= 365 {
            let r = rng.gen::<f32>();
            if r < *rates.get(&of.colony).unwrap() {
                commands.get_entity(entity).map(|mut e| {
                    e.despawn();
                    death_event.send(CitizenDied::infant_death(of.colony, entity));
                });
                // ? Should this really be here?  this kills a child so i think it should
                let _ = infra.get_mut(of.colony).map(|(_, _, mut p)| {
                    p.yearly_infant_deaths += 1;
                });
            }
        }
    });
}
