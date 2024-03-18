use bevy::prelude::*;
use chrono::NaiveDate;

use crate::worlds::food::components::FedStatus;

use super::CitizenDied;

use super::super::config;

/// In cm
pub static NEW_BORN_HEIGHT: f32 = 50.;
/// In kg
pub static NEW_BORN_WEIGHT: f32 = 3.3;

#[derive(Component, Default)]
pub struct Population {
    pub count: usize,
    pub working_pop: usize,
    pub younglings: usize,
    pub retirees: usize,
    pub average_age: usize,
    pub average_children_per_mother: f32,
}

#[derive(Component, Default)]
pub struct DietMacroRatios {
    protein: usize,
    fat: usize,
    carbs: usize
}

impl DietMacroRatios {
    pub fn from_config(config: config::DietMacroRatiosConfig) -> Result<Self, ()> {
        let protein = config.protein();
        let fat = config.fat();
        let carbs = config.carbs();
        match (protein + fat + carbs) == 100 {
            true => Ok(Self { protein, fat, carbs }),
            false => Err(())
        }
    }
    pub fn new(protein: usize, fat: usize, carbs: usize) -> Result<Self, ()> {
        match (protein + fat + carbs) == 100 {
            true => Ok(Self { protein, fat, carbs }),
            false => Err(())
        }
    }
    pub fn protein(&self) -> f32 { self.protein as f32 / 100. }
    pub fn fat(&self) -> f32 { self.fat as f32 / 100. }
    pub fn carbs(&self) -> f32 { self.carbs as f32 / 100. }
}

#[derive(Bundle)]
pub struct MaleCitizenBundle {
    citizen: Citizen,
    of: CitizenOf,
    male: Male,
    growing: StillGrowing,
    youngling: Youngling,
}

impl MaleCitizenBundle {
    pub fn new(
        name: String, colony: Entity, birthday: NaiveDate,
    ) -> Self {
        Self { 
            citizen: Citizen { name, birthday, height: NEW_BORN_HEIGHT, weight: NEW_BORN_WEIGHT }, 
            of: CitizenOf { colony }, male: Male, growing: StillGrowing,
            youngling: Youngling
        }
    }
}

#[derive(Bundle)]
pub struct FemaleCitizenBundle {
    citizen: Citizen,
    of: CitizenOf,
    female: Female,
    growing: StillGrowing,
    youngling: Youngling,
}

impl FemaleCitizenBundle {
    pub fn new(
        name: String, colony: Entity, birthday: NaiveDate,
    ) -> Self {
        Self { 
            citizen: Citizen { name, birthday, height: NEW_BORN_HEIGHT, weight: NEW_BORN_WEIGHT }, 
            of: CitizenOf { colony }, female: Female { children_had: 0 }, 
            growing: StillGrowing, youngling: Youngling
        }
    }
}

#[derive(Component, PartialEq, Clone)]
pub struct Citizen {
    pub name: String,
    pub birthday: NaiveDate,
    pub height: f32,
    pub weight: f32,
}

#[derive(Component)]
pub struct CitizenOf {
    pub colony: Entity,
}

#[derive(Eq, PartialEq)]
pub enum StarvingStatus {
    StoppedStarving,
    ReducedStarving,
    StillStarving,
    IncreasedStarving,
    Died
}

#[derive(Component)]
pub struct Starving {
    pub days_since_last_meal: usize
}

impl Starving {
    const LIMIT_DAYS_STARVATION: usize = 21;
    pub fn started() -> Self { Self { days_since_last_meal: 1 } }
    pub fn died(&self) -> bool { self.days_since_last_meal > Self::LIMIT_DAYS_STARVATION }
    pub fn feed_many(&mut self, statuses: &Vec<&FedStatus>) -> StarvingStatus {
        let mut final_status = StarvingStatus::StillStarving;
        for status in statuses {
            final_status = self.feed(status);
            if final_status == StarvingStatus::Died {
                return final_status;
            }
        }
        final_status
    }
    pub fn feed(&mut self, status: &FedStatus) -> StarvingStatus {
        match status {
            FedStatus::FedCorrectDiet { .. } | FedStatus::FedInccorectDiet { .. } => self.feed_once(),
            FedStatus::InsuficcentlyFed { .. } => StarvingStatus::StillStarving,
            FedStatus::NotFed => self.starve_once()
        }
    }
    fn feed_once(&mut self) -> StarvingStatus {
        match self.days_since_last_meal {
            0 => StarvingStatus::StoppedStarving,
            1 => {
                self.days_since_last_meal -= 1;
                StarvingStatus::StoppedStarving
            },
            2..=Self::LIMIT_DAYS_STARVATION => {
                self.days_since_last_meal -= 1;
                StarvingStatus::ReducedStarving
            },
            _ => StarvingStatus::Died
        }
    }
    fn starve_once(&mut self) -> StarvingStatus {
        match self.days_since_last_meal {
            0..=Self::LIMIT_DAYS_STARVATION => {
                self.days_since_last_meal += 1;
                StarvingStatus::IncreasedStarving
            },
            _ => StarvingStatus::Died
        }
    }
}

#[derive(Component)]
pub struct Female {
    pub children_had: usize
}

#[derive(Component)]
pub struct Male;

#[derive(Component)]
pub struct Ovulation {
    pub ovulation_start_date: NaiveDate,
}

#[derive(Component)]
pub struct Pregnancy {
    pub baby_due_date: NaiveDate,
}

#[derive(Component)]
pub struct Spouse {
    pub spouse: Entity,
}

#[derive(Component)]
pub struct Widowed;

#[derive(Component)]
pub struct StillGrowing;

#[derive(Component)]
pub struct Employed;

#[derive(Component)]
pub struct Employable;


#[derive(Component)]
pub struct Youngling;

#[derive(Component)]
pub struct Retiree;

/// Total Food Resources Consumed per year/season used to track if we need to spawn a new farm
#[derive(Component, Default)]
pub struct CarbConsumed {
    pub amount: f32,
}

#[derive(Component, Default)]
pub struct MeatConsumed {
    pub amount: f32,
}
