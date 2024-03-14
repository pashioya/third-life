use std::{str::FromStr, sync::Arc};

use bevy::prelude::*;
use chrono::NaiveDate;
use influxdb2::Client;
use influxdb2_derive::WriteDataPoint;

use crate::worlds::food::components::{CarbResource, MeatResource};

use super::utils::to_timestamp;


#[derive(Resource)]
pub struct InfluxDB {
    pub client: Arc<Client>,
    pub bucket: Arc<String>,
}

impl InfluxDB {
    pub fn new(client: Client, bucket: String) -> Self {
        InfluxDB { client: client.into(), bucket: bucket.into() }
    }
    pub fn get(&self) -> (Arc<Client>, Arc<String>) {
        (Arc::clone(&self.client), Arc::clone(&self.bucket))
    }
}


#[derive(Default, WriteDataPoint, Clone)]
pub struct MeatResourceRecord {
    #[influxdb(timestamp)]
    time: i64,
    #[influxdb(field)]
    amount: f64,
    #[influxdb(tag)]
    colony_entity: String,
    #[influxdb(tag)]
    resource_type: String,
}

#[derive(Default, WriteDataPoint)]
pub struct CarbResourceRecord {
    #[influxdb(timestamp)]
    time: i64,
    #[influxdb(field)]
    amount: f64,
    #[influxdb(tag)]
    colony_entity: String,
    #[influxdb(tag)]
    resource_type: String,
}

impl MeatResourceRecord {
    pub fn from_resource(
        date: &NaiveDate,
        res: &MeatResource,
        colony: &Entity,
    ) -> Self {
        Self { 
            time: to_timestamp(date),
            amount: res.amount as f64,
            colony_entity: format!("{colony:?}"),
            resource_type: String::from("carb")
        }
    }
}

impl CarbResourceRecord {
    pub fn from_resource(
        date: &NaiveDate,
        res: &CarbResource,
        colony: &Entity,
    ) -> Self {
        Self { 
            time: to_timestamp(date),
            amount: res.amount as f64,
            colony_entity: format!("{colony:?}"),
            resource_type: String::from("carb")
        }
    }
}
