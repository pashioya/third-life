#![allow(dead_code)]
//! Simulation of worlds and their inhabitatnts. Models a number of things
//! which have been researched but still dumbed down to make them easier to
//! implement. For more details check out the [Gitlab Wiki](https://gitlab.com/groups/kdg-ti/the-lab/teams-23-24/third-life/-/wikis/home)

pub(crate) mod common;
/// Coponets for the reading and creation of config Files
pub(crate) mod config;
pub(crate) mod time;
pub(crate) mod worlds;
pub(crate) mod animation;
pub(crate) mod ui;
mod data_agg;



use animation::ThirdLifeAnimationPlugin;
use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use config::ConfigurationPlugin;
use data_agg::DataAggPlugin;
use time::TimeDatePlugin;
use ui::ThridLifeUiPlugin;
use worlds::WorldsPlugin;

/// State of the simulation, manages the initial configuration load
/// menu.
///
/// Has three states in which the configuration can be selected the
/// configuration is loading and then has finished loading. The intermediary
/// step is important becase initializasions needs the configuration to be
/// completly loaded.
#[derive(Debug, Clone, States, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum SimulationState {
    #[default]
    ConfigSelection,
    LoadingConfig,
    FinishedLoadingConfig,
    LoadingDatabase,
    FinishedLoadingDatabase,
    Running,
}

fn main() {
    App::new()
        .init_state::<SimulationState>()
        .add_systems(
            OnEnter(SimulationState::FinishedLoadingConfig),
            start_loading_database
        )
        .add_systems(
            OnEnter(SimulationState::FinishedLoadingDatabase),
            start_running
        )
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "Third Life".into(),
                        resolution: (800.0, 600.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
            EguiPlugin,
            ConfigurationPlugin,
            TimeDatePlugin,
            WorldsPlugin,
            ThridLifeUiPlugin,
            ThirdLifeAnimationPlugin,
            DataAggPlugin
        ))
        .run();
}

fn start_loading_database(mut sim_state: ResMut<NextState<SimulationState>>) {
    println!("starting the loading of the db");
    sim_state.set(SimulationState::LoadingDatabase);
}

fn start_running(mut sim_state: ResMut<NextState<SimulationState>>) {
    println!("running the sim");
    sim_state.set(SimulationState::Running);
}
