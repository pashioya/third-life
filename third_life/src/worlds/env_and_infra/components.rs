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
#[derive(Component)]
pub struct CivilInfrastructure {
    pub urbanization_index: f32,
    // number between 0 and 1 to define how mechanized farming is
    pub farming_mechanization: f32,
}

impl Default for CivilInfrastructure {
    fn default() -> Self {
        Self {
            urbanization_index: 0.,
            farming_mechanization: 0.5,
        }
    }
}

impl CivilInfrastructure {
    pub fn update(&mut self, work_force: usize, farmers_count: usize) {
        self.urbanization_index_fn(work_force, farmers_count);
    }
    fn urbanization_index_fn(&mut self, work_force: usize, farmers_count: usize) {
        self.urbanization_index = 1.0 - (farmers_count as f32 / work_force as f32);
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
        self.health_index_score = corr_ln(5.00186, 68.60778, spending);
    }
    fn live_birth_mortality_rate_fn(&mut self, spending: f32) {
        self.live_birth_mortality_rate = corr_ln(-0.00260, 0.00923, spending);
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
    pollution_intensity_index: f32,
}

impl EnvironmentalHealth {
    pub fn update(&mut self, spending: f32) {
        self.pollution_intensity_index_fn(spending);
    }
    fn pollution_intensity_index_fn(&mut self, spending: f32) {
        // FIXME: Find a proper correlation formula
        self.pollution_intensity_index = corr_ln(0.00001, 0.00001, spending);
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
    air_quality_index: f32,
}

impl EcosystemVitality {
    pub fn update(&mut self, spending: f32) {
        self.air_quality_index_fn(spending);
    }
    fn air_quality_index_fn(&mut self, spending: f32) {
        // FIXME: Find a proper correlation formula
        self.air_quality_index = corr_ln(0.00001, 0.00001, spending);
    }
}

fn corr_ln(a: f32, b: f32, x: f32) -> f32 {
    a * x.ln() + b
}

pub fn weighted_range(start: f32, end: f32, weight: f32) -> f32 {
    (end - start) * weight + start
}
