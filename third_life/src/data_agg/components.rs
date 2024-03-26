use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use sqlx::{Pool, Postgres};
use uuid::Uuid;






#[derive(Resource)]
pub struct PostgresDB {
    pub pool: Arc<Pool<Postgres>>,
}

#[derive(Resource, Clone)]
pub struct SimulationUuid{
    pub uuid: Uuid,
    pub name: String,
}


impl PostgresDB {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool: Arc::new(pool) }
    }
    pub fn get(&self) -> Arc<Pool<Postgres>> {
        Arc::clone(&self.pool)
    }
}

impl Default for SimulationUuid {
    fn default() -> Self {
        Self { 
            uuid: Uuid::new_v4(),
            name: String::from("this simulation has no name")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoadingDatabase {
    Waiting,
    Recived
}


#[derive(Debug, Clone, Default, States, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum ShouldSaveToDatabase {
    #[default]
    Yes,
    No
}

#[derive(Resource, Default)]
pub struct LoadingDatabases(pub HashMap<String, LoadingDatabase>);

#[derive(Event)]
pub struct RegisterDatabseEvent(pub String);

#[derive(Event)]
pub struct LoadedDatabaseEvent(pub String);



