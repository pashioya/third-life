use std::collections::HashMap;

use bevy::prelude::*;
use bevy_async_task::AsyncTaskPool;

use crate::{
    time::DateChanged,
    worlds::{food::{
        components::{CarbResource, MeatResource, ResourceOf},
        events::{CarbConsumedEvent, CarbCreated, MeatConsumedEvent, MeatCreated}, tracking::{CarbProduced, MeatProduced},
    }, population::{components::Population, events::{CitizenCreated, CitizenDied, DeathReason}}, WorldColony},
    SimulationState,
};

use super::{components::{PostgresDB, ShouldSaveToDatabase, SimulationUuid}, postgres_types::{ColonyEventsContainer, ColonyRecord, EventsContainer}};

pub struct DataRecordingPlugin;

impl Plugin for DataRecordingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (record_daily_data, event_sourcer)
                .run_if(in_state(SimulationState::Running))
                .run_if(in_state(ShouldSaveToDatabase::Yes)),
        ).init_resource::<ColonyEventsContainer>();
    }
}

pub fn record_daily_data(
    uuid: Res<SimulationUuid>,
    db: Res<PostgresDB>,
    mut task_pool: AsyncTaskPool<()>,
    colonies: Query<Entity, With<WorldColony>>,
    mut day_changed: EventReader<DateChanged>,
    meat_resources: Query<(&MeatResource, &ResourceOf)>,
    carb_resources: Query<(&CarbResource, &ResourceOf)>,
    populations: Query<(Entity, &Population)>,
    mut event_container: ResMut<ColonyEventsContainer>,
) {
    let Some(DateChanged { date }) = day_changed.read().last() else {
        return;
    };

    let mut records = colonies.into_iter().map(|e| (e, ColonyRecord::new(uuid.0.clone(), date, e))).collect::<HashMap<_, _>>();

    meat_resources.into_iter()
        .for_each(|(res, ResourceOf { colony })| {
            let record = records.get_mut(colony).unwrap();
            record.meat_resources = res.get_kgs();
            record.meat_quality = res.quality;
        });

    carb_resources.into_iter()
        .for_each(|(res, ResourceOf { colony })| {
            let record = records.get_mut(colony).unwrap();
            record.carb_resources = res.get_kgs();
            record.carb_quality = res.quality;
        });

    populations.into_iter()
        .for_each(|(colony, pop)| {
            let record = records.get_mut(&colony).unwrap();
            record.total_pop = pop.count as i32;
            record.average_age = pop.average_age as f32;
            record.working_pop = pop.working_pop as i32;
            record.younglings = pop.younglings as i32;
            record.retirees = pop.retirees as i32;
            record.average_children_per_mother = pop.average_children_per_mother as f32;
        });



    event_container.iter()
        .for_each(|(colony, container)| {
            let EventsContainer { 
                infant_deaths, starvation_deaths, old_age_death,
                citizen_created, meat_consumed, carb_consumed, meat_produced,
                carb_produced
            } = container;
            let record = records.get_mut(colony).unwrap();
            record.infant_deaths = *infant_deaths;
            record.starvation_deaths = *starvation_deaths;
            record.old_age_death = *old_age_death;
            record.citizen_created = *citizen_created;
            record.meat_consumed = *meat_consumed;
            record.carb_consumed = *carb_consumed;
            record.meat_produced = *meat_produced;
            record.carb_produced = *carb_produced;
        });
    event_container.clear();


    let pool = db.get();
    task_pool.spawn(async move {
        for (_, ColonyRecord { 
            uuid, date, colony, total_pop, average_age, younglings, working_pop, 
            retirees, average_children_per_mother, infant_deaths, starvation_deaths, 
            old_age_death, citizen_created, meat_resources, meat_quality, 
            meat_consumed, carb_resources, carb_quality, carb_consumed, meat_produced,
            carb_produced
        }) in records {
            sqlx::query(r#"
                        insert into simulation_records (
                            uuid, date, colony, total_pop, average_age, younglings, working_pop,
                            retirees, average_children_per_mother, infant_deaths, starvation_deaths,
                            old_age_death, citizen_created, meat_resources, meat_quality, 
                            meat_consumed, carb_resources, carb_quality, carb_consumed,
                            meat_produced, carb_produced
                            ) values (
                                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                                $15, $16, $17, $18, $19, $20, $21
                                );
                        "#)
                .bind(uuid).bind(date).bind(colony).bind(total_pop).bind(average_age)
                .bind(younglings).bind(working_pop).bind(retirees)
                .bind(average_children_per_mother).bind(infant_deaths)
                .bind(starvation_deaths).bind(old_age_death).bind(citizen_created)
                .bind(meat_resources).bind(meat_quality).bind(meat_consumed)
                .bind(carb_resources).bind(carb_quality).bind(carb_consumed)
                .bind(meat_produced).bind(carb_produced)
                .execute(&*pool).await.unwrap();
        }
    });
}


pub fn event_sourcer(
    mut event_container: ResMut<ColonyEventsContainer>,
    mut meat_consumed: EventReader<MeatConsumedEvent>,
    mut meat_produced: EventReader<MeatCreated>,
    mut carb_consumed: EventReader<CarbConsumedEvent>,
    mut carb_produced: EventReader<CarbCreated>,
    mut births: EventReader<CitizenCreated>,
    mut deaths: EventReader<CitizenDied>,
) {
    meat_consumed.read()
        .for_each(|MeatConsumedEvent { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .meat_consumed += amount; 
        });

    meat_produced.read()
        .for_each(|MeatCreated { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .meat_produced += amount;
        });


    carb_consumed.read()
        .for_each(|CarbConsumedEvent { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .carb_consumed += amount;
        });

    carb_produced.read()
        .for_each(|CarbCreated { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .carb_produced += amount;
        });
    births.read()
        .for_each(|CitizenCreated { colony, .. }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .citizen_created += 1;
        });

    deaths.read()
        .for_each(|CitizenDied { colony, reason, .. }| {
            let map = event_container
                .entry(*colony)
                .or_insert(EventsContainer::default());
            match reason {
                DeathReason::InfantDeath => map.infant_deaths += 1,
                DeathReason::Starvation => map.starvation_deaths += 1,
                DeathReason::OldAge => map.old_age_death += 1,
            };
        });
}
