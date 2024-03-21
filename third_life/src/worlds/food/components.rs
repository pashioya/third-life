use bevy::prelude::*;
use chrono::NaiveDate;

pub fn from_cals_to_kgs(min: f32, max: f32, quality: f32, cals: f32) -> f32 {
    cals / ((max - min) * quality + min)
}
pub fn from_kgs_to_cals(min: f32, max: f32, quality: f32, kgs: f32) -> f32 {
    ((max - min) * quality + min) * kgs
}


#[derive(Component)]
pub struct Employed;

/// Describes if a citizen or a number of citizens have been fed and if the ratio
/// they have been fed is according to their diet
///
/// Internally stores the numbers as kgs of carbs and meat that has been consumed
#[derive(PartialEq)]
pub enum FedStatus {
    FedCorrectDiet { carbs: f32, meat: f32 },
    FedInccorectDiet { carbs: f32, meat: f32 },
    InsuficcentlyFed { carbs: f32, meat: f32 },
    NotFed
}

impl FedStatus {
    pub fn correct_diet_cals(
        carbs: f32, carb_quality: f32, meat: f32, meat_quality: f32
    ) -> Self {
        Self::FedCorrectDiet { 
            carbs: Self::carb_cal_to_kg(carb_quality, carbs),
            meat: Self::meat_cal_to_kg(meat_quality, meat)
        }
    }
    pub fn incorrect_diet_cals(
        carbs: f32, carb_quality: f32, meat: f32, meat_quality: f32
    ) -> Self {
        Self::FedInccorectDiet { 
            carbs: Self::carb_cal_to_kg(carb_quality, carbs),
            meat: Self::meat_cal_to_kg(meat_quality, meat)
        }
    }
    pub fn insufficent_cals(
        carbs: f32, carb_quality: f32, meat: f32, meat_quality: f32
    ) -> Self {
        Self::FedInccorectDiet { 
            carbs: Self::carb_cal_to_kg(carb_quality, carbs),
            meat: Self::meat_cal_to_kg(meat_quality, meat) 
        }
    }
    fn carb_cal_to_kg(qual: f32, carb: f32) -> f32 {
        from_cals_to_kgs(CarbResource::MIN_CAL, CarbResource::MAX_CAL, qual, carb)
    }
    fn meat_cal_to_kg(qual: f32, meat: f32) -> f32 {
        from_cals_to_kgs(MeatResource::MIN_CAL, MeatResource::MAX_CAL, qual, meat)
    }
    pub fn correct_diet_kgs(carbs: f32, meat: f32) -> Self {
        Self::FedCorrectDiet { carbs, meat }
    }
    pub fn incorrect_diet_kgs(carbs: f32, meat: f32) -> Self {
        Self::FedInccorectDiet { carbs, meat }
    }
    pub fn insufficent_kgs(carbs: f32, meat: f32) -> Self {
        Self::InsuficcentlyFed { carbs, meat }
    }
    pub fn not_fed() -> Self { Self::NotFed }
    pub fn is_ok(&self) -> bool {
        match self {
            Self::FedCorrectDiet{ .. } | Self::FedInccorectDiet{ .. } => true,
            Self::InsuficcentlyFed { .. } | Self::NotFed => false
        }
    }
}


#[derive(Component, PartialEq, Eq, Hash)]
pub struct ResourceOf {
    pub colony: Entity,
}

#[derive(Component, Default)]
pub struct CarbResource {
    pub quality: f32,
    amount: f32,
}

impl CarbResource {
    /// minimum caloris contained in 1kg
    pub const MIN_CAL: f32 = 3000.;
    /// maximum caloris contained in 1kg
    pub const MAX_CAL: f32 = 3500.;

    pub const CARBS: f32 = 0.71;
    pub const PROTEIN: f32 = 0.15;
    pub const FATS: f32 = 0.;
    pub const FIBER: f32 = 0.1;

    pub fn from_amount(amount: f32) -> Self {
        Self { quality: 0., amount }
    }

