use bevy::prelude::*;

use crate::{
    worlds::{wealth::components::Treasury, WorldColony},
    SimulationState,
};

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

fn update_ecosystem_info(mut colonies: Query<(&Treasury, &mut EcosystemVitality, &WorldColony)>) {
    for (policy, mut infra, colony) in colonies.iter_mut() {
        infra.update(
            policy.total_sanitation_spending(),
            colony.space_left(),
            colony.farm_space(),
            colony.human_space(),
        );
    }
}
