
use bevy::prelude::*;
use chrono::NaiveDate;
use rand::Rng;
use rand_distr::{Distribution, Normal};


use crate::{common::utils::roll_chance, worlds::{config::PopulationConfig, food::components::FedStatus}};

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
    pub average_height: f32,
    pub average_weight: f32,
    pub average_children_per_mother: f32,
    pub yearly_infant_deaths: usize,
    pub yearly_infant_births: usize,
    pub num_couples: usize,
    pub males: usize,
    pub females: usize,
    pub working_in_wheat: usize,
    pub working_in_beef: usize,
    pub working_not_in_farming: usize,
}

#[derive(Component, Default)]
pub struct DietMacroRatios {
    protein: usize,
    fat: usize,
    carbs: usize,
}

impl DietMacroRatios {
    pub fn from_config(config: config::DietMacroRatiosConfig) -> Result<Self, ()> {
        let protein = config.protein();
        let fat = config.fat();
        let carbs = config.carbs();
        match (protein + fat + carbs) == 100 {
            true => Ok(Self {
                protein,
                fat,
                carbs,
            }),
            false => Err(()),
        }
    }
    pub fn new(protein: usize, fat: usize, carbs: usize) -> Result<Self, ()> {
        match (protein + fat + carbs) == 100 {
            true => Ok(Self {
                protein,
                fat,
                carbs,
            }),
            false => Err(()),
        }
    }
    pub fn protein(&self) -> f32 {
        self.protein as f32 / 100.
    }
    pub fn fat(&self) -> f32 {
        self.fat as f32 / 100.
    }
    pub fn carbs(&self) -> f32 {
        self.carbs as f32 / 100.
    }
}

pub struct CitizenBuilder {
    age: Option<usize>,
    bundle: Option<CitizenBundle>,
    youngling: Option<Youngling>,
    growing: Option<StillGrowing>,
    retiree: Option<Retiree>,
    male: Option<Male>,
    female: Option<Female>,
}

impl CitizenBuilder {
    pub fn start(
        game_date: NaiveDate, colony: Entity, birthday: NaiveDate,
        pop_config: PopulationConfig
    ) -> Self {
        let age = game_date.years_since(birthday).unwrap() as usize;
        let mut builder = Self { 
            bundle: Some(CitizenBundle::new(age, colony, birthday, pop_config.clone())), 
            age: Some(age), youngling: None, growing: None, retiree: None,
            male: None, female: None,
        };
        if age < 25 {
            builder.growing();

        }
        if age < pop_config.age_of_adult() {
            builder.youngling();
        }
        if age > pop_config.age_of_retirement() {
            builder.retiree();
        }

        builder
    }
    fn growing(&mut self) { self.growing = Some(StillGrowing); }
    fn youngling(&mut self) { self.youngling = Some(Youngling); }
    fn retiree(&mut self) { self.retiree = Some(Retiree); }
    pub fn male(mut self) -> Self {
        self.male = Some(Male);
        self.female = None;
        self
    }
    pub fn female(mut self) -> Self {
        let children_had = if self.age.unwrap() > 18 {
            rand::thread_rng().gen_range(0..5)
        } else { 0 };
        self.female = Some(Female::grown(children_had, None));
        self
    }
    
    pub fn build(self) -> Result<CreatedCitizen, ()> {
        Ok(match (
            self.bundle, self.youngling, self.growing, self.retiree,
            self.male, self.female
        ) {
            (None, _, _, _, _, _) => Err(())?,
            (Some(bundle), Some(youngling), Some(growing), None, Some(male), None) => 
                CreatedCitizen::NewbornMaleCitizen((bundle, male, growing, youngling)),
            (Some(bundle), Some(youngling), Some(growing), None, None, Some(female)) =>
                CreatedCitizen::NewbornFemaleCitizen((bundle, female, growing, youngling)),
            (Some(bundle), None, Some(growing), None, Some(male), None) => 
                CreatedCitizen::AdultGrowingMaleCitizen((bundle, male, growing)),
            (Some(bundle), None, Some(growing), None, None, Some(female)) => 
                CreatedCitizen::AdultGrowingFemaleCitizen((bundle, female, growing)),
            (Some(bundle), None, None, None, Some(male), None) =>
                CreatedCitizen::GrownMaleCitizen((bundle, male)),
            (Some(bundle), None, None, None, None, Some(female)) =>
                CreatedCitizen::GrownFemaleCitizen((bundle, female)),
            (Some(bundle), None, None, Some(retiree), Some(male), None) => 
                CreatedCitizen::RetireeMaleCitizen((bundle, male, retiree)),
            (Some(bundle), None, None, Some(retiree), None, Some(female)) =>
                CreatedCitizen::RetireeFemaleCitizen((bundle, female, retiree)),
            _ => Err(())?
        })
    }
}

