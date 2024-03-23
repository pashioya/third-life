use bevy::prelude::*;

use crate::{
    worlds::{
        food::components::{CowFarmer, WheatFarmer},
        population::components::{CitizenOf, Pregnancy, Retiree, Youngling},
        wealth::components::Treasury,
    },
    SimulationState,
};

use super::components::CivilInfrastructure;

pub struct CivilInfrastructurePlugin;

impl Plugin for CivilInfrastructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_civil_infra_info).run_if(in_state(SimulationState::Running)),
        );
    }
}

fn update_civil_infra_info(
    mut colonies: Query<(Entity, &Treasury, &mut CivilInfrastructure)>,
    farmers: Query<&CitizenOf, (With<CowFarmer>, With<WheatFarmer>)>,
    working_pop: Query<
        &CitizenOf,
        (
            Without<Youngling>,
            Without<Retiree>,
            Without<Pregnancy>,
            Without<CowFarmer>,
            Without<WheatFarmer>,
        ),
    >,
) {
    for (entity, treasury, mut infra) in colonies.iter_mut() {
        let free_citizens = working_pop
            .iter()
            .filter(|citizen_of| citizen_of.colony == entity);
        let farmers = farmers
            .iter()
            .filter(|citizen_of| citizen_of.colony == entity);

        infra.update(
            treasury.total_civil_spending(),
            free_citizens.count(),
            farmers.count(),
        )
    }
}
