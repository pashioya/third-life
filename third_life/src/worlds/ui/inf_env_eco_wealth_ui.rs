use bevy_egui::egui::Ui;

use bevy::prelude::*;

use crate::{
    worlds::{
        env_and_infra::components::{
            CivilInfrastructure, EcosystemVitality, EnvironmentalHealth, SanitationInfrastructure,
        },
        wealth::components::Treasury,
    },
    SimulationState,
};

use super::{EcosystemUi, EnvironmentUi, InfrastructureUi, WealthUi, WorldUiEntity};

pub struct InfEnvEcoWealthPlugin;

impl Plugin for InfEnvEcoWealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_environment_ui_data,
                update_infrastructure_ui_data,
                update_ecosystem_ui_data,
                update_wealth_ui_data,
            )
                .run_if(in_state(SimulationState::Running)),
        );
    }
}

pub fn inf_env_eco_wealth_info(
    ui: &mut Ui,
    inf: &InfrastructureUi,
    env: &EnvironmentUi,
    eco: &EcosystemUi,
    wealth: &WealthUi,
) {
    let size = [300., 200.];
    ui.horizontal_wrapped(|ui| {
        ui.add_sized(size, |ui: &mut Ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("urbanization index: {:.2}", inf.urbanization_index));
                    ui.label(format!(
                        "farming mechanization: {:.2}",
                        inf.farming_mechanization
                    ));
                    ui.label(format!("health index score: {:.2}", inf.health_index_score));
                    ui.label(format!(
                        "live birth mortality rate: {:.4}",
                        inf.live_birth_mortality_rate
                    ));
                    ui.label(format!("global hunger index: {:.2}", inf.global_hunger_index));
                    ui.label(format!(
                        "actual infant death ratio: {:.2}",
                        inf.actual_infant_death_ratio
                    ));
                });
            })
            .response
        });
        ui.add_sized(size, |ui: &mut Ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!(
                        "indoor air pollution: {:.2}",
                        env.indoor_air_pollution
                    ));
                    ui.label(format!("drinking water: {:.2}", env.drinking_water));
                    ui.label(format!("urban particulates: {:.2}", env.urban_particulates));
                });
            })
            .response
        });
        ui.add_sized(size, |ui: &mut Ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("air quality index: {:.2}", eco.air_quality_index));
                    ui.label(format!(
                        "productive natural resources: {:.2}",
                        eco.productive_natural_resources
                    ));
                    ui.label(format!("biodiversity: {:.2}", eco.biodiversity));
                });
            })
            .response
        });
        ui.add_sized(size, |ui: &mut Ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("total wealth: {:.2}", wealth.total_wealth));
                    ui.label(format!("spending available: {:.2}", wealth.spending_available));
                    ui.label(format!("citizen payout: {:.2}", wealth.citizen_payout));
                    ui.label(format!("civil spending: {:.2}", wealth.civil_spending));
                    ui.label(format!(
                        "sanitation spending: {:.2}",
                        wealth.sanitation_spending
                    ));
                    ui.label(format!("social spending: {:.2}", wealth.social_spending));
                    ui.label(format!(
                        "environmental spending: {:.2}",
                        wealth.environmental_spending
                    ));
                });
            })
            .response
        });
    });
}

fn update_infrastructure_ui_data(
    mut uis: Query<(&WorldUiEntity, &mut InfrastructureUi)>,
    colonies: Query<(&CivilInfrastructure, &SanitationInfrastructure)>,
) {
    for (WorldUiEntity(colony), mut ui) in uis.iter_mut() {
        if let Ok((
            CivilInfrastructure {
                urbanization_index,
                farming_mechanization,
            },
            SanitationInfrastructure {
                health_index_score,
                live_birth_mortality_rate,
                global_hunger_index,
                actual_infant_death_ratio,
            },
        )) = colonies.get(*colony)
        {
            ui.urbanization_index = *urbanization_index;
            ui.farming_mechanization = *farming_mechanization;
            ui.health_index_score = *health_index_score;
            ui.live_birth_mortality_rate = *live_birth_mortality_rate;
            ui.global_hunger_index = *global_hunger_index;
            ui.actual_infant_death_ratio = *actual_infant_death_ratio;
        };
    }
}

fn update_environment_ui_data(
    mut uis: Query<(&WorldUiEntity, &mut EnvironmentUi)>,
    colonies: Query<&EnvironmentalHealth>,
) {
    for (WorldUiEntity(colony), mut ui) in uis.iter_mut() {
        if let Ok(EnvironmentalHealth {
            indoor_air_pollution,
            drinking_water,
            urban_particulates,
        }) = colonies.get(*colony)
        {
            ui.indoor_air_pollution = *indoor_air_pollution;
            ui.drinking_water = *drinking_water;
            ui.urban_particulates = *urban_particulates;
        }
    }
}

fn update_ecosystem_ui_data(
    mut uis: Query<(&WorldUiEntity, &mut EcosystemUi)>,
    colonies: Query<&EcosystemVitality>,
) {
    for (WorldUiEntity(colony), mut ui) in uis.iter_mut() {
        if let Ok(EcosystemVitality {
            air_quality_index,
            productive_natural_resources,
            biodiversity,
        }) = colonies.get(*colony)
        {
            ui.air_quality_index = *air_quality_index;
            ui.productive_natural_resources = *productive_natural_resources;
            ui.biodiversity = *biodiversity;
        }
    }
}

fn update_wealth_ui_data(
    mut uis: Query<(&WorldUiEntity, &mut WealthUi)>,
    colonies: Query<&Treasury>,
) {
    for (WorldUiEntity(colony), mut ui) in uis.iter_mut() {
        if let Ok(treasury) = colonies.get(*colony) {
            ui.total_wealth = treasury.total_wealth;
            ui.spending_available = treasury.spending_available;
            ui.citizen_payout = treasury.citizen_payout;
            ui.civil_spending = treasury.total_civil_spending();
            ui.sanitation_spending = treasury.total_sanitation_spending();
            ui.social_spending = treasury.total_social_spending();
            ui.environmental_spending = treasury.total_environmental_spending();
        }
    }
}
