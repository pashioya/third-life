use bevy::prelude::*;

use crate::{worlds::wealth::components::Treasury, SimulationState};

use super::components::EcosystemVitality;

pub struct EcosystemVitalityPlugin;

impl Plugin for EcosystemVitalityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_ecosystem_info).run_if(in_state(SimulationState::Running)),
        );
    }
}

fn update_ecosystem_info(mut colonies: Query<(&Treasury, &mut EcosystemVitality)>) {
    for (policy, mut infra) in colonies.iter_mut() {
        infra.update(policy.total_sanitation_spending());
    }
}
