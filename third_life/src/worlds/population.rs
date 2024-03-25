pub mod components;
mod dying;
pub mod events;
mod food_consumption;
mod giving_birth;
mod growing;
mod relationships;

use components::*;
use dying::*;
use events::*;
use giving_birth::*;
use relationships::*;

use crate::{
    time::{DateChanged, GameDate},
    SimulationState,
};
use bevy::{prelude::*, utils::hashbrown::HashMap};

use chrono::{Datelike, NaiveDate};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, SkewNormal};

use self::{food_consumption::FoodConsumptionPlugin, growing::GrowingPlugin};

use super::{
    config::WorldConfig,
    food::components::{CowFarmer, WheatFarmer},
    init_colonies, WorldColony,
};

pub struct PopulationPlugin;

impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimulationState::Running),
            (init_citizens).chain().after(init_colonies),
        )
        .add_systems(
            Update,
            (update_population, check_birthdays, come_of_age, retirement, take_up_space)
                .run_if(in_state(SimulationState::Running)),
        )
        .add_plugins((
            GivingBirthPlugin,
            DeathsPlugin,
            RelationshipsPlugin,
            FoodConsumptionPlugin,
            GrowingPlugin,
        ))
        .add_event::<CitizenBirthday>();
    }
}

pub fn init_citizens(
    colonies: Query<(Entity, &WorldConfig), With<WorldColony>>,
    mut commands: Commands,
    mut event_writer: EventWriter<CitizenCreated>,
    game_date: Res<GameDate>,
) {
    for (colony, pop_config) in colonies.iter() {
        let pop_config = pop_config.population();
        let mut rng = thread_rng();
        //let name_rng = RNG::try_from(&Language::Roman).unwrap();
        let skew_normal = SkewNormal::new(
            pop_config.population_dist().location(),
            pop_config.population_dist().scale(),
            pop_config.population_dist().shape(),
        )
        .unwrap();
        let mut age_gen = skew_normal.sample_iter(&mut rng);
        let year = game_date.date.year_ce().1 as usize;

        for _ in 0..pop_config.population_size() {
            let age = age_gen.next().unwrap().floor() as usize;
            let birthday = NaiveDate::from_yo_opt(
                (year - age).try_into().unwrap(),
                thread_rng().gen_range(1..=365),
            )
            .unwrap();
            create_citizen(**game_date, colony, birthday, pop_config.clone())
                .spawn(&mut commands);

            event_writer.send(CitizenCreated { age, colony, mother_age: None});
        }
        commands.entity(colony).try_insert(Population::default());
    }
}

pub fn update_population(
    game_date: Res<GameDate>,
    mut populations: Query<(Entity, &mut Population)>,
    citizens: Query<(&Citizen, &CitizenOf)>,
    working_pop: Query<&CitizenOf, (Without<Youngling>, Without<Retiree>, Without<Pregnancy>)>,
    younglings: Query<&CitizenOf, With<Youngling>>,
    retirees: Query<&CitizenOf, With<Retiree>>,
    women: Query<(&CitizenOf, &Female)>,
    spouses: Query<&CitizenOf, With<Spouse>>,
    men: Query<&CitizenOf, With<Male>>,
    wheat_farmer: Query<&CitizenOf, With<WheatFarmer>>,
    beef_farmer: Query<&CitizenOf, With<CowFarmer>>,
) {
    let citizens = citizens.iter()
        .fold(HashMap::new(), |mut acc, (Citizen { birthday, height, weight, ..}, CitizenOf { colony })| {
            let age = game_date.date.years_since(*birthday).unwrap() as f32;
            let (
                ref mut ages,
                ref mut count,
                ref mut heights,
                ref mut weights,
            ) = acc.entry(colony).or_insert((vec![], 0, vec![], vec![]));
            ages.push(age);
            *count += 1;
            heights.push(*height);
            weights.push(*weight);
            acc
        }).into_iter().fold(HashMap::new(), |mut acc, (colony, (ages, count, heights, weights))| {
            let avg_age = ages.iter().sum::<f32>() / ages.len() as f32;
            let avg_heights = heights.iter().sum::<f32>() / heights.len() as f32;
            let avg_weights = weights.iter().sum::<f32>() / weights.len() as f32;
            acc.entry(colony).or_insert((avg_age, count, avg_heights, avg_weights));
            acc
        });
    let younglings = younglings.into_iter()
        .fold(HashMap::new(), |mut acc, CitizenOf { colony }| {
            *acc.entry(colony).or_insert(0) += 1; acc
        });
    let working_pop = working_pop.into_iter()
        .fold(HashMap::new(), |mut acc, CitizenOf { colony }| {
            *acc.entry(colony).or_insert(0) += 1; acc
        });
    let retirees = retirees.into_iter()
        .fold(HashMap::new(), |mut acc, CitizenOf { colony }| {
            *acc.entry(colony).or_insert(0) += 1; acc
        });
    let women = women.into_iter()
        .fold(HashMap::new(), |mut acc, (CitizenOf { colony }, Female { children_had, last_child_birth_date })| {
            let (
                ref mut count,
                ref mut children_had_vec
            ) = acc.entry(colony).or_insert((0, vec![]));
            *count += 1;
            if last_child_birth_date.is_some() {
                children_had_vec.push(*children_had as f32);
            };
            acc
        }).into_iter().fold(HashMap::new(), |mut acc, (colony, (count, children_had_vec))| {
            let avg_children_had = children_had_vec.iter().sum::<f32>() / children_had_vec.len() as f32;
            acc.entry(colony).or_insert((count, avg_children_had));
            acc
        });

    let spouses = spouses.into_iter()
        .fold(HashMap::new(), |mut acc, CitizenOf { colony } | {
              *acc.entry(colony).or_insert(0) += 1; acc
        });
    let men_counts = men.into_iter()
        .fold(HashMap::new(), |mut acc, CitizenOf { colony }| {
            *acc.entry(colony).or_insert(0) += 1; acc
        });

    let wheat_farmer = wheat_farmer.into_iter()
        .fold(HashMap::new(), |mut acc, CitizenOf { colony } | {
              *acc.entry(colony).or_insert(0) += 1; acc
        });
    let beef_farmer = beef_farmer.into_iter()
        .fold(HashMap::new(), |mut acc, CitizenOf { colony } | {
              *acc.entry(colony).or_insert(0) += 1; acc
        });

    for (colony, mut population) in &mut populations.iter_mut() {

        population.younglings = *younglings.get(&colony).unwrap_or(&0);
        population.retirees = *retirees.get(&colony).unwrap_or(&0); 
        population.working_pop = *working_pop.get(&colony).unwrap_or(&0); 

        let (women_count, average_children_per_mother) = women.get(&colony).unwrap_or(&(0, 0.));
        population.average_children_per_mother = *average_children_per_mother;
        population.females = *women_count;
        population.males = *men_counts.get(&colony).unwrap_or(&0);

        let (
            ref avg_age, ref count, ref avg_height, ref avg_weight,
        ) = *citizens.get(&colony).unwrap_or(&(0., 0, 0., 0.));

        population.average_age = avg_age.floor() as usize;

        population.count = *count;
        population.average_height = *avg_height;
        population.average_weight = *avg_weight;
        population.num_couples = spouses.get(&colony).map(|n| n/2).unwrap_or(0);
        population.working_in_wheat = *wheat_farmer.get(&colony).unwrap_or(&0);
        population.working_in_beef = *beef_farmer.get(&colony).unwrap_or(&0);
        if population.working_pop >= (population.working_in_wheat + population.working_in_beef) {
            population.working_not_in_farming = population.working_pop - (population.working_in_wheat + population.working_in_beef);
        } else {
            population.working_not_in_farming = 0;
        }
    }
}

