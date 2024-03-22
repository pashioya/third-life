use bevy::prelude::*;

#[derive(Event)]
pub struct CitizenCreated {
    pub age: usize,
    pub colony: Entity,
    pub mother_age: Option<usize>,
}

#[derive(Hash, PartialEq, Eq)]
pub enum DeathReason {
    OldAge,
    Starvation,
    InfantDeath,
}

impl DeathReason {
    pub fn to_lowercase(&self) -> String {
        String::from(match self {
            Self::OldAge => "old_age",
            Self::Starvation => "starvation",
            Self::InfantDeath => "infant_death"
        })
    }
}

#[derive(Event)]
pub struct CitizenDied {
    pub colony: Entity,
    pub citizen: Entity,
    pub reason: DeathReason,
}

impl CitizenDied {
    pub fn old_age(colony: Entity, citizen: Entity) -> Self {
        Self {
            colony,
            citizen,
            reason: DeathReason::OldAge,
        }
    }
    pub fn starved(colony: Entity, citizen: Entity) -> Self {
        Self {
            colony,
            citizen,
            reason: DeathReason::Starvation,
        }
    }
    pub fn infant_death(colony: Entity, citizen: Entity) -> Self {
        Self {
            colony,
            citizen,
            reason: DeathReason::InfantDeath,
        }
    }
}

#[derive(Event)]
pub struct CitizenBirthday {
    pub entity: Entity,
    pub colony: Entity,
    pub age: usize,
}
