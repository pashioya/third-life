use std::collections::HashMap;

use bevy::prelude::*;
use bevy_async_task::AsyncTaskPool;

use crate::{
    time::DateChanged,
    worlds::{
        env_and_infra::components::{
            CivilInfrastructure, EcosystemVitality, EnvironmentalHealth, SanitationInfrastructure,
        }, food::{
            components::{CarbResource, MeatResource, ResourceOf},
            events::{CarbConsumedEvent, CarbCreated, CowFarmCreated, MeatConsumedEvent, MeatCreated, WheatFarmCreated},
        }, population::{
            components::Population,
            events::{CitizenCreated, CitizenDied, DeathReason},
        }, wealth::components::Treasury, WorldColony
    },
    SimulationState,
};

use super::{
    components::{PostgresDB, ShouldSaveToDatabase, SimulationUuid},
    postgres_types::{ColonyEventsContainer, ColonyRecord, EventsContainer},
};

pub struct DataRecordingPlugin;

impl Plugin for DataRecordingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (record_daily_data, event_sourcer)
                .run_if(in_state(SimulationState::Running))
                .run_if(in_state(ShouldSaveToDatabase::Yes)),
        )
        .init_resource::<ColonyEventsContainer>();
    }
}

pub fn record_daily_data(
    uuid: Res<SimulationUuid>,
    db: Res<PostgresDB>,
    mut task_pool: AsyncTaskPool<()>,
    colonies: Query<
        (
            Entity,
            &WorldColony,
            &CivilInfrastructure,
            &SanitationInfrastructure,
            &EnvironmentalHealth,
            &EcosystemVitality,
            &Treasury
        ),
        With<WorldColony>,
    >,
    mut day_changed: EventReader<DateChanged>,
    meat_resources: Query<(&MeatResource, &ResourceOf)>,
    carb_resources: Query<(&CarbResource, &ResourceOf)>,
    populations: Query<(Entity, &Population)>,
    mut event_container: ResMut<ColonyEventsContainer>,
) {
    let Some(DateChanged { date }) = day_changed.read().last() else {
        return;
    };

    let mut records = colonies
        .into_iter()
        .map(
            |(
                e,
                colony,
                CivilInfrastructure {
                    urbanization_index,
                    farming_mechanization,
                },
                SanitationInfrastructure {
                    health_index_score,
                    live_birth_mortality_rate,
                    global_hunger_index,
                    actual_infant_death_ratio,
                },
                EnvironmentalHealth {
                    indoor_air_pollution,
                    drinking_water,
                    urban_particulates,
                },
                EcosystemVitality {
                    air_quality_index,
                    productive_natural_resources,
                    biodiversity,
                },
                treasury
            )| {
                (e, {
                    let mut record = ColonyRecord::new(uuid.0.clone(), date, e);
                    record.farm_space = colony.farm_space();
                    record.human_space = colony.human_space();
                    record.urbanization_index = *urbanization_index;
                    record.farming_mechanization = *farming_mechanization;
                    record.health_index_score = *health_index_score;
                    record.live_birth_mortality_rate = *live_birth_mortality_rate;
                    record.global_hunger_index = *global_hunger_index;
                    record.actual_infant_death_ratio = *actual_infant_death_ratio;
                    record.indoor_air_pollution = *indoor_air_pollution;
                    record.drinking_water = *drinking_water;
                    record.urban_particulates = *urban_particulates;
                    record.air_quality_index = *air_quality_index;
                    record.productive_natural_resources = *productive_natural_resources;
                    record.biodiversity = *biodiversity;
                    record.total_wealth = treasury.total_wealth;
                    record.spending_available = treasury.spending_available;
                    record.citizen_payout = treasury.citizen_payout;
                    record.civil_spending = treasury.total_civil_spending();
                    record.sanitation_spending = treasury.total_sanitation_spending();
                    record.social_spending = treasury.total_social_spending();
                    record.environmental_spending = treasury.total_environmental_spending();
                    record
                })
            },
        )
        .collect::<HashMap<_, _>>();

    meat_resources
        .into_iter()
        .for_each(|(res, ResourceOf { colony })| {
            let record = records.get_mut(colony).unwrap();
            record.meat_resources = res.get_kgs();
            record.meat_quality = res.quality;
        });

    carb_resources
        .into_iter()
        .for_each(|(res, ResourceOf { colony })| {
            let record = records.get_mut(colony).unwrap();
            record.carb_resources = res.get_kgs();
            record.carb_quality = res.quality;
        });

    populations.into_iter().for_each(|(colony, pop)| {
        let record = records.get_mut(&colony).unwrap();
        record.total_pop = pop.count as i32;
        record.average_age = pop.average_age as f32;
        record.working_pop = pop.working_pop as i32;
        record.younglings = pop.younglings as i32;
        record.retirees = pop.retirees as i32;
        record.average_children_per_mother = pop.average_children_per_mother as f32;
        record.num_couples = pop.num_couples as i32;
        record.males = pop.males as i32;
        record.females = pop.females as i32;
        record.working_in_wheat = pop.working_in_wheat as i32;
        record.working_in_beef = pop.working_in_beef as i32;
        record.working_not_in_farming = pop.working_not_in_farming as i32;
    });

    event_container.iter().for_each(|(colony, container)| {
        let EventsContainer {
            infant_deaths,
            starvation_deaths,
            old_age_death,
            citizen_created,
            meat_consumed,
            carb_consumed,
            meat_produced,
            carb_produced,
            meat_farms_created,
            carb_farms_created,
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
        record.meat_farms_created = *meat_farms_created;
        record.carb_farms_created = *carb_farms_created;
    });
    event_container.clear();

    let pool = db.get();
    task_pool.spawn(async move {
        for (
            _,
            ColonyRecord {
                uuid,
                date,
                colony,
                total_pop,
                average_age,
                younglings,
                working_pop,
                retirees,
                average_children_per_mother,
                infant_deaths,
                starvation_deaths,
                old_age_death,
                citizen_created,
                meat_resources,
                meat_quality,
                meat_consumed,
                carb_resources,
                carb_quality,
                carb_consumed,
                meat_produced,
                carb_produced,
                num_couples,
                males,
                females,
                working_in_wheat,
                working_in_beef,
                working_not_in_farming,
                farm_space,
                human_space,
                urbanization_index,
                farming_mechanization,
                health_index_score,
                live_birth_mortality_rate,
                global_hunger_index,
                actual_infant_death_ratio,
                indoor_air_pollution,
                drinking_water,
                urban_particulates,
                air_quality_index,
                productive_natural_resources,
                biodiversity,
                total_wealth,
                spending_available,
                citizen_payout,
                civil_spending,
                sanitation_spending,
                social_spending,
                environmental_spending,
                meat_farms_created,
                carb_farms_created,
            },
        ) in records
        {
            sqlx::query(
                r#"
                insert into simulation_records (
                    uuid, date, colony, total_pop, average_age, younglings, working_pop,
                    retirees, average_children_per_mother, infant_deaths, starvation_deaths,
                    old_age_death, citizen_created, meat_resources, meat_quality, 
                    meat_consumed, carb_resources, carb_quality, carb_consumed,
                    meat_produced, carb_produced, num_couples, males, females,
                    working_in_wheat, working_in_beef, working_not_in_farming,
                    farm_space, human_space, urbanization_index, farming_mechanization,
                    health_index_score, live_birth_mortality_rate, global_hunger_index,
                    actual_infant_death_ratio, indoor_air_pollution, drinking_water,
                    urban_particulates, air_quality_index, productive_natural_resources,
                    biodiversity, total_wealth, spending_available, citizen_payout,
                    civil_spending, sanitation_spending, social_spending,
                    environmental_spending, meat_farms_created, carb_farms_created
                ) values (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26,
                    $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, 
                    $39, $40, $41, $42, $43, $44, $45, $46, $47, $48, $49, $50
                );
                "#,
            )
            .bind(uuid)
            .bind(date)
            .bind(colony)
            .bind(total_pop)
            .bind(average_age)
            .bind(younglings)
            .bind(working_pop)
            .bind(retirees)
            .bind(average_children_per_mother)
            .bind(infant_deaths)
            .bind(starvation_deaths)
            .bind(old_age_death)
            .bind(citizen_created)
            .bind(meat_resources)
            .bind(meat_quality)
            .bind(meat_consumed)
            .bind(carb_resources)
            .bind(carb_quality)
            .bind(carb_consumed)
            .bind(meat_produced)
            .bind(carb_produced)
            .bind(num_couples)
            .bind(males)
            .bind(females)
            .bind(working_in_wheat)
            .bind(working_in_beef)
            .bind(working_not_in_farming)
            .bind(farm_space)
            .bind(human_space)
            .bind(urbanization_index)
            .bind(farming_mechanization)
            .bind(health_index_score)
            .bind(live_birth_mortality_rate)
            .bind(global_hunger_index)
            .bind(actual_infant_death_ratio)
            .bind(indoor_air_pollution)
            .bind(drinking_water)
            .bind(urban_particulates)
            .bind(air_quality_index)
            .bind(productive_natural_resources)
            .bind(biodiversity)
            .bind(total_wealth)
            .bind(spending_available)
            .bind(citizen_payout)
            .bind(civil_spending)
            .bind(sanitation_spending)
            .bind(social_spending)
            .bind(environmental_spending)
            .bind(meat_farms_created)
            .bind(carb_farms_created)
            .execute(&*pool)
            .await
            .unwrap();
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
    mut meat_farms: EventReader<CowFarmCreated>,
    mut carb_farms: EventReader<WheatFarmCreated>,
) {
    meat_consumed
        .read()
        .for_each(|MeatConsumedEvent { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .meat_consumed += amount;
        });

    meat_produced
        .read()
        .for_each(|MeatCreated { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .meat_produced += amount;
        });

    carb_consumed
        .read()
        .for_each(|CarbConsumedEvent { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .carb_consumed += amount;
        });

    carb_produced
        .read()
        .for_each(|CarbCreated { colony, amount }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .carb_produced += amount;
        });
    births.read().for_each(|CitizenCreated { colony, .. }| {
        event_container
            .entry(*colony)
            .or_insert(EventsContainer::default())
            .citizen_created += 1;
    });

    deaths
        .read()
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

    meat_farms
        .read()
        .for_each(|CowFarmCreated { colony }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .meat_farms_created += 1;
        });
    carb_farms
        .read()
        .for_each(|WheatFarmCreated { colony }| {
            event_container
                .entry(*colony)
                .or_insert(EventsContainer::default())
                .meat_farms_created += 1;
        });
}
