
use crate::time::DateChanged;
use crate::worlds::food::components::{CarbResource, MeatResource};
use crate::worlds::food::events::{CarbConsumedEvent, MeatConsumedEvent};
use crate::{time::GameDate, worlds::food::components::ResourceOf};
use bevy::prelude::*;
use bevy::tasks::futures_lite::stream;

use bevy_async_task::AsyncTaskPool;

use super::components::{CarbConsumedRecord, CarbResourceRecord, MeatConsumedRecord, MeatResourceRecord};
use super::utils::{to_influx_mapped, to_influx_vec};
use super::InfluxDB;

pub fn meat_recording(
    task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    mut day_changed: EventReader<DateChanged>,
    meat_resources: Query<(&MeatResource, &ResourceOf)>,
) {
    let mapping_fn = |date, (res, &ResourceOf { colony })| {
        MeatResourceRecord::from_resource(
                &date, res, &colony
        )
    };
    to_influx_mapped(
        task_pool, &influxdb, 
        mapping_fn, meat_resources.into_iter().collect(),
        day_changed
    );
}

pub fn meat_consumption_recording(
    task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    day_changed: EventReader<DateChanged>,
    mut events: EventReader<MeatConsumedEvent>,
) {
    let data = MeatConsumedRecord::from_events(
        &game_date.date, events.read().collect()
    );

    to_influx_vec(task_pool, &influxdb, data, day_changed);

}

pub fn carb_recording(
    task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    day_changed: EventReader<DateChanged>,
    meat_resources: Query<(&CarbResource, &ResourceOf)>,
) {
    let mapping_fn = |date, (res, &ResourceOf { colony })| {
        CarbResourceRecord::from_resource(
                &date, res, &colony
        )
    };

    to_influx_mapped(
        task_pool, &influxdb, 
        mapping_fn, meat_resources.into_iter().collect(),
        day_changed
    );
}

pub fn carb_consumption_recording(
    task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    day_changed: EventReader<DateChanged>,
    mut events: EventReader<CarbConsumedEvent>,
) {
    let data = CarbConsumedRecord::from_events(
        &game_date.date, events.read().collect()
    );

    to_influx_vec(task_pool, &influxdb, data, day_changed);

}
