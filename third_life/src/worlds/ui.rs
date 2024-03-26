mod components;
mod population_ui;
mod resources_ui;
mod inf_env_eco_wealth_ui;

use components::*;
use population_ui::*;
use resources_ui::*;

use bevy::prelude::*;
use bevy_egui::{egui::{ScrollArea, Window}, EguiContexts};
use chrono::NaiveDate;
use egui_plot::{PlotPoint, PlotPoints};
use crate::{config::ThirdLifeConfig, time::GameDate, SimulationState};

use self::inf_env_eco_wealth_ui::{inf_env_eco_wealth_info, InfEnvEcoWealthPlugin};

use super::{init_colonies, WorldColony, WorldEntity};


pub struct WorldsUiPlugin;

impl Plugin for WorldsUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(SimulationState::Running), init_worlds_windows.after(init_colonies))
            .add_systems(Update, (display_world_uis, update_size_used).run_if(in_state(SimulationState::Running)))
            .add_plugins((PopulationUiPlugin, ResourcesUiPlugin, InfEnvEcoWealthPlugin));
    }
}

fn init_worlds_windows(
    mut commands: Commands,
    worlds: Query<(Entity, &WorldEntity, &WorldColony)>,
) {
    for (entity, world, world_colony) in &worlds {
        commands.spawn(WorldUiBundle::new(world.name.clone(), entity, world_colony.size));
    }
}

fn display_world_uis(
    mut contexts: EguiContexts,
    config: Res<ThirdLifeConfig>,
    game_date: Res<GameDate>,
    ui_data: Query<(
        &WorldUiName,
        &WorldUiSize,
        &ResourceStorage,
        &PopulationUiData,
        &PopulationDeathLines,
        &FarmsCount,
        &ResourceProduction,
        &ResourceConsumption,
        &InfrastructureUi,
        &EnvironmentUi,
        &EcosystemUi,
        &WealthUi,
    )>,
) {
    for (world, size, stor, pop, death, farms, prod, cons, inf, env, eco, wealth) in &ui_data {
        let name = &world.0;
        Window::new(format!("Window of {name}"))
            .default_open(true)
            .show(contexts.ctx_mut(), |ui| {
                ScrollArea::new([false, true])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Size {:.3}/{} hectares", 
                                (size.human_space + size.farm_space),
                                size.size
                            ));
                            ui.label(format!("farm space: {} hectares", size.farm_space));
                            ui.label(format!("human space: {} m^2", size.human_space * 10_000.));
                        });
                        let start_date = NaiveDate::from_ymd_opt(config.starting_day().year(),config.starting_day().month(), config.starting_day().day()).unwrap();
                        ui.label(format!("Date: {}", game_date.date));
                        ui.label(format!("Years Elapsed: {}", game_date.date.years_since(start_date).unwrap()));
                        ui.separator();
                        resources(name, ui, &stor, &farms, &prod, &cons);
                        ui.separator();
                        general_pop(ui, &pop);
                        ui.separator();
                        age_histogram(name, ui, &pop.ages);
                        ui.separator();
                        death_lines(name, ui, death);
                        ui.separator();
                        inf_env_eco_wealth_info(ui, inf, env, eco, wealth);
                        ui.separator();
                        age_histogram(&format!("{name} birth ages"), ui, &pop.births_per_age);
                    });
            });
    }
}

fn update_size_used(
    mut ui_size: Query<(&WorldUiEntity, &mut WorldUiSize)>,
    size: Query<(Entity, &WorldColony)>
) {
    for (WorldUiEntity(world_ui_entity), mut world_ui_size) in ui_size.iter_mut() {
        let entity = size.get(*world_ui_entity).unwrap().1;
        world_ui_size.human_space = entity.human_space();
        world_ui_size.farm_space = entity.farm_space();
    }
}


pub fn f32_to_plotpoints(
    vec: &Vec<f32>
) -> PlotPoints {
    let vec = vec.into_iter().enumerate()
        .map(|(i, n)| PlotPoint::new(i as f64, *n))
        .collect::<Vec<_>>();
    PlotPoints::Owned(vec)
}

pub fn usize_to_plotpoints(
    vec: &Vec<usize>
) -> PlotPoints {
    let vec = vec.into_iter().enumerate()
        .map(|(i, n)| PlotPoint::new(i as f64, *n as f64))
        .collect::<Vec<_>>();
    PlotPoints::Owned(vec)
}
