use std::ops::Deref;

use crate::{data_agg::components::LoadingDatabase, SimulationState};
use bevy::{prelude::*, utils::uuid};
use chrono::Local;
use influxdb2::api::organization::ListOrganizationRequest;
use influxdb2::models::PostBucketRequest;
use influxdb2::Client;

use ::uuid::Uuid;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};

use super::components::{InfluxDB, LoadedDatabaseEvent, LoadingDatabases, RegisterDatabseEvent, SimulationUuid};

const INFLUX_DB: &'static str = "influx_db";
const INFLUX_DB_ORG_NAME: &'static str = "third-life-team";

pub struct InfluxDbPlugin;

impl Plugin for InfluxDbPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(
            Startup, 
            register_influx
        )
        .add_systems(
            Update,
            init_influx_db.run_if(in_state(SimulationState::LoadingDatabases)),
        );
    }
}

pub fn register_influx(mut register_influx: EventWriter<RegisterDatabseEvent>) {
    info!("Registering Influx DB.");
    register_influx.send(RegisterDatabseEvent(String::from(INFLUX_DB)));
}

pub fn init_influx_db(
    mut commands: Commands,
    mut task_executor: AsyncTaskRunner<InfluxDB>,
    mut loaded_database: EventWriter<LoadedDatabaseEvent>,
    simulation_uuid: Res<SimulationUuid>,
    loading_databases: Res<LoadingDatabases>,
) {
    let simulation_uuid = simulation_uuid.deref().deref().clone();
    
    match task_executor.poll() {
        AsyncTaskStatus::Idle => {
            let Some(status) = loading_databases.0.get(INFLUX_DB) else {
                panic!("influx db was never registered!");
            };
            if *status != LoadingDatabase::Waiting {
                return;
            };
            info!("Starting Influx DB connection.");
            task_executor.start(async move {
                let client = Client::new("http://localhost:8086", "third-life-team", "admin-token");

                let organization_id = client
                    .list_organizations(ListOrganizationRequest {
                        ..Default::default()
                    })
                    .await
                    .unwrap()
                    .orgs
                    .into_iter()
                    .filter(|org| org.name == INFLUX_DB_ORG_NAME)
                    .map(|org| org.id.clone().unwrap())
                    .next()
                    .unwrap();


                let new_bucket_name = format!("third_life_sim_{}", simulation_uuid);

                let result = client
                    .create_bucket(Some(PostBucketRequest::new(
                        organization_id,
                        new_bucket_name.clone(),
                    )))
                    .await;

                println!(
                    "Task finished: InfluxDb client Initialization was successful?: {}",
                    result.is_ok()
                );
                InfluxDB::new(client, new_bucket_name)
            });
        },
        AsyncTaskStatus::Pending => (),
        AsyncTaskStatus::Finished(influx_db) => {
            info!("Finished Influx DB connection.");
            commands.insert_resource(influx_db);
            loaded_database.send(LoadedDatabaseEvent(String::from(INFLUX_DB)));
        }
    }
}
