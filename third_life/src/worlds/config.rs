#![allow(dead_code)]


use proc_macros::{ConfigFile, Config};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::ConfigurationLoader;
impl ConfigurationLoader for WorldsConfig {
    fn path_with_name() -> &'static str {
        "worlds"
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Resource, ConfigFile, Default, Config)]
pub struct WorldsConfig {
    worlds: Vec<WorldConfig>
}

#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config, Component)]
pub struct WorldConfig {
    /// Name should be unique, since its used for identification of multiple 
    /// things.
    name: String,
    #[def(300.0)]
    size: Option<f32>,
    #[def(8.)]
    work_day_length: Option<f32>,
    world_position: (isize, isize),
    #[def(PopulationConfig::def_conf())]
    population: Option<PopulationConfig>,
    #[def(GovernmentConfig::def_conf())]
    government: Option<GovernmentConfig>,
    #[def(EnvironmentConfig::def_conf())]
    environment: Option<EnvironmentConfig>,
    #[def(FoodConfig::def_conf())]
    food: Option<FoodConfig>,
    sprite: SpriteConfig,
}

/// Different parameters affecting the population directly
#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct PopulationConfig {
    /// Starting number of Peple. Any Real number
    #[def(1000)]
    population_size: Option<u32>,
    ///age of adult and retirement
    #[def(18)]
    age_of_adult: Option<usize>,
    #[def(65)]
    age_of_retirement: Option<usize>,

    #[def(PopulationDistributionConfig::def_conf())]
    population_dist: Option<PopulationDistributionConfig>,
    /// The spread in this case refers to at which age the probability of death
    /// starts to increase. Does not affect the actual average life expectancy.
    #[def(6.)]
    life_expectancy_spread: Option<f32>,
    /// Where the average person falls in terms of bodyweight
    #[def(WeightDistributionConfig::def_conf())]
    weight_dist: Option<WeightDistributionConfig>,
    /// Where the average person falls in terms of height
    #[def(HeightDistributionConfig::def_conf())]
    height_dist: Option<HeightDistributionConfig>,
    /// The target ratio of macros in a colony. Has to add up to make 100
    #[def(DietMacroRatiosConfig::def_conf())]
    diet_macro_ratios: Option<DietMacroRatiosConfig>
}

#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct PopulationDistributionConfig {
    /// Location of the Skew normal distribution. Any Positive number
    #[def(18.)]
    location: Option<f32>,
    /// Scale of the Skew normal distribution. Any Real number
    #[def(6.)]
    scale: Option<f32>,
    /// Shape of the Skew normal distribution
    #[def(10.)]
    shape: Option<f32>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct WeightDistributionConfig {
    #[def(70.)]
    average: Option<f32>
}
#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct HeightDistributionConfig {
    #[def(180.)]
    average: Option<f32>
}

#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct DietMacroRatiosConfig {
    #[def(20)]
    protein: usize,
    #[def(30)]
    fat: usize,
    #[def(50)]
    carbs: usize
}

#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct GovernmentConfig {
    #[def(0.1)]
    citizen_payout: Option<f32>,
    #[def(25)]
    civil_spending: Option<usize>,
    #[def(25)]
    sanitation_spending: Option<usize>,
    #[def(25)]
    social_spending: Option<usize>,
    #[def(25)]
    environmental_spending: Option<usize>,
}

/// General factors of the environment of the world
#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct EnvironmentConfig {
    #[def(0.5)]
    urbanization: Option<f32>,
    #[def(1.)]
    env_health: Option<f32>,
    #[def(1.)]
    ecosystem_vitylity: Option<f32>,
}


#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct FoodConfig {
    #[def(6)]
    cow_farms: Option<usize>,
    #[def(34.0)]
    cow_farm_size: Option<f32>,
    #[def(4)]
    wheat_farms: Option<usize>,
    #[def(17.4)]
    wheat_farm_size: Option<f32>,
    #[def(5000.0)]
    starting_beef: Option<f32>,
    #[def(5000.0)]
    starting_carb: Option<f32>,
    #[def(1.0)]
    min_surplus_multiplier: Option<f32>,
    #[def(5.0)]
    max_surplus_multiplier: Option<f32>,

}

#[derive(Deserialize, Serialize, Debug, Clone, Resource, Default, Config)]
pub struct SpriteConfig {
    sprite_sheet: String,
    frames: usize,
    frames_layout: (usize, usize),
    shape: (usize, usize),
    animation_timer: f32
}