    /// Takes in calories but returnes the consumed amount of kgs
    fn consume_calories_unchecked(&mut self, calories: f32) -> f32 {
        let to_consume = from_cals_to_kgs(Self::MIN_CAL, Self::MAX_CAL, self.quality, calories);
        self.amount -= to_consume;
        calories
    }

    pub fn total_calories(&self) -> f32 {
        from_kgs_to_cals(Self::MIN_CAL, Self::MAX_CAL, self.quality, self.amount)
    }
    
    pub fn has_sufficent_calories(&self, calories: f32) -> bool {
        self.total_calories() >= calories
    }
    pub fn consume_calories(&mut self, calories: f32) -> Result<f32, ()> {
        if self.has_sufficent_calories(calories) {
            Ok(self.consume_calories_unchecked(calories))
        } else {
            Err(())
        }
    }
    pub fn consume_all(&mut self) -> f32 {
        let ret = self.amount;
        self.amount = 0.;
        ret
    }
    pub fn add_kgs(&mut self, kgs: f32) { self.amount += kgs; }
    pub fn get_kgs(&self) -> f32 { self.amount }
}

#[derive(Component)]
pub struct WheatFarm {
    pub size: f32,
    pub harvested: f32,
}

impl WheatFarm {
    pub fn remaining_for_harvest(&self) -> f32 {
        self.size - self.harvested
    }
}

#[derive(Component)]
pub struct WheatFarmOf {
    pub colony: Entity,
}

#[derive(Component)]
pub struct WheatFarmer {
    pub farm: Entity,
}

#[derive(Component)]
pub struct Cow {
    pub birthday: NaiveDate
}

#[derive(Component)]
pub struct CowOf {
    pub cow_farm: Entity,
}

#[derive(Component)]
pub struct IsBull;

#[derive(Component)]
pub struct IsBreeder;

#[derive(Component, Default)]
pub struct MeatResource {
    pub quality: f32,
    amount: f32,
}

impl MeatResource {
    /// minimum caloris contained in 1kg
    pub const MIN_CAL: f32 = 2000.;
    /// maximum caloris contained in 1kg
    pub const MAX_CAL: f32 = 3000.;

    pub const CARBS: f32 = 0.;
    pub const PROTEIN: f32 = 0.26;
    pub const FATS: f32 = 0.12;
    pub const FIBER: f32 = 0.;

    pub fn from_amount(amount: f32) -> Self {
        Self { quality: 0., amount }
    }

    /// Takes in calories but returnes the consumed amount of kgs
    fn consume_calories_unchecked(&mut self, calories: f32) -> f32 {
        let to_consume = from_cals_to_kgs(Self::MIN_CAL, Self::MAX_CAL, self.quality, calories);
        self.amount -= to_consume;
        calories
    }

    pub fn total_calories(&self) -> f32 {
        from_kgs_to_cals(Self::MIN_CAL, Self::MAX_CAL, self.quality, self.amount)
    }
    
    pub fn has_sufficent_calories(&self, calories: f32) -> bool {
        self.total_calories() >= calories
    }
    pub fn consume_calories(&mut self, calories: f32) -> Result<f32, ()> {
        if self.has_sufficent_calories(calories) {
            Ok(self.consume_calories_unchecked(calories))
        } else {
            Err(())
        }
    }
    pub fn consume_all(&mut self) -> f32 { 
        let ret = self.amount;
        self.amount = 0.;
        ret
    }
    pub fn add_kgs(&mut self, kgs: f32) { self.amount += kgs; }
    pub fn get_kgs(&self) -> f32 { self.amount }
}

#[derive(Component)]
pub struct CowFarm {
    pub size: f32,
    pub farmers_wanted: usize,
    pub hours_worked: f32
}

#[derive(Component)]
pub struct CowFarmOf {
    pub colony: Entity,
}

#[derive(Component)]
pub struct CowFarmer {
    pub farm: Entity,
}
