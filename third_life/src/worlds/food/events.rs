use bevy::prelude::*;

#[derive(Event)]
pub struct WheatFarmNeedsWorker {
    pub colony: Entity,
    pub farm: Entity,
}

#[derive(Event)]
pub struct CowFarmNeedsWorker {
    pub colony: Entity,
    pub farm: Entity,
}

#[derive(Event)]
pub struct MeatCreated {
    pub colony: Entity,
    pub amount: f32
}

#[derive(Event)]
pub struct MeatConsumedEvent {
    pub colony: Entity,
    pub amount: f32
}

#[derive(Event)]
pub struct CarbCreated {
    pub colony: Entity,
    pub amount: f32
}

#[derive(Event)]
pub struct CarbConsumedEvent {
    pub colony: Entity,
    pub amount: f32
}

#[derive(Event)]
pub struct FoodCreated {
    pub colony: Entity,
    pub amount: f32
}
#[derive(Event)]
pub struct CowFarmCreated {
    pub colony: Entity
}
#[derive(Event)]
pub struct WheatFarmCreated {
    pub colony: Entity
}
#[derive(Event)]
pub struct CowFarmRemoved {
    pub colony: Entity
}
#[derive(Event)]
pub struct WheatFarmRemoved {
    pub colony: Entity
}
