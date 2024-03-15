

use std::collections::HashMap;

use bevy::prelude::*;

use crate::{time::GameDate, worlds::config::WorldConfig, SimulationState};

use super::{Citizen, CitizenOf, StillGrowing, NEW_BORN_HEIGHT, NEW_BORN_WEIGHT};

pub struct GrowingPlugin;

impl Plugin for GrowingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                grow_citizens,
                stop_growing,
            ).run_if(in_state(SimulationState::Running)));
    }
}

const STOP_GROWING_AGE: usize = 25;

fn grow_citizens(
    game_date: Res<GameDate>,
    worlds: Query<(Entity, &WorldConfig)>,
    mut citizens: Query<(&mut Citizen, &CitizenOf), With<StillGrowing>>
) {
    let worlds = worlds.iter().map(|w|
        (w.0, (w.1.population().height_dist().average(), w.1.population().weight_dist().average()))
    ).collect::<HashMap<_, _>>();
    for (mut citizen, CitizenOf { colony }) in citizens.iter_mut() {
        let age_normalized = (game_date.years_since(citizen.birthday).unwrap() as usize / STOP_GROWING_AGE) as f32;
        let (height_avg, weight_avg) = worlds.get(colony).unwrap();
        citizen.height = *height_avg * age_normalized - NEW_BORN_HEIGHT;
        citizen.weight = *weight_avg * age_normalized - NEW_BORN_WEIGHT;
    }
}


fn stop_growing(
    game_date: Res<GameDate>,
    mut commands: Commands, 
    citizens: Query<(Entity, &Citizen), With<StillGrowing>>,
) {
    for (entity, citizen) in citizens.iter() {
        let age = game_date.years_since(citizen.birthday).unwrap();
        if age as usize > STOP_GROWING_AGE {
            commands.get_entity(entity).map(|mut e| {
                e.remove::<StillGrowing>();
            });
        }
    }
}
