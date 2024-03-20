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
    mut infra: Query<(
        Entity,
        &mut SanitationInfrastructure,
        &Population,
    )>,
    citizens: Query<(&Citizen, &CitizenOf)>,
    children: Query<(&Citizen, &CitizenOf), With<StillGrowing>>,
    starving_pop: Query<(&Citizen, &CitizenOf), With<Starving>>,
    mut date_change_event: EventReader<DateChanged>,
) {
    for date_event in date_change_event.read() {
        for (ent, mut infra, population) in infra.iter_mut() {
            let total_citizens = citizens
                .iter()
                .filter(|(_, citizen_of)| citizen_of.colony == ent)
                .count();
            let undernourishment = if total_citizens == 0 {
                0.0
            } else {
                starving_pop
                    .iter()
                    .filter(|(_, citizen_of)| citizen_of.colony == ent)
                    .count() as f32
                    / total_citizens as f32
            };
            let young_children = children
                .iter()
                .filter(|c| {
                    c.1.colony == ent && date_event.date.years_since(c.0.birthday).unwrap() < 5
                })
                .collect::<Vec<_>>();

            let all_child_weights = young_children
                .iter()
                .map(|c| c.0.weight)
                .collect::<Vec<f32>>();
            let all_child_heights = young_children
                .iter()
                .map(|c| c.0.height)
                .collect::<Vec<f32>>();


            let child_stunting = if all_child_heights.is_empty() {
                0.0
            } else {
                young_children
                    .iter()
                    .filter(|c| c.0.height < (c.0.daily_growth * (date_event.date - c.0.birthday).num_days() as f32))
                    .count() as f32
                    / all_child_heights.len() as f32
            };

            let child_wasting = if all_child_heights.is_empty() {
                0.0
            } else {
                young_children
                    .iter()
                    .filter(|c| c.0.weight < (c.0.daily_fattening * (date_event.date - c.0.birthday).num_days() as f32))
                    .count() as f32
                    / all_child_weights.len() as f32
            };
            infra.actual_infant_mortality_rate_fn(
                population.yearly_infant_deaths,
                population.yearly_infant_births,
            );
            infra.calc_global_hunger_index_fn(undernourishment, child_stunting, child_wasting);
        }
    }
}
