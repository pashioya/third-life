use crate::time::{DateChanged, GameDate};
use crate::worlds::population::components::Population;
use crate::worlds::population::events::{CitizenCreated, CitizenDied};
use crate::worlds::WorldEntity;
use bevy::prelude::*;
use chrono::{NaiveDateTime, NaiveTime, TimeZone, Utc};
use futures::stream;
use influxdb2::models::DataPoint;

use bevy_async_task::{AsyncTask, AsyncTaskPool};

use super::components::{CitizenCreatedRecord, CitizenDiedRecord, InfluxDB, PopulationRecord};
use super::utils::{to_influx_mapped, to_influx_vec};



pub fn population_recording(
    task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    day_changed: EventReader<DateChanged>,
    populations: Query<(Entity, &Population)>,
) {
    let mapping_fn = |date, (entity, pop)| {
        PopulationRecord::from_population(
                &date, pop, &entity
        )
    };

    to_influx_mapped(
        task_pool, &influxdb, 
        mapping_fn, populations.into_iter().collect(),
        day_changed
    );
}

pub fn birth_recording(
    task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    day_changed: EventReader<DateChanged>,
    mut births: EventReader<CitizenCreated>
) {
    let records = CitizenCreatedRecord::from_events(
        &game_date.date, births.read().collect()
    );

    to_influx_vec(task_pool, &influxdb, records, day_changed);
}

pub fn death_recording(
    task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    day_changed: EventReader<DateChanged>,
    mut deaths: EventReader<CitizenDied>
) {
    let records = CitizenDiedRecord::from_events(
        &game_date.date, deaths.read().collect()
    );

    to_influx_vec(task_pool, &influxdb, records, day_changed);
}
