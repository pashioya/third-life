use bevy::prelude::*;
use crate::{
    time::{DateChanged, GameDate},
    SimulationState,
};

use super::{Citizen, Starving, StillGrowing, NEW_BORN_HEIGHT, NEW_BORN_WEIGHT};

pub struct GrowingPlugin;

impl Plugin for GrowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (grow_citizens, stop_growing).run_if(in_state(SimulationState::Running)),
        );
    }
}
const STOP_GROWING_AGE: usize = 25;
const GROWING_DAYS: f32 = 9125.0;

fn grow_citizens(
    mut date_change_event: EventReader<DateChanged>,
    mut citizens: Query<&mut Citizen, (With<StillGrowing>, Without<Starving>)>,
) {
     let days = date_change_event.read().collect::<Vec<_>>();
     if let Some(_) = days.last() {
        let len_days = days.len();
        for mut citizen in citizens.iter_mut() {
            let days_growth =( (citizen.genetic_height - NEW_BORN_HEIGHT)/ GROWING_DAYS) * len_days as f32;
            let days_fattening  = ((citizen.genetic_weight - NEW_BORN_WEIGHT) / GROWING_DAYS) * len_days as f32;
            citizen.height += days_growth;
            citizen.weight += days_fattening;
        }    
    } 
}

fn stop_growing(
    game_date: Res<GameDate>,
    mut commands: Commands,
    citizens: Query<(Entity, &Citizen), With<StillGrowing>>,
) {
    for (entity, citizen) in citizens.iter() {
        let age = game_date.years_since(citizen.birthday).unwrap_or(0);
        if age as usize > STOP_GROWING_AGE {
            commands.get_entity(entity).map(|mut e| {
                e.remove::<StillGrowing>();
            });
        }
    }
}