pub fn check_birthdays(
    mut day_changed_event_reader: EventReader<DateChanged>,
    mut birthday_events: EventWriter<CitizenBirthday>,
    citizens: Query<(Entity, &Citizen, &CitizenOf)>,
) {
    for day_changed_event in day_changed_event_reader.read() {
        let game_date = day_changed_event.date;
        for (citizen_entity, citizen, citizen_of) in citizens.iter() {
            if game_date.month() == citizen.birthday.month()
                && game_date.day() == citizen.birthday.day()
            {
                birthday_events.send(CitizenBirthday {
                    entity: citizen_entity,
                    colony: citizen_of.colony,
                    age: game_date.years_since(citizen.birthday).unwrap() as usize,
                });
            }
        }
    }
}

pub fn come_of_age(
    mut commands: Commands,
    mut birthday_event_reader: EventReader<CitizenBirthday>,
    colonies: Query<(Entity, &WorldConfig)>,
) {
    for birthday in birthday_event_reader.read() {
        let config = colonies.get(birthday.colony).unwrap().1;
        if birthday.age == config.population().age_of_adult() {
            commands.get_entity(birthday.entity).map(|mut e| {
                e.remove::<Youngling>();
            });
        }
    }
}

pub fn retirement(
    mut commands: Commands,
    mut birthday_event_reader: EventReader<CitizenBirthday>,
    colonies: Query<(Entity, &WorldConfig)>,
) {
    for birthday in birthday_event_reader.read() {
        let config = colonies.get(birthday.colony).unwrap().1;
        if birthday.age == config.population().age_of_retirement() {
            commands.get_entity(birthday.entity).map(|mut e| {
                e.remove::<WheatFarmer>();
                e.remove::<CowFarmer>();
                e.remove::<Employed>();
                e.try_insert(Retiree);
            });
        }
    }
}

pub fn take_up_space(
    mut colonies: Query<(Entity, &mut WorldColony, &WorldConfig)>,
    mut died: EventReader<CitizenDied>,
    mut born: EventReader<CitizenCreated>,
) {
    let mut colonies = colonies.iter_mut().map(|e|(e.0, (e.1, e.2))).collect::<HashMap<_, _>>();
    for CitizenDied { colony, .. } in died.read() {
        colonies.get_mut(colony).map(|(colony, config)| {
            colony.free_up_human_space(config.population().space_per_person());
        });
    }
    for CitizenCreated { colony, .. } in born.read() {
        colonies.get_mut(colony).map(|(colony, config)| {
            colony.take_up_human_space(config.population().space_per_person());
        });
    }
}
