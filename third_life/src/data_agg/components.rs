use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use chrono::NaiveDate;
use influxdb2::Client;
use influxdb2_derive::WriteDataPoint;

use crate::worlds::{food::{components::{CarbResource, MeatResource}, events::{CarbConsumedEvent, MeatConsumedEvent}}, population::{components::Population, events::{self, CitizenCreated, CitizenDied}}};

use super::utils::{entity_to_str, to_timestamp};


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
pub struct PopulationRecord {
    #[influxdb(timestamp)]
    time: i64,
    #[influxdb(field)]
    total_pop: f64,
    #[influxdb(field)]
    average_age: f64,
    #[influxdb(field)]
    younglings: f64,
    #[influxdb(field)]
    working_pop: f64,
    #[influxdb(field)]
    retirees: f64,
    #[influxdb(field)]
    average_children_per_mother: f64,
    #[influxdb(tag)]
    colony_entity: String,
}

#[derive(Default, WriteDataPoint, Clone)]
pub struct CitizenDiedRecord {
    #[influxdb(timestamp)]
    time: i64,
    #[influxdb(field)]
    deaths: i64,
    #[influxdb(tag)]
    reason: String,
    #[influxdb(tag)]
    colony_entity: String,
}

#[derive(Default, WriteDataPoint, Clone)]
pub struct CitizenCreatedRecord {
    #[influxdb(timestamp)]
    time: i64,
    #[influxdb(field)]
    number: i64,
    #[influxdb(tag)]
    colony_entity: String,
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

#[derive(Default, WriteDataPoint, Clone)]
pub struct MeatConsumedRecord {
    #[influxdb(timestamp)]
    time: i64,
    #[influxdb(field)]
    consumed: f64,
    #[influxdb(tag)]
    colony_entity: String,
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

#[derive(Default, WriteDataPoint, Clone)]
pub struct CarbConsumedRecord {
    #[influxdb(timestamp)]
    time: i64,
    #[influxdb(field)]
    consumed: f64,
    #[influxdb(tag)]
    colony_entity: String,
}

impl PopulationRecord {
    pub fn from_population(
        date: &NaiveDate,
        pop: &Population,
        colony: &Entity
    ) -> Self {
        Self { 
            time: to_timestamp(date),
            total_pop: pop.count as f64,
            average_age: pop.average_age as f64,
            younglings: pop.younglings as f64,
            working_pop: pop.working_pop as f64,
            retirees: pop.retirees as f64,
            average_children_per_mother: pop.average_children_per_mother as f64,
            colony_entity: entity_to_str(colony)
        }
    }
}

impl MeatResourceRecord {
    pub fn from_resource(
        date: &NaiveDate,
        res: &MeatResource,
        colony: &Entity,
    ) -> Self {
        Self { 
            time: to_timestamp(date),
            amount: res.get_kgs() as f64,
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
            amount: res.get_kgs() as f64,
            colony_entity: entity_to_str(colony),
            resource_type: String::from("carb")
        }
    }
}

impl CitizenDiedRecord {
    pub fn from_events(
        date: &NaiveDate,
        events: Vec<&CitizenDied>,
    ) -> Vec<Self> {
        events.into_iter()
            .fold(HashMap::new(), |mut acc, event| {
                *acc.entry((&event.reason, event.colony)).or_insert(0) += 1;
                acc
            }).into_iter()
            .map(|((reason, colony), deaths)| {
                Self {
                    time: to_timestamp(date),
                    deaths,
                    reason: reason.to_lowercase(),
                    colony_entity: entity_to_str(&colony),
                }
            }).collect()

    }
}

impl CitizenCreatedRecord {
    pub fn from_events(
        date: &NaiveDate,
        events: Vec<&CitizenCreated>,
    ) -> Vec<Self> {
        events.into_iter()
            .fold(HashMap::new(), |mut acc, event| {
                *acc.entry(event.colony).or_insert(0) += 1;
                acc
            }).into_iter()
            .map(|(colony, number)| {
                Self {
                    time: to_timestamp(date),
                    number,
                    colony_entity: entity_to_str(&colony),
                }
            }).collect()
    }
}

impl MeatConsumedRecord {
    pub fn from_events(
        date: &NaiveDate,
        events: Vec<&MeatConsumedEvent>
    ) -> Vec<Self> {
        events.into_iter()
            .fold(HashMap::new(), |mut acc, event| {
                *acc.entry(event.colony).or_insert(0.) += event.amount as f64;
                acc
            }).into_iter()
            .map(|(colony, consumed)| {
                Self {
                    time: to_timestamp(date),
                    consumed,
                    colony_entity: entity_to_str(&colony)
                }
            }).collect()
    }
}

impl CarbConsumedRecord {
    pub fn from_events(
        date: &NaiveDate,
        events: Vec<&CarbConsumedEvent>
    ) -> Vec<Self> {
        events.into_iter()
            .fold(HashMap::new(), |mut acc, event| {
                *acc.entry(event.colony).or_insert(0.) += event.amount as f64;
                acc
            }).into_iter()
            .map(|(colony, consumed)| {
                Self {
                    time: to_timestamp(date),
                    consumed,
                    colony_entity: entity_to_str(&colony)
                }
            }).collect()
    }
}
