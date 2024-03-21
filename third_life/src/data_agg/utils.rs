

use bevy::{ecs::query::QueryData, prelude::*};

use bevy_async_task::AsyncTaskPool;
use chrono::{NaiveDate, NaiveTime};
use futures::stream;
use influxdb2::models::WriteDataPoint;

use crate::time::DateChanged;

use super::components::InfluxDB;

pub fn to_timestamp(date: &NaiveDate) -> i64 {
    date.and_time(NaiveTime::default()).timestamp_nanos_opt().unwrap()
}

pub fn entity_to_str(entity: &Entity) -> String {
    format!("{entity:?}")
}


pub fn to_influx_mapped<Data, Record>(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: &InfluxDB,
    mapping_fn: impl Fn(NaiveDate, Data) -> Record,
    data: Vec<Data>,
    mut day_changed: EventReader<DateChanged>,
)
where
    Data: QueryData,
    Record: WriteDataPoint + Sync + Send + 'static,  
{
    if let Some(DateChanged { date }) = day_changed.read().last() {
        let records: Vec<Record> = data.into_iter()
            .map(|e|mapping_fn(*date, e))
            .collect::<Vec<_>>();

        let (client, bucket) = influxdb.get();
        task_pool.spawn(async move {
            let _ = client.write(&bucket, stream::iter(records)).await;
        });
    }
}

pub fn to_influx_vec<Record>(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: &InfluxDB,
    data: Vec<Record>,
    mut day_changed: EventReader<DateChanged>,
)
where
    Record: WriteDataPoint + Sync + Send + 'static,  
{
    if let Some(_) = day_changed.read().last() {

        let (client, bucket) = influxdb.get();
        task_pool.spawn(async move {
            let _ = client.write(&bucket, stream::iter(data)).await;
        });
    }
}