pub enum CreatedCitizen {
    RetireeMaleCitizen((CitizenBundle, Male, Retiree)),
    RetireeFemaleCitizen((CitizenBundle, Female, Retiree)),
    GrownMaleCitizen((CitizenBundle, Male)),
    GrownFemaleCitizen((CitizenBundle, Female)),
    AdultGrowingMaleCitizen((CitizenBundle, Male, StillGrowing)),
    AdultGrowingFemaleCitizen((CitizenBundle, Female, StillGrowing)),
    NewbornMaleCitizen((CitizenBundle, Male, StillGrowing, Youngling)),
    NewbornFemaleCitizen((CitizenBundle, Female, StillGrowing, Youngling)),

}

impl CreatedCitizen {
    pub fn spawn(self, commands: &mut Commands) {
        match self {
            CreatedCitizen::RetireeMaleCitizen(t) => commands.spawn(t),
            CreatedCitizen::RetireeFemaleCitizen(t) => commands.spawn(t), 
            CreatedCitizen::GrownMaleCitizen(t) => commands.spawn(t), 
            CreatedCitizen::GrownFemaleCitizen(t) => commands.spawn(t), 
            CreatedCitizen::AdultGrowingMaleCitizen(t) => commands.spawn(t), 
            CreatedCitizen::AdultGrowingFemaleCitizen(t) => commands.spawn(t), 
            CreatedCitizen::NewbornMaleCitizen(t) => commands.spawn(t), 
            CreatedCitizen::NewbornFemaleCitizen(t) => commands.spawn(t),
        };
    }
}

pub fn create_citizen(
    game_date: NaiveDate, colony: Entity, birthday: NaiveDate,
    pop_config: PopulationConfig
) -> CreatedCitizen {
    let mut builder = CitizenBuilder::start(game_date, colony, birthday, pop_config);
    
    builder = match roll_chance(50) {
        true => builder.male(),
        false => builder.female(),
    };
    builder.build().unwrap()
}

#[derive(Bundle)]
pub struct CitizenBundle {
    citizen: Citizen,
    of: CitizenOf,
}

impl CitizenBundle {
    fn new(
        age: usize, colony: Entity, birthday: NaiveDate, 
        pop_config: PopulationConfig
    ) -> Self {
        let genetic_height = Normal::new(pop_config.height_dist().average(), 7.0).unwrap().sample(&mut rand::thread_rng());
        let genetic_weight = Normal::new(pop_config.weight_dist().average(), 10.0).unwrap().sample(&mut rand::thread_rng());

        // amount of growth per day
        let daily_growth = (genetic_height - NEW_BORN_HEIGHT)/ 9125.0;
        let daily_fattening = (genetic_weight - NEW_BORN_WEIGHT) / 9125.0;

        let height = (age * 365) as f32 * daily_growth + NEW_BORN_HEIGHT;
        let weight = (age * 365) as f32 * daily_fattening + NEW_BORN_WEIGHT;
        Self { 
            citizen: Citizen { 
                name: String::from("Name"), birthday, genetic_height,
                genetic_weight, height, weight, daily_growth, daily_fattening
            },
            of: CitizenOf { colony }

        }
    }
}

#[derive(Component, PartialEq, Clone)]
pub struct Citizen {
    pub name: String,
    pub birthday: NaiveDate,
    pub genetic_height: f32,
    pub genetic_weight: f32,
    pub height: f32,
    pub weight: f32,
    pub daily_growth: f32,
    pub daily_fattening: f32,
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
    Died,
}

#[derive(Component)]
pub struct Starving {
    pub days_since_last_meal: usize,
}

impl Starving {
    const LIMIT_DAYS_STARVATION: usize = 21;
    pub fn started() -> Self {
        Self {
            days_since_last_meal: 1,
        }
    }
    pub fn died(&self) -> bool {
        self.days_since_last_meal > Self::LIMIT_DAYS_STARVATION
    }
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
            FedStatus::FedCorrectDiet { .. } | FedStatus::FedInccorectDiet { .. } => {
                self.feed_once()
            }
            FedStatus::InsuficcentlyFed { .. } => StarvingStatus::StillStarving,
            FedStatus::NotFed => self.starve_once(),
        }
    }
    fn feed_once(&mut self) -> StarvingStatus {
        match self.days_since_last_meal {
            0 => StarvingStatus::StoppedStarving,
            1 => {
                self.days_since_last_meal -= 1;
                StarvingStatus::StoppedStarving
            }
            2..=Self::LIMIT_DAYS_STARVATION => {
                self.days_since_last_meal -= 1;
                StarvingStatus::ReducedStarving
            }
            _ => StarvingStatus::Died,
        }
    }
    fn starve_once(&mut self) -> StarvingStatus {
        match self.days_since_last_meal {
            0..=Self::LIMIT_DAYS_STARVATION => {
                self.days_since_last_meal += 1;
                StarvingStatus::IncreasedStarving
            }
            _ => StarvingStatus::Died,
        }
    }
}

#[derive(Component)]
pub struct Female {
    pub children_had: usize,
    pub last_child_birth_date: Option<NaiveDate>,
}

impl Female {
    pub fn grown(
        children_had: usize, last_child_birth_date: Option<NaiveDate>
    ) -> Self {
        Self { children_had, last_child_birth_date }
    }
    pub fn newborn() -> Self {
        Self { children_had: 0, last_child_birth_date: None }
    }
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
