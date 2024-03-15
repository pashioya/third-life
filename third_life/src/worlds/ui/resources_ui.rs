
use std::collections::{HashMap, VecDeque};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui::{Color32, Window, Ui}};
use chrono::NaiveDate;
use egui_plot::{Plot, BarChart, Legend, Bar, PlotPoint, PlotPoints, Line};
use crate::worlds::{
    food::{
        components::{
            CarbResource, CowFarmOf, MeatResource, ResourceOf, WheatFarmOf,
        },
        events::{CarbCreated, MeatCreated}, tracking::{CarbProduced, MeatProduced},
    }, population::components::{CarbConsumed, MeatConsumed}, ui::components::*
};

use crate::{config::ThirdLifeConfig, time::GameDate, SimulationState};

use super::f32_to_plotpoints;

pub struct ResourcesUiPlugin;

impl Plugin for ResourcesUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                resources_changed,
                update_carb_produced,
                update_meat_produced,
                update_carb_consumed,
                update_meat_consumed,
                update_ui_farms_count,
            )
                .run_if(in_state(SimulationState::Running)),
        );
    }
}

pub fn resources_changed(
    time: Res<Time>,
    mut graph: Query<(&WorldUiEntity, &mut ResourceStorage)>,
    resources: Query<(
        &ResourceOf,
        Option<&CarbResource>,
        Option<&MeatResource>
    )>,
) {
    let mut map = graph
        .iter_mut()
        .map(|(e, p)| (e.0, p))
        .collect::<HashMap<_, _>>();
    let resources_map = resources.iter()
        .map(|r| (
                r.0.colony,
                (r.1.map(|e|e.get_kgs()),
                r.2.map(|e|e.get_kgs()))
        ))
        .fold(HashMap::new(), |mut acc: HashMap<Entity, _>, (col, vals)| {
            let entry = acc.entry(col).or_insert((0., 0.));
            match vals {
                (Some(carbs), None) => entry.0 = carbs,
                (None, Some(meat)) => entry.1 = meat,
                _ => panic!("Should not be possible there should not be a single entity with two resource types")
            };
            acc
        });

    for (colony, (carb, meat)) in resources_map.iter() {
        let Some(storage) = map.get_mut(&colony) else {
            continue;
        };

        let push = storage.timer.tick(time.delta()).just_finished();

        let set_or_push = |vec: &mut Vec<f32>, amount: &f32| {
            if push {
                vec.push(0.);
                if vec.len() > 50 {
                    vec.remove(0);
                }
            }
            *vec.last_mut().unwrap() = *amount;
        };

        set_or_push(&mut storage.carb, carb);
        set_or_push(&mut storage.meat, meat);
    }
}

fn update_ui_farms_count(
    cow_farms: Query<(&CowFarmOf)>,
    wheat_farms: Query<(&WheatFarmOf)>,
    mut ui_data: Query<(&WorldUiEntity, &mut FarmsCount)>,
) {
    let cow_farms_map = cow_farms.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, usize>, cow_farm_of | {
            *acc.entry(cow_farm_of.colony).or_insert(0) += 1;
            acc
        },
    );

    let wheat_farms_map = wheat_farms.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<Entity, usize>, wheat_farm_of | {
            *acc.entry(wheat_farm_of.colony).or_insert(0) += 1;
            acc
        },
    );

    for (world_ui_entity, mut farms_count) in ui_data.iter_mut() {
        farms_count.meat = cow_farms_map.get(&world_ui_entity.0).unwrap().clone();
        farms_count.carb = wheat_farms_map.get(&world_ui_entity.0).unwrap().clone();
    }
}

fn update_carb_produced(
    carb_produced: Query<(Entity, &CarbProduced)>,
    mut ui_data: Query<(&WorldUiEntity, &mut ResourceProduction)>
) {
    for (world_ui_entity, mut resource_production) in ui_data.iter_mut() {
        resource_production.carb = carb_produced.get(world_ui_entity.0).unwrap().1.amount;
    }
}

fn update_meat_produced(
    meat_produced: Query<(Entity, &MeatProduced)>,
    mut ui_data: Query<(&WorldUiEntity, &mut ResourceProduction)>
) {
    for (world_ui_entity, mut resource_production) in ui_data.iter_mut() {
        resource_production.meat = meat_produced.get(world_ui_entity.0).unwrap().1.amount;
    }
}

fn update_carb_consumed(
    carb_consumed: Query<(Entity, &CarbConsumed)>,
    mut ui_data: Query<(&WorldUiEntity, &mut ResourceConsumption)>
) {
    for (world_ui_entity, mut resource_consumption) in ui_data.iter_mut() {
        resource_consumption.carb = carb_consumed.get(world_ui_entity.0).unwrap().1.amount;
    }
}

fn update_meat_consumed(
    meat_produced: Query<(Entity, &MeatConsumed)>,
    mut ui_data: Query<(&WorldUiEntity, &mut ResourceConsumption)>
) {
    for (world_ui_entity, mut resource_consumption) in ui_data.iter_mut() {
        resource_consumption.meat = meat_produced.get(world_ui_entity.0).unwrap().1.amount;
    }
}

pub fn resources(
    name: &str,
    ui: &mut Ui,
    stor: &ResourceStorage,
    farms: &FarmsCount,
    prod: &ResourceProduction,
    cons: &ResourceConsumption,
) {
    ui.heading("Resources");
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(format!(
                "Meat Farms: {:?} (+{:?}: -{:?})",
                farms.meat, prod.meat, cons.meat
            ));
            ui.label(format!("Meat in storage {:.2}", stor.meat.last().unwrap()));
            plot_resource_line(format!("Meat Line {name}"), ui, &stor.meat);
            ui.label(format!(
                "Carb Farms: {:?} (+{:?}: -{:?})",
                farms.carb, prod.carb, cons.carb
            ));
            ui.label(format!("Carbs in storage {:.2}", stor.carb.last().unwrap()));
            plot_resource_line(format!("Carb Line {name}"), ui, &stor.carb);
        });
    });
}

pub fn plot_resource_line(label: String, ui: &mut Ui, vec: &Vec<f32>) {
    Plot::new(label)
        .height(75.)
        .width(400.)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(f32_to_plotpoints(vec))
                    .color(Color32::from_rgb(100, 200, 100))
                    .name("line"),
            );
        });
}
