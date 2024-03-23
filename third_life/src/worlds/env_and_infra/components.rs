//! Here 0 is always the bad value and 1 is the good value!

use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct ColonyInfraAndEnvBundle {
    civil: CivilInfrastructure,
    sanitation: SanitationInfrastructure,
    social: SocialInfrastructure,
    env_health: EnvironmentalHealth,
    ecosystem_vitality: EcosystemVitality,
}

/// Power, Water, Sewage and so on
/// Calculate the urbanization index using the following formula:
/// Urbanization Ratio = 1 - (Number of Farmers / Total Workforce)
#[derive(Component, Default)]
pub struct CivilInfrastructure {
    pub urbanization_index: f32,
    // number between 0 and 1 to define how mechanized farming is
    pub farming_mechanization: f32,
}

impl CivilInfrastructure {
    pub fn update(
        &mut self, spending: f32,
        work_force: usize, farmers_count: usize
    ) {
        self.farming_mechanization_fn(spending);
        self.urbanization_index_fn(work_force, farmers_count);
    }
    fn farming_mechanization_fn(&mut self, spending: f32) {
        self.farming_mechanization = corr_ln(
            0.38, -0.6, spending
        ).clamp(0., 1.);
    }
    fn urbanization_index_fn(&mut self, work_force: usize, farmers_count: usize) {
        self.urbanization_index = (1.0 - (farmers_count as f32 / work_force as f32)).clamp(0., 1.);
    }
}

/// Anything that has to do with sanitation like hospitals but also
/// things like bathrooms.
///
/// According to research and the correlation math we did
/// this relates in the following way to percentage of GDP spent
/// on healthcare:
/// ```math
/// 5 * log(x) + 68.5
/// ```
#[derive(Component, Default, Debug)]
pub struct SanitationInfrastructure {
    pub health_index_score: f32,
    pub live_birth_mortality_rate: f32,
    pub global_hunger_index: f32,
    pub actual_infant_death_ratio: f32,
}

impl SanitationInfrastructure {
    pub fn update(&mut self, spending: f32) {
        self.health_index_score_fn(spending);
        self.live_birth_mortality_rate_fn(spending);
    }
    fn health_index_score_fn(&mut self, spending: f32) {
        self.health_index_score = corr_ln(5.00186, 68.60778, spending).clamp(0., 1.);
    }
    fn live_birth_mortality_rate_fn(&mut self, spending: f32) {
        self.live_birth_mortality_rate = corr_ln(-0.00260, 0.00923, spending + 0.01).clamp(0., 1.);
    }
    pub fn actual_infant_mortality_rate_fn(
        &mut self,
        yearly_infant_deaths: usize,
        yearly_infant_births: usize,
    ) {
        if yearly_infant_births == 0 {
            self.actual_infant_death_ratio = 0.0;
            return;
        }
        self.actual_infant_death_ratio =
            (yearly_infant_deaths / yearly_infant_births) as f32 * 1000.0;
    }
    pub fn calc_global_hunger_index_fn(
        &mut self,
        undernourishment: f32,
        child_stunting: f32,
        child_wasting: f32,
    ) {
        self.global_hunger_index = (undernourishment / 80.0) * 100.0
            + (child_stunting / 70.0) * 100.0
            + (child_wasting / 30.0) * 100.0
            + (self.actual_infant_death_ratio / 35.0) * 100.0;
        //  FIXME: Why is this needed? shouldtnt this be 0 by default?
        if self.global_hunger_index.is_nan() {
            self.global_hunger_index = 0.0;
        }
    }
}

/// Explains itself
#[derive(Component, Default)]
pub struct SocialInfrastructure {}

/// Contains things that are directly affected by humans like:
/// - indoor air pollution
/// - drinking water
/// - urban particulates
#[derive(Component, Default)]
pub struct EnvironmentalHealth {
    /// affected by density of human spacing
    pub indoor_air_pollution: f32,
    /// affected by spending
    pub drinking_water: f32,
    /// affected by the human space in relation to total space
    pub urban_particulates: f32,
}

impl EnvironmentalHealth {
    pub fn update(
        &mut self, spending: f32, space_per_human: f32, unused_space: f32,
        human_space: f32, humans: usize
    ) {
        self.indoor_air_pollution_fn(space_per_human);
        self.drinking_water_fn(spending);
        self.urban_particulates_fn(unused_space, human_space, humans);
    }
    pub fn total_avg_val(&self) -> f32 {
        (self.indoor_air_pollution + self.drinking_water + self.urban_particulates) / 3.
    }
    fn indoor_air_pollution_fn(
        &mut self, space_per_human: f32
    ) {
        self.indoor_air_pollution = corr_ln(
            0.38, -0.6, space_per_human * 10_000.
        ).clamp(0., 1.);
    }
    fn drinking_water_fn(&mut self, spending: f32) {
        self.drinking_water = corr_ln(0.17, 0.1, spending).clamp(0., 1.);
    }
    fn urban_particulates_fn(
        &mut self, unused_space: f32, human_space: f32, humans: usize
    ) {
        self.urban_particulates = corr_ln(
            0.35, -1., (unused_space + human_space) * 10_000. / humans as f32
        ).clamp(0., 1.);
    }
    
}

/// Things that are indirectly affected by humans only when the expand to
/// - air quality
/// - water resources
/// - productive natural resources
/// - biodiversity
/// - sustainable energy
#[derive(Component, Default)]
pub struct EcosystemVitality {
    /// affected by spending on the ecosystem
    pub air_quality_index: f32,
    /// affected by ratio of unused land to total land
    pub productive_natural_resources: f32,
    /// affected by ratio of farm land to natural land
    pub biodiversity: f32,
}

impl EcosystemVitality {
    pub fn update(
        &mut self, spending: f32,
        unused_land: f32, farm_land: f32, human_land: f32,
    ) {
        self.air_quality_index_fn(spending);
        self.productive_natural_resources_fn(
            unused_land, unused_land + farm_land + human_land
        );
        self.biodiversity_fn(unused_land, farm_land);
    }
    pub fn total_avg_val(&self) -> f32 {
        (self.air_quality_index + self.productive_natural_resources + self.biodiversity) / 3.
    }
    fn air_quality_index_fn(&mut self, spending: f32) {
        self.air_quality_index = corr_ln(0.17, 0.1, spending).clamp(0., 1.);
    }
    fn productive_natural_resources_fn(&mut self, unused_land: f32, total_land: f32) {
        self.productive_natural_resources = (unused_land / total_land).clamp(0., 1.);
    }
    fn biodiversity_fn(&mut self, unused_land: f32, farm_land: f32) {
        self.biodiversity = (unused_land / farm_land).clamp(0., 1.);
    }

}

fn corr_ln(a: f32, b: f32, x: f32) -> f32 {
    a * x.ln() + b
}

pub fn weighted_range(start: f32, end: f32, weight: f32) -> f32 {
    (end - start) * weight + start
}
