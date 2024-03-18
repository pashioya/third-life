use bevy::prelude::*;

use crate::{
    worlds::wealth::components::{SpendingPolicy, Treasury},
    SimulationState,
};

use super::components::SanitationInfrastructure;

pub struct SanitationInfrastructurePlugin;

impl Plugin for SanitationInfrastructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_sanitation_info).run_if(in_state(SimulationState::Running)),
        );
    }
}

fn update_sanitation_info(mut colonies: Query<(&Treasury, &mut SanitationInfrastructure)>) {
    for (policy, mut infra) in colonies.iter_mut() {
        infra.update(policy.total_sanitation_spending());
    }
}
