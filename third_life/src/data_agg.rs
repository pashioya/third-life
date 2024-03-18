mod components;
mod food_data;
mod influx_conn;
mod influx_types;
mod population_data;
mod postgres_conn;
mod utils;

use core::panic;

use crate::SimulationState;
use bevy::prelude::*;
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

use self::components::{
    InfluxDB, LoadedDatabaseEvent, LoadingDatabase, LoadingDatabases, RegisterDatabseEvent,
    ShouldSaveToDatabase, SimulationUuid,
};
use self::food_data::{
    carb_consumption_recording, carb_recording, meat_consumption_recording, meat_recording,
};
use self::influx_conn::InfluxDbPlugin;
use self::population_data::{birth_recording, death_recording, population_recording};
use self::postgres_conn::PostgresDbPlugin;

pub struct DataAggPlugin;

impl Plugin for DataAggPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                population_recording,
                birth_recording,
                death_recording,
                meat_recording,
                carb_recording,
                meat_consumption_recording,
                carb_consumption_recording,
            )
                .run_if(in_state(SimulationState::Running))
                .run_if(in_state(ShouldSaveToDatabase::Yes)),
        )
        .add_systems(
            Update,
            show_saving_choice.run_if(in_state(SimulationState::SaveToDatabaseChoice)),
        )
        .add_systems(
            Update,
            recive_databse_registrations.run_if(not(in_state(SimulationState::Running))),
        )
        .add_systems(
            Update,
            recive_finished_loading_database.run_if(in_state(SimulationState::LoadingDatabases)),
        )
        .add_event::<RegisterDatabseEvent>()
        .add_event::<LoadedDatabaseEvent>()
        .init_resource::<LoadingDatabases>()
        .init_resource::<SimulationUuid>()
        .init_state::<ShouldSaveToDatabase>()
        .add_plugins((PostgresDbPlugin, InfluxDbPlugin));
    }
}

fn show_saving_choice(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<SimulationState>>,
    mut should_save_state: ResMut<NextState<ShouldSaveToDatabase>>,
) {
    Window::new("choose whether to save to db or not").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            if ui.button("dont save data").clicked() {
                should_save_state.set(ShouldSaveToDatabase::No);
                next_state.set(SimulationState::FinishedLoadingDatabases);
            }
            if ui.button("save data").clicked() {
                should_save_state.set(ShouldSaveToDatabase::Yes);
                next_state.set(SimulationState::LoadingDatabases);
            }
        });
    });
}

fn recive_databse_registrations(
    mut event_reader: EventReader<RegisterDatabseEvent>,
    mut loading_databases: ResMut<LoadingDatabases>,
) {
    for event in event_reader.read() {
        if loading_databases.0.contains_key(&event.0) {
            panic!("Database is beeing registered twice: {}", event.0);
        }
        loading_databases
            .0
            .insert(event.0.clone(), LoadingDatabase::Waiting);
    }
}

fn recive_finished_loading_database(
    mut event_reader: EventReader<LoadedDatabaseEvent>,
    mut loading_databases: ResMut<LoadingDatabases>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    for event in event_reader.read() {
        let Some(status) = loading_databases.0.get_mut(&event.0) else {
            panic!("Database has not been registered: {}", event.0);
        };
        match status {
            LoadingDatabase::Waiting => *status = LoadingDatabase::Recived,
            _ => panic!(
                "Database finished loading event has been sent out twice: {}",
                event.0
            ),
        }
    }
    if loading_databases
        .0
        .iter()
        .all(|(_, status)| *status == LoadingDatabase::Recived)
    {
        next_state.set(SimulationState::FinishedLoadingDatabases);
    }
}
