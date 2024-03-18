mod civil_infrastructure;
pub mod components;
mod ecosystem_vitality;
mod enviromental_health;
mod sanitation_infra;

use bevy::prelude::*;

use self::{
    civil_infrastructure::CivilInfrastructurePlugin, ecosystem_vitality::EcosystemVitalityPlugin,
    enviromental_health::EnvironmentalHealthPlugin,
    sanitation_infra::SanitationInfrastructurePlugin,
};

pub struct InfrastructurePlugin;

impl Plugin for InfrastructurePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SanitationInfrastructurePlugin,
            EcosystemVitalityPlugin,
            EnvironmentalHealthPlugin,
            CivilInfrastructurePlugin,
        ));
    }
}
