use bevy::prelude::*;

use crate::{
    worlds::{
        config::WorldConfig, population::components::Population, wealth::components::Treasury,
        WorldColony,
    },
    SimulationState,
};

use super::components::EnvironmentalHealth;

pub struct EnvironmentalHealthPlugin;

impl Plugin for EnvironmentalHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_env_health_info).run_if(in_state(SimulationState::Running)),
        );
    }
}

fn update_env_health_info(
    mut colonies: Query<(
        &Treasury,
        &mut EnvironmentalHealth,
        &WorldConfig,
        &Population,
        &WorldColony,
    )>,
) {
    for (policy, mut infra, config, pop, colony) in colonies.iter_mut() {
        infra.update(
            policy.total_sanitation_spending(),
            config.population().space_per_person(),
            colony.space_left(),
            colony.human_space(),
            pop.count,
        );
    }
}
