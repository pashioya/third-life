use bevy::prelude::*;
use std::{collections::HashMap, time::Duration, usize};

#[derive(Component)]
pub struct WorldUi;

#[derive(Component)]
pub struct WorldUiName(pub String);

#[derive(Component, Default)]
pub struct WorldUiSize {
    pub size: f32,
    pub farm_space: f32,
    pub human_space: f32,
}

impl WorldUiSize{
    fn new(size: f32) -> Self {
        WorldUiSize{ size, ..default() }
    }
}

#[derive(Component, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct WorldUiEntity(pub Entity);

#[derive(Component, Default)]
pub struct PopulationUiData {
    pub count: usize,
    pub working_pop: usize,
    pub younglings: usize,
    pub retirees: usize,
    pub average_age: usize,
    pub ages: HashMap<usize, usize>,
    pub average_children_per_mother: f32,
    pub births_per_age: HashMap<usize, usize>,
    pub num_couples: usize,
    pub males: usize,
    pub females: usize,
    pub working_in_wheat: usize,
    pub working_in_beef: usize,
    pub working_not_in_farming: usize,
    pub total_births: usize,
}

#[derive(Component)]
pub struct PopulationDeathLines {
    pub timer: Timer,
    pub old_age_deaths: Vec<usize>,
    pub starvation_deaths: Vec<usize>,
    pub infant_deaths: Vec<usize>,
}

impl PopulationDeathLines {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            old_age_deaths: vec![0],
            starvation_deaths: vec![0],
            infant_deaths: vec![0],
        }
    }
    pub fn new_step(&mut self, delta: Duration) {
        if self.timer.tick(delta).just_finished() {
            self.old_age_deaths.push(0);
            self.starvation_deaths.push(0);
            self.infant_deaths.push(0);
        }
        if self.old_age_deaths.len() > 25 {
            self.old_age_deaths.remove(0);
        }
        if self.starvation_deaths.len() > 25 {
            self.starvation_deaths.remove(0);
        }
        if self.infant_deaths.len() > 25 {
            self.infant_deaths.remove(0);
        }
    }
}

#[derive(Component)]
pub struct ResourceStorage {
    pub timer: Timer,
    pub meat: Vec<f32>,
    pub carb: Vec<f32>,
}

impl ResourceStorage {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            meat: vec![0.],
            carb: vec![0.],
        }
    }
}
#[derive(Component, Default)]
pub struct ResourceConsumption {
    pub meat: f32,
    pub carb: f32,
}

#[derive(Component, Default)]
pub struct ResourceProduction {
    pub meat: f32,
    pub carb: f32,
}

#[derive(Component, Default)]
pub struct FarmsCount {
    pub meat: usize,
    pub carb: usize,
}

#[derive(Bundle)]
pub struct WorldUiBundle {
    pub ui: WorldUi,
    pub name: WorldUiName,
    pub size: WorldUiSize,
    pub entity: WorldUiEntity,
    pub pop: PopulationUiData,
    pub deaths: PopulationDeathLines,
    pub farms_count: FarmsCount,
    pub prod: ResourceProduction,
    pub cons: ResourceConsumption,
    pub stor: ResourceStorage,
    pub infra: InfrastructureUi,
    pub env: EnvironmentUi,
    pub eco: EcosystemUi,
    pub wealth: WealthUi,
}

impl WorldUiBundle {
    pub fn new(name: String, entity: Entity, size: f32) -> Self {
        Self {
            ui: WorldUi,
            name: WorldUiName(name),
            size: WorldUiSize::new(size),
            entity: WorldUiEntity(entity),
            pop: PopulationUiData::default(),
            deaths: PopulationDeathLines::new(),
            farms_count: FarmsCount::default(),
            prod: ResourceProduction::default(),
            cons: ResourceConsumption::default(),
            stor: ResourceStorage::new(),
            infra: InfrastructureUi::default(),
            env: EnvironmentUi::default(),
            eco: EcosystemUi::default(),
            wealth: WealthUi::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct InfrastructureUi {
    pub urbanization_index: f32,
    pub farming_mechanization: f32,
    pub health_index_score: f32,
    pub live_birth_mortality_rate: f32,
    pub global_hunger_index: f32,
    pub actual_infant_death_ratio: f32,
}

#[derive(Component, Default)]
pub struct EnvironmentUi {
    pub indoor_air_pollution: f32,
    pub drinking_water: f32,
    pub urban_particulates: f32,
}

#[derive(Component, Default)]
pub struct EcosystemUi {
    pub air_quality_index: f32,
    pub productive_natural_resources: f32,
    pub biodiversity: f32,
}

#[derive(Component, Default)]
pub struct WealthUi {
    pub total_wealth: f32,
    pub spending_available: f32,
    pub citizen_payout: f32,
    pub civil_spending: f32,
    pub sanitation_spending: f32,
    pub social_spending: f32,
    pub environmental_spending: f32,
}
