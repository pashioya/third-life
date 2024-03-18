use bevy::prelude::*;

use crate::{
    worlds::wealth::components::{SpendingPolicy, Treasury},
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

fn update_env_health_info(mut colonies: Query<(&Treasury, &mut EnvironmentalHealth)>) {
    for (policy, mut infra) in colonies.iter_mut() {
        infra.update(policy.total_sanitation_spending());
    }
}
