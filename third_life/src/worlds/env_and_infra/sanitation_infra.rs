use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    time::DateChanged,
    worlds::{
        population::components::{Citizen, CitizenOf, Population, Starving, StillGrowing},
        wealth::components::Treasury,
    },
    SimulationState,
};

use super::components::SanitationInfrastructure;

pub struct SanitationInfrastructurePlugin;

impl Plugin for SanitationInfrastructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_sanitation_info, update_global_hunger_index)
                .run_if(in_state(SimulationState::Running)),
        );
    }
}

fn update_sanitation_info(mut colonies: Query<(&Treasury, &mut SanitationInfrastructure)>) {
    for (policy, mut infra) in colonies.iter_mut() {
        infra.update(policy.total_sanitation_spending());
    }
}

fn update_global_hunger_index(
    mut infra: Query<(Entity, &mut SanitationInfrastructure, &Population)>,
    citizens: Query<(&Citizen, &CitizenOf)>,
    children: Query<(&Citizen, &CitizenOf), With<StillGrowing>>,
    starving_pop: Query<(&Citizen, &CitizenOf), With<Starving>>,
    mut date_change_event: EventReader<DateChanged>,
) {
    let citizens = citizens.into_iter()
        .fold(HashMap::new(), |mut acc, e| {
            acc.entry(e.1.colony).or_insert(vec![]).push(e.0); acc
        });
    let children = children.into_iter()
        .fold(HashMap::new(), |mut acc, e| {
            acc.entry(e.1.colony).or_insert(vec![]).push(e.0); acc
        });
    let starving_pop = starving_pop.into_iter()
        .fold(HashMap::new(), |mut acc, e| {
            acc.entry(e.1.colony).or_insert(vec![]).push(e.0); acc
        });

    if let Some(DateChanged { date }) = date_change_event.read().last() {
        for (ent, mut infra, population) in infra.iter_mut() {
            let total_citizens = citizens.get(&ent).map(Vec::len).unwrap_or(0);
            let undernourishment = if total_citizens != 0 {
                starving_pop.get(&ent).map(Vec::len).unwrap_or(0) as f32
                    / total_citizens as f32
            } else { 0.0
        };
            let young_children = children.get(&ent)
                .map(|vec| {
                    vec
                        .iter()
                        .filter(|Citizen { birthday, ..  }| {
                            date.years_since(*birthday).unwrap_or(0) < 5
                        })
                        .collect::<Vec<_>>()
                }).unwrap_or(vec![]);

            let all_child_weights = young_children
                .iter()
                .map(|Citizen { weight, .. }| *weight)
                .collect::<Vec<f32>>();
            let all_child_heights = young_children
                .iter()
                .map(|Citizen { height, .. }| *height)
                .collect::<Vec<f32>>();

            let stunded_children = young_children
                .iter()
                .filter(|Citizen { birthday, daily_growth, height, .. }| {
                    *height < (
                        *daily_growth * (
                            *date - *birthday
                        ).num_days() as f32
                    )
                })
                .count() as f32;

            let child_stunting = if stunded_children > 0. {
                stunded_children / all_child_heights.len() as f32
            } else { 0. };

            let wasted_children = young_children
                .iter()
                .filter(|Citizen { weight, daily_fattening, birthday, .. }| {
                    *weight < (
                        *daily_fattening * (
                            *date - *birthday
                        ).num_days() as f32
                    )
                })
                .count() as f32;
            let child_wasting = if wasted_children > 0. {
                wasted_children / all_child_weights.len() as f32
            } else { 0. };

            infra.actual_infant_mortality_rate_fn(
                population.yearly_infant_deaths,
                population.yearly_infant_births,
            );
            infra.calc_global_hunger_index_fn(
                undernourishment, child_stunting, child_wasting
            );
        }
    }
}
