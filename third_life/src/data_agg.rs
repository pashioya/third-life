mod food_data;
mod population_data;
mod components;
mod utils;


use crate::SimulationState;
use bevy::prelude::*;
use chrono::Local;
use influxdb2::api::organization::ListOrganizationRequest;
use influxdb2::models::PostBucketRequest;
use influxdb2::Client;

use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};

use self::components::InfluxDB;
use self::food_data::{carb_recording, meat_recording};

pub struct DataAggPlugin;

impl Plugin for DataAggPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
            Update,
            init_influx_db.run_if(in_state(SimulationState::LoadingDatabase)),
        )
        .add_systems(
            Update,
            (
                meat_recording, carb_recording
            ).run_if(in_state(SimulationState::Running)),
        );
    }
}


const INFLUX_DB_ORG_NAME: &'static str = "third-life-team";

fn init_influx_db(
    mut commands: Commands,
    mut task_executor: AsyncTaskRunner<InfluxDB>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    println!("Initializing DataAgg");

    let init_infra_task = async move {
        let client = Client::new("http://localhost:8086", "third-life-team", "admin-token");

        let organization_id = client
            .list_organizations(ListOrganizationRequest {
                ..Default::default()
            })
            .await.unwrap().orgs.into_iter()
            .filter(|org| org.name == INFLUX_DB_ORG_NAME)
            .map(|org| org.id.clone().unwrap())
            .next().unwrap();

        let new_bucket_name = format!(
            "third_life_sim_{}",
            Local::now().format("%d/%m/%Y_%H:%M")
        );

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
    };

    match task_executor.poll() {
        AsyncTaskStatus::Idle => task_executor.start(init_infra_task),
        AsyncTaskStatus::Pending => (),
        AsyncTaskStatus::Finished(influx_db) => {
            commands.insert_resource(influx_db);
            next_state.set(SimulationState::FinishedLoadingDatabase);
        }
    }

}

