use crate::time::GameDate;
use crate::worlds::population::components::Population;
use crate::worlds::WorldEntity;
use bevy::prelude::*;
use chrono::{NaiveDateTime, NaiveTime, TimeZone, Utc};
use influxdb2::models::DataPoint;

use bevy_async_task::{AsyncTask, AsyncTaskPool};


/*
pub fn population_queue(
    mut task_pool: AsyncTaskPool<()>,
    influxdb: Res<InfluxDB>,
    game_date: Res<GameDate>,
    populations: Query<(&WorldEntity, &Population)>,
) {
    let date_time = Utc::from_local_datetime(
        &Utc,
        &NaiveDateTime::new(
            game_date.date,
            NaiveTime::from_hms_nano_opt(1, 1, 1, 1).unwrap(),
        ),
    )
    .unwrap()
    .to_string();

    for (world, population) in &populations {
        let data = vec![DataPoint::builder("population")
            .tag("world", world.name.clone())
            .field("population_count", population.count as i64)
            .field("average_age", population.average_age as f64)
            .field(
                "average_children_per_mother",
                population.average_children_per_mother as f64,
            )
            .field("game_date", date_time.clone())
            //.timestamp(date_time)
            .build()
            .unwrap()];

        let client = influxdb.client.clone();
        let bucket = influxdb.bucket.clone();
        let task = AsyncTask::new(async move {
            let result = write_data(client, bucket.as_str(), data).await;
            if !result.is_ok() {
                println!("Error writing population data to InfluxDB");
                println!("{:?}", result.err());
            }
        });
        let (fut, _) = task.into_parts();
        task_pool.spawn(async {
            fut.await;
        })
    }
}*/
