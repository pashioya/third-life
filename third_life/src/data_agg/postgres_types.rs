use std::collections::HashMap;

use bevy::prelude::*;
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct ColonyRecord {
    pub uuid: Uuid,
    pub date: NaiveDate,
    pub colony: i64,
    pub total_pop: i32,
    pub average_age: f32,
    pub younglings: i32,
    pub working_pop: i32,
    pub retirees: i32,
    pub average_children_per_mother: f32,
    pub infant_deaths: i32,
    pub starvation_deaths: i32,
    pub old_age_death: i32,
    pub citizen_created: i32,
    pub meat_resources: f32,
    pub meat_quality: f32,
    pub meat_consumed: f32,
    pub carb_resources: f32,
    pub carb_quality: f32,
    pub carb_consumed: f32,
    pub meat_produced: f32,
    pub carb_produced: f32,
    pub num_couples: i32,
    pub males: i32,
    pub females: i32,
    pub working_in_wheat: i32,
    pub working_in_beef: i32,
    pub working_not_in_farming: i32,
    pub total_births: i32,
}

impl ColonyRecord {
    pub fn new(uuid: Uuid, date: &NaiveDate, colony: Entity) -> Self {
        Self {
            uuid, date: *date, colony: colony.to_bits() as i64,
            ..default()
        }
    }
}


#[derive(Debug, Default, Resource)]
pub struct ColonyEventsContainer(pub HashMap<Entity, EventsContainer>);

impl std::ops::Deref for ColonyEventsContainer {
    type Target = HashMap<Entity, EventsContainer>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ColonyEventsContainer {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct EventsContainer {
    pub infant_deaths: i32,
    pub starvation_deaths: i32,
    pub old_age_death: i32,
    pub citizen_created: i32,
    pub meat_consumed: f32,
    pub meat_produced: f32,
    pub carb_consumed: f32,
    pub carb_produced: f32,
}
