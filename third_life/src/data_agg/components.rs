use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use sqlx::{Pool, Postgres};
use uuid::Uuid;






#[derive(Resource)]
pub struct PostgresDB {
    pub pool: Arc<Pool<Postgres>>,
}

#[derive(Resource)]
pub struct SimulationUuid(pub Uuid);


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
        Self(Uuid::new_v4())
    }
}

impl std::ops::Deref for SimulationUuid {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
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



