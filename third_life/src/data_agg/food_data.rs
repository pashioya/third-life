
use crate::time::DateChanged;
use crate::worlds::food::components::{CarbResource, MeatResource};
use crate::{time::GameDate, worlds::food::components::ResourceOf};
use bevy::prelude::*;
use bevy::tasks::futures_lite::stream;

use bevy_async_task::AsyncTaskPool;
use chrono::Local;
use influxdb2::models::DataPoint;

use super::components::{CarbResourceRecord, MeatResourceRecord};
use super::InfluxDB;

pub fn meat_recording(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    _day_changed: EventReader<DateChanged>,
    meat_resources: Query<(&MeatResource, &ResourceOf)>,
) {
    let records = meat_resources.into_iter()
        .map(|(res, ResourceOf { colony })| MeatResourceRecord::from_resource(
                &game_date.date, res, colony
        )).collect::<Vec<_>>();

    let (client, bucket) = influxdb.get();
    task_pool.spawn(async move {
        let _ = client.write(&bucket, stream::iter(records)).await;
    });
}

pub fn carb_recording(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    _day_changed: EventReader<DateChanged>,
    meat_resources: Query<(&CarbResource, &ResourceOf)>,
) {
    let records = meat_resources.into_iter()
        .map(|(res, ResourceOf { colony })| CarbResourceRecord::from_resource(
                &game_date.date, res, colony
        )).collect::<Vec<_>>();

    let (client, bucket) = influxdb.get();
    task_pool.spawn(async move {
        let _ = client.write(&bucket, stream::iter(records)).await;
    });
}

/*pub fn food_queue(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    carb_resources: Query<(&CarbResource, &ResourceOf)>,
    meat_resources: Query<(&MeatResource, &ResourceOf)>,
    worlds: Query<(Entity, &WorldEntity)>,
) {
    let date_time = Utc::from_local_datetime(
        &Utc,
        &NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                game_date.date.year(),
                game_date.date.month(),
                game_date.date.day(),
            )
            .unwrap(),
            NaiveTime::from_hms_nano_opt(0, 0, 0, 0).unwrap(),
        ),
    )
    .unwrap()
    .timestamp_nanos_opt()
    .unwrap();

    for (entity, world) in worlds.iter() {
        let filtered_resources = food_resources
            .iter()
            .zip(carb_resources.iter())
            .zip(meat_resources.iter())
            .filter_map(
                |(
                    ((food, food_resource_of), (carb, carb_resource_of)),
                    (meat, meat_resource_of),
                )| {
                    if food_resource_of.colony == entity
                        && carb_resource_of.colony == entity
                        && meat_resource_of.colony == entity
                    {
                        Some((food_resource_of, carb, meat, food))
                    } else {
                        None
                    }
                },
            )
            .collect::<Vec<_>>();

        for (_, carb, meat, food) in filtered_resources {
            let data = vec![DataPoint::builder("resources")
                .tag("world", world.name.clone())
                .field("carb", carb.amount as f64)
                .field("meat", meat.amount as f64)
                .field("food", food.amount as f64)
                //.timestamp(date_time)
                .field("game_date", date_time)
                .build()
                .unwrap()];

            let client = influxdb.client.clone();
            let bucket = influxdb.bucket.clone();
            let task = AsyncTask::new(async move {
                let result = write_data(client, bucket.as_str(), data.clone()).await;
                if !result.is_ok() {
                    println!("Error writing data to InfluxDB");
                    println!("{:?}", result.err());
                }
            });
            task_pool.spawn(task);
        }
    }
}*/
