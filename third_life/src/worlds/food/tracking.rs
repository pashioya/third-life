use bevy::{prelude::*};

use crate::{worlds::{config::WorldConfig, init_colonies, WorldColony}, SimulationState};

use super::{CarbCreated, MeatCreated};

pub struct FoodProductionTrackingPlugin;
impl Plugin for FoodProductionTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            init_tracking.after(init_colonies),
        )
        .add_systems(Update, (update_carb_produced, update_meat_produced));
    }
}

/// Total Food Resource produced per year/season used to track if we need to spawn a new farm
#[derive(Component, Default)]
pub struct CarbProduced {
    pub amount: f32
}

#[derive(Component, Default)]
pub struct MeatProduced {
    pub amount: f32
}

fn init_tracking (
    mut commands: Commands,
    colonies: Query<Entity, With<WorldColony>>
) {
    for colony_entity in colonies.iter() {
        commands.get_entity(colony_entity).map(|mut c| { 
            c.try_insert((
                CarbProduced::default(),
                MeatProduced::default()
            ));
        });
    }
}

fn update_carb_produced (
    mut carbs_created: EventReader<CarbCreated>, 
    mut colonies: Query<(Entity, &mut CarbProduced), With<WorldColony>>,
) {
    for carb_created in carbs_created.read() {
        colonies.get_mut(carb_created.colony).unwrap().1.amount += carb_created.amount;
    }
}

fn update_meat_produced (
    mut meats_created: EventReader<MeatCreated>, 
    mut colonies: Query<(Entity, &mut MeatProduced), With<WorldColony>>,
) {
    for meat_created in meats_created.read() {
        colonies.get_mut(meat_created.colony).unwrap().1.amount += meat_created.amount;
    }
}
