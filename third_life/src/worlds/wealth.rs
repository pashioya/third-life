pub mod components;

use std::collections::HashMap;

use bevy::prelude::*;

use crate::SimulationState;

use self::components::Treasury;

use super::population::components::{CitizenOf, Employed, Population};

pub struct WealthPlugin;

impl Plugin for WealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (calculate_wealth).run_if(in_state(SimulationState::Running)),
        );
    }
}

fn calculate_wealth(
    working_citizens: Query<&CitizenOf, Without<Employed>>,
    mut colonies: Query<(Entity, &mut Treasury, &Population)>,
) {
    let citizens_per_colony =
        working_citizens
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<_, usize>, c_of| {
                *acc.entry(c_of.colony).or_insert(0) += 1;
                acc
            });
    colonies.iter_mut().for_each(|(e, mut mof, pop)| {
        let Some(workers) = citizens_per_colony.get(&e) else {
            return;
        };
        mof.update(*workers, pop.count);
    });
}
