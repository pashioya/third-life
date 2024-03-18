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
/// Calculate the urbanization index using the folowing formula:
/// Urbanization Ratio = 1 - (Number of Farmers / Total Workforce)
#[derive(Component, Default)]
pub struct CivilInfrastructure {
    // ? Should this be in the civil infrastructure or in the treasury?
    pub urbanization_index: f32,
}

impl CivilInfrastructure {
    pub fn update(&mut self, work_force: usize, farmers_count: usize) {
        self.urbanization_index_fn(work_force, farmers_count);
    }
    fn urbanization_index_fn(&mut self, work_force: usize, farmers_count: usize) {
        self.urbanization_index = 1.0 - (farmers_count as f32 / work_force as f32);
    }
}

/// Anthing that has to do with sanitation like hospitals but also
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
